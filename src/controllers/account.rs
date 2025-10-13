/*
 * src/controllers/account.rs
 *
 * File for Account Controller API Endpoints
 *
 * Purpose:
 *   Serve Account Related API Requests
 *
 * Include:
 *   api_signup         - POST /api/account/signup -> creates an account
 *   api_login          - POST /api/account/login  -> authenticates and sets auth cookie
 *   api_me             - POST /api/account/me     -> returns current user (protected by middleware)
 */

use axum::{Extension, Json, Router, http::StatusCode, routing::post};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use tower_cookies::{
    Cookie, Cookies,
    cookie::{Key, SameSite, time::Duration},
};

use chrono::{Duration as ChronoDuration, Utc};
use sqlx::PgPool;
use tracing::{error, info};

use crate::error::{ApiResult, AppError, PrivateError, PublicError};
use crate::middleware::{AuthUser, auth_middleware};
use crate::models::account::*;

/// Create a new user.
///
/// # Method
/// `POST /api/account/signup`
///
/// # Request Body
/// - `email`: A valid email address (string, required).
/// - `first_name`: The user's first name (string, required).
/// - 'last_name': The user's last name (string, required).
/// - 'password': The user's password (string, required).
///
/// # Responses
/// - `201 CREATED` - Signup successful with JSON body `{ "id": i32, "email": string }`
/// - `400 BAD_REQUEST` - Validation failure (public error)
/// - `409 CONFLICT` - Email already exists (public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X POST http://localhost:3000/api/account/signup
///   -H "Content-Type: application/json"
///   -d '{
///        "email": "alice@example.com",
///        "first_name": "alice",
///        "last_name": "grace",
///        "password": "password123."
///       }'
/// ```
///
pub async fn api_signup(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<SignupPayload>,
) -> ApiResult<(StatusCode, Json<SignupResponse>)> {
    info!(
        "HANDLER ->> /api/account/signup 'api_signup' - Payload: {:?}",
        payload
    );

    // Validate input
    if let Err(validation_error) = payload.validate() {
        error!(
            "ERROR ->> /api/account/signup 'api_signup' REASON: Validation failed: {}",
            validation_error
        );
        return Err(PublicError::Validation(validation_error).into());
    }

    // Check if user already exists
    let existing_user_result =
        sqlx::query!("SELECT id FROM accounts WHERE email = $1", payload.email)
            .fetch_optional(&pool)
            .await;

    match existing_user_result {
        Ok(Some(_)) => {
            error!(
                "ERROR ->> /api/account/signup 'api_signup' REASON: Email already exists: {}",
                payload.email
            );
            return Err(PublicError::Conflict("email already exists".to_string()).into());
        }
        Err(e) => {
            error!(
                "ERROR ->> /api/account/signup 'api_signup' REASON: Database query error: {:?}",
                e
            );
            return Err(AppError::from(PrivateError::Db(e)));
        }
        Ok(None) => {
            // User doesn't exist, proceed with signup
        }
    }

    // Hash the password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| {
            error!(
                "ERROR ->> /api/account/signup 'api_signup' REASON: Failed to hash password: {:?}",
                e
            );
            AppError::from(PrivateError::PasswordHash(e))
        })?
        .to_string();

    // Insert new user into database
    let insert_result = sqlx::query!(
        "INSERT INTO accounts (email, first_name, last_name, password)
         VALUES ($1, $2, $3, $4)
         RETURNING id",
        payload.email,
        payload.first_name,
        payload.last_name,
        password_hash
    )
    .fetch_one(&pool)
    .await;

    match insert_result {
        Ok(record) => {
            info!(
                "INFO ->> /api/account/signup 'api_signup' - Created user with ID: {}",
                record.id
            );

            Ok((
                StatusCode::CREATED,
                Json(SignupResponse {
                    id: record.id,
                    email: payload.email,
                }),
            ))
        }
        Err(e) => {
            error!(
                "ERROR ->> /api/account/signup 'api_signup' REASON: Database insert error: {:?}",
                e
            );
            Err(AppError::from(PrivateError::Db(e)))
        }
    }
}

