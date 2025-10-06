/*
 * src/models/Account.rs
 *
 * File for Account Controller API Endpoints
 *
 * Purpose:
 *   Serve Account Related API Requests
 *
 * Include:
 *   api_signup         - /account/signup -> serves signup functionality
 *   api_login          - /account/login  -> serves login functionality
 *   api_test           - /account/test   -> serves test of account api functionality
 */

use axum::{
    Extension, Json, Router,
    http::StatusCode,
    routing::post,
};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use tower_cookies::{
    Cookie, Cookies,
    cookie::{SameSite, time::Duration, Key},
};

use sqlx::PgPool;
use tracing::{error, info};
use chrono::{Utc, Duration as ChronoDuration};
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use base64::Engine;

use crate::error::ApiResult;
use crate::models::account::*;

/// Create a new user.
///
/// # Method
/// `POST /account/signup`
///
/// # Request Body
/// - `email`: A valid email address (string, required).
/// - `first_name`: The user's first name (string, required).
/// - 'last_name': The user's last name (string, required).
/// - 'password': The user's password (string, required).
///
/// # Responses
/// - `201 CREATED` - Signup successful with JSON body `{ "id": i32, "email": string }`
/// - `409 CONFLICT` - Email already exists in the database
/// - `500 INTERNAL_SERVER_ERROR` - Database error or password hashing failure
///
/// # Examples
/// ```bash
/// curl -X POST http://localhost:3000/account/signgup
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
        "HANDLER ->> /api/signup 'api_signup' - Payload: {:?}",
        payload
    );

    // Validate input
    if let Err(validation_error) = payload.validate() {
        error!(
            "ERROR ->> /api/signup 'api_signup' REASON: Validation failed: {}",
            validation_error
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check if user already exists
    let existing_user_result =
        sqlx::query!("SELECT id FROM accounts WHERE email = $1", payload.email)
            .fetch_optional(&pool)
            .await;

    match existing_user_result {
        Ok(Some(_)) => {
            error!(
                "ERROR ->> /api/signup 'api_signup' REASON: Email already exists: {}",
                payload.email
            );
            return Err(StatusCode::CONFLICT);
        }
        Err(e) => {
            error!(
                "ERROR ->> /api/signup 'api_signup' REASON: Database query error: {:?}",
                e
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
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
                "ERROR ->> /api/signup 'api_signup' REASON: Failed to hash password: {:?}",
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR
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
                "INFO ->> /api/signup 'api_signup' - Created user with ID: {}",
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
                "ERROR ->> /api/signup 'api_signup' REASON: Database insert error: {:?}",
                e
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Attempt user login
///
/// # Method
/// `POST /account/login`
///
/// # Request Body
/// - `email`: A valid email address (string, required).
/// - 'password': The user's password (string, required).
///
/// # Responses
/// - `200 OK` - Login successful with JSON body `{ "id": i32, "token": string }` + auth cookie set
/// - `400 BAD_REQUEST` - Invalid credentials (wrong email or password)
/// - `500 INTERNAL_SERVER_ERROR` - Database error or password verification failure
///
/// # Examples
/// ```bash
/// curl -X POST http://localhost:3000/account/login
///   -H "Content-Type: application/json"
///   -d '{
///        "email": "alice@example.com",
///        "password": "password123."
///       }'
/// ```
///
pub async fn api_login(
    cookies: Cookies,
    Extension(key): Extension<Key>,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<LoginPayload>,
) -> ApiResult<Json<LoginResponse>> {
    info!(
        "HANDLER ->> /api/login 'api_login' - Payload: {:?}",
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
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Attempt to match the password hashes
            if let Err(_) =
                Argon2::default().verify_password(payload.password.as_bytes(), &parsed_hash)
            {
                return Err(StatusCode::BAD_REQUEST);
            }

            // Create token and set cookie as before
            let domain = option_env!("DOMAIN").unwrap_or("localhost");
            let app_env = option_env!("APP_ENV").unwrap_or("development");
            let on_production = app_env == "production";

            // Create a token value (in a real app, this would be a JWT or similar)
            // Embed expiration epoch seconds inside the token for server-side validation
            let exp_epoch = (Utc::now() + ChronoDuration::days(3)).timestamp();
            let raw_token = format!("user-{}.{}.sign", result.id, exp_epoch);
            let token_value = B64.encode(raw_token.as_bytes());

            info!(
                "INFO ->> /api/login 'api_login' - Generated token value: {}. Production is: {}",
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
                "ERROR ->> /api/signup 'api_signup' REASON: No account for Email: {}",
                payload.email
            );
            return Err(StatusCode::BAD_REQUEST);
        }
    }
}

pub fn account_routes() -> Router {
    Router::new()
        .route("/signup", post(api_signup))
        .route("/login", post(api_login))
}
