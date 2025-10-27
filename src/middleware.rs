use crate::error::AppError;
use axum::{extract::Request, middleware::Next, response::IntoResponse};
use chrono::Utc;
use sqlx::PgPool;
use tower_cookies::{cookie::{time::{Duration, OffsetDateTime}, Cookie, Key, SameSite}, Cookies};

/// Inserted into request extensions on authenticated requests
#[derive(Clone, Copy, Debug)]
pub struct AuthUser {
	pub id: i32,
}

/// Auth middleware for account routes
/// - Decrypts `auth-token` private cookie using `Key` from extensions
/// - Validates embedded expiration and that the user exists in DB
/// - Inserts `AuthUser` into request extensions on success; otherwise 401
pub async fn middleware_auth(cookies: Cookies, mut req: Request, next: Next) -> impl IntoResponse {
	let key = match req.extensions().get::<Key>() {
		Some(k) => k.clone(),
		None => return AppError::Unauthorized.into_response(),
	};
	let pool = match req.extensions().get::<PgPool>() {
		Some(p) => p.clone(),
		None => return AppError::Unauthorized.into_response(),
	};

	// Decrypt private cookie and extract token
	let decrypted = match cookies.private(&key).get("auth-token") {
		Some(c) => c,
		None => return AppError::Unauthorized.into_response(),
	};
	let token = decrypted.value().to_string();

	// Expect format: user-<id>.<exp>.sign
	let parts: Vec<&str> = token.split('.').collect();

	if parts.len() != 3 || parts[2] != "sign" || !parts[0].starts_with("user-") {
		return AppError::Unauthorized.into_response();
	}

	let user_id: i32 = match parts[0][5..].parse() {
		Ok(v) => v,
		Err(_) => return AppError::Unauthorized.into_response(),
	};

	let exp: i64 = match parts[1].parse() {
		Ok(v) => v,
		Err(_) => return AppError::Unauthorized.into_response(),
	};

	let now = Utc::now().timestamp();
	if now > exp {
		return AppError::Unauthorized.into_response();
	}

	// If the cookie will expire in less than an hour, set it's expiration to one hour from now
	let one_hour = 3600;
	if exp - now < one_hour {
		let new_exp = now + one_hour;
		let new_token = format!("user-{}.{}.sign", user_id, new_exp);

		let domain = option_env!("DOMAIN").unwrap_or("localhost");
		let app_env = option_env!("APP_ENV").unwrap_or("development");
		let on_production = app_env == "production";

		let new_cookie = Cookie::build(("auth-token", new_token.clone()))
			.domain(domain.to_string())
			.path("/")
			.secure(on_production)
			.http_only(true)
			.same_site(if on_production {
				SameSite::Strict
			} else {
				SameSite::Lax
			})
			.expires(OffsetDateTime::now_utc().saturating_add(Duration::hours(1)))
			.max_age(Duration::hours(1))
			.build();

		cookies.private(&key).add(new_cookie);
	}

	// Ensure user exists
	let exists_row =
		sqlx::query_as::<_, (bool,)>("SELECT EXISTS(SELECT 1 FROM accounts WHERE id = $1)")
			.bind(user_id)
			.fetch_one(&pool)
			.await
			.unwrap_or((false,));

	if !exists_row.0 {
		return AppError::Unauthorized.into_response();
	}

	// Attach user to request
	req.extensions_mut().insert(AuthUser { id: user_id });

	next.run(req).await
}