/// Attempt user login
///
/// # Method
/// `POST /api/account/login`
///
/// # Request Body
/// - `email`: A valid email address (string, required).
/// - 'password': The user's password (string, required).
///
/// # Responses
/// - `200 OK` - Login successful with JSON body `{ "id": i32, "token": string }` and `auth-token` private cookie set
/// - `400 BAD_REQUEST` - Invalid credentials (public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X POST http://localhost:3000/api/account/login
///   -H "Content-Type: application/json"
///   -d '{
///        "email": "alice@example.com",
///        "password": "password123."
///       }'
/// ```
///
/// Notes:
/// - Token format is `user-<id>.<exp>.sign`, where `<exp>` is epoch seconds (UTC) ~3 days out.
/// - Cookie name is `auth-token`; in development it uses `SameSite=Lax`, not `Secure`.
///
pub async fn api_login(
    cookies: Cookies,
    Extension(key): Extension<Key>,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<LoginPayload>,
) -> ApiResult<Json<LoginResponse>> {
    info!(
        "HANDLER ->> /api/account/login 'api_login' - Payload: {:?}",
        payload
    );

    // Get user from database
    let user_result = sqlx::query!(
        "SELECT id, email, password
         FROM accounts
         WHERE email = $1;",
        payload.email
    )
    .fetch_one(&pool)
    .await;

    match user_result {
        Ok(result) => {
            // Verify password
            let parsed_hash = PasswordHash::new(&result.password)
                .map_err(|e| AppError::from(PrivateError::PasswordHash(e)))?;

            // Attempt to match the password hashes
            if let Err(_) =
                Argon2::default().verify_password(payload.password.as_bytes(), &parsed_hash)
            {
                return Err(PublicError::BadRequest("invalid credentials".to_string()).into());
            }

            // Create token and set cookie as before
            let domain = option_env!("DOMAIN").unwrap_or("localhost");
            let app_env = option_env!("APP_ENV").unwrap_or("development");
            let on_production = app_env == "production";

            // Create a token value (in a real app, this would be a JWT or similar)
            // Embed expiration epoch seconds inside the token for server-side validation
            let exp_epoch = (Utc::now() + ChronoDuration::days(3)).timestamp();
            let token_value = format!("user-{}.{}.sign", result.id, exp_epoch);

            info!(
                "INFO ->> /api/account/login 'api_login' - Generated token value: {}. Production is: {}",
                token_value, on_production
            );

            // Build the cookie with enhanced security
            // Store encrypted (private) cookie so value is confidential and authenticated
            let cookie = Cookie::build("auth-token", token_value.clone())
                .domain(domain.to_string())
                .path("/")
                .secure(on_production)
                .http_only(true)
                .same_site(if on_production {
                    SameSite::None
                } else {
                    SameSite::Lax
                })
                .max_age(Duration::days(3))
                .finish();

            // encrypt/sign cookie (private cookie via CookieManagerLayer key)
            cookies.private(&key).add(cookie.clone());

            return Ok(Json(LoginResponse {
                id: result.id,
                token: token_value,
            }));
        }
        Err(_) => {
            error!(
                "ERROR ->> /api/account/signup 'api_signup' REASON: No account for Email: {}",
                payload.email
            );
            return Err(PublicError::BadRequest("invalid credentials".to_string()).into());
        }
    }
}

/// Return the current authenticated user's ID.
/// Hit this route to validate the `auth-token` private cookie.
///
/// # Method
/// `POST /api/account/validate`
///
/// # Auth
/// Protected by `auth_middleware` which validates the `auth-token` private cookie,
/// checks expiration, and injects `Extension<AuthUser>`.
///
/// # Responses
/// - `200 OK` - `{ "id": i32 }` for the authenticated user
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
pub async fn api_validate(Extension(user): Extension<AuthUser>) -> ApiResult<Json<ValidateResponse>> {
    Ok(Json(ValidateResponse { id: user.id }))
}

pub fn account_routes() -> Router {
    Router::new()
        .route("/validate", post(api_validate))
        .route_layer(axum::middleware::from_fn(auth_middleware))
        .route("/signup", post(api_signup))
        .route("/login", post(api_login))
}