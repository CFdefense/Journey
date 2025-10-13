use axum::{
    http::{Request, header},
    middleware::Next,
    response::IntoResponse,
};
use chrono::Utc;
use sqlx::PgPool;
use tower_cookies::cookie::{Cookie, CookieJar, Key};
use crate::error::{AppError, PublicError};

/// Inserted into request extensions on authenticated requests
#[derive(Clone, Copy, Debug)]
pub struct AuthUser {
    pub id: i32,
}

/// Auth middleware for account routes
/// - Decrypts `auth-token` private cookie using `Key` from extensions
/// - Validates embedded expiration and that the user exists in DB
/// - Inserts `AuthUser` into request extensions on success; otherwise 401
pub async fn auth_middleware<B>(mut req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let key = match req.extensions().get::<Key>() {
        Some(k) => k.clone(),
        None => return AppError::from(PublicError::Unauthorized).into_response(),
    };
    let pool = match req.extensions().get::<PgPool>() {
        Some(p) => p.clone(),
        None => return AppError::from(PublicError::Unauthorized).into_response(),
    };

    // Read Cookie header
    let cookie_header = match req.headers().get(header::COOKIE) {
        Some(v) => v,
        None => return AppError::from(PublicError::Unauthorized).into_response(),
    };
    let cookie_str = match cookie_header.to_str() {
        Ok(s) => s,
        Err(_) => return AppError::from(PublicError::Unauthorized).into_response(),
    };

    // Build a jar from incoming cookies
    let mut jar = CookieJar::new();
    for pair in cookie_str.split(';') {
        let s = pair.trim();
        if s.is_empty() {
            continue;
        }
        if let Ok(parsed) = Cookie::parse(s.to_string()) {
            jar.add(parsed);
        }
    }

    // Decrypt private cookie and extract token
    let decrypted = match jar.private(&key).get("auth-token") {
        Some(c) => c,
        None => return AppError::from(PublicError::Unauthorized).into_response(),
    };
    let token = decrypted.value().to_string();

    // Expect format: user-<id>.<exp>.sign
    let parts: Vec<&str> = token.split('.').collect();

    if parts.len() != 3 || parts[2] != "sign" || !parts[0].starts_with("user-") {
        return AppError::from(PublicError::Unauthorized).into_response();
    }

    let user_id: i32 = match parts[0][5..].parse() {
        Ok(v) => v,
        Err(_) => return AppError::from(PublicError::Unauthorized).into_response(),
    };

    let exp: i64 = match parts[1].parse() {
        Ok(v) => v,
        Err(_) => return AppError::from(PublicError::Unauthorized).into_response(),
    };

    if Utc::now().timestamp() > exp {
        return AppError::from(PublicError::Unauthorized).into_response();
    }

    // Ensure user exists
    let exists_row =
        sqlx::query_as::<_, (bool,)>("SELECT EXISTS(SELECT 1 FROM accounts WHERE id = $1)")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap_or((false,));

    if !exists_row.0 {
        return AppError::from(PublicError::Unauthorized).into_response();
    }

    // Attach user to request
    req.extensions_mut().insert(AuthUser { id: user_id });

    next.run(req).await
}
