/*
 * src/controllers/account.rs
 *
 * File for Account Controller API Endpoints
 *
 * Purpose:
 *   Serve Account Related API Requests
 */

use axum::{Extension, Json, Router, routing::{get, post}};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use tower_cookies::{
    cookie::{time::{Duration, OffsetDateTime}, Key, SameSite}, Cookie, Cookies
};

use sqlx::PgPool;
use tracing::debug;

use crate::error::{ApiResult, AppError};
use crate::middleware::{AuthUser, middleware_auth};
use crate::models::account::*;

/// Creates and sets the cookie containing the hashed account id, expiration time, and other data.
///
/// Notes:
/// - Token format is `user-<id>.<exp>.sign`, where `<exp>` is epoch seconds (UTC) ~3 days out.
/// - Cookie name is `auth-token`; in development it uses `SameSite=Lax`, not `Secure`.
fn set_cookie(account_id: i32, expired: bool, cookies: &Cookies, key: &Key) {
    // Create token and set cookie as before
    let domain = option_env!("DOMAIN").unwrap_or("localhost");
    let app_env = option_env!("APP_ENV").unwrap_or("development");
    let on_production = app_env == "production";

    // Create a token value (in a real app, this would be a JWT or similar)
    // Embed expiration epoch seconds inside the token for server-side validation
    let (expires, max_age) = if expired {
  		(OffsetDateTime::UNIX_EPOCH, Duration::days(0))
    } else {
    	let age = Duration::days(3);
    	(OffsetDateTime::now_utc() + age, age)
    };
    let token_value = format!("user-{}.{}.sign", account_id, expires.unix_timestamp());

    debug!(
        "INFO ->> Generated token: {}. Production is: {}",
        token_value,
        on_production
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
        .expires(Some(expires))
        .max_age(max_age)
        .finish();

    // encrypt/sign cookie (private cookie via CookieManagerLayer key)
    cookies.private(key).add(cookie);
}

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
/// - `200 OK` - Signup successful
/// - `400 BAD_REQUEST` - Validation failure (public error)
/// - `409 CONFLICT` - Email already exists (public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X POST http://localhost:3001/api/account/signup
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
	cookies: Cookies,
    Extension(key): Extension<Key>,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<SignupPayload>,
) -> ApiResult<()> {
    debug!(
        "HANDLER ->> /api/account/signup 'api_signup' - Payload: {:?}",
        payload
    );

    // Validate input
    if let Err(validation_error) = payload.validate() {
        return Err(AppError::Validation(validation_error));
    }

    // Check if user already exists
    let existing_user_result =
        sqlx::query!("SELECT id FROM accounts WHERE email = $1", payload.email)
            .fetch_optional(&pool)
            .await;

    match existing_user_result {
        Ok(Some(_)) => {
            return Err(AppError::Conflict("email already exists".to_string()));
        }
        Err(e) => {
            return Err(AppError::from(e));
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
        .map_err(|e| AppError::from(e))?
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
            debug!(
                "INFO ->> /api/account/signup 'api_signup' - Created user with id: {}",
                record.id
            );

            set_cookie(record.id, false, &cookies, &key);

            Ok(())
        }
        Err(e) => {
            Err(AppError::from(e))
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
/// - `200 OK` - Login successful with private cookie set
/// - `400 BAD_REQUEST` - Invalid credentials (public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X POST http://localhost:3001/api/account/login
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
) -> ApiResult<()> {
    debug!(
        "HANDLER ->> /api/account/login 'api_login' - Payload: {:?}",
        payload
    );

    // Get user from database as Account
    let user_result = sqlx::query_as!(
        Account,
        r#"
        SELECT
            id,
            email,
            password,
            first_name,
            last_name,
            budget_preference as "budget_preference: BudgetBucket",
            risk_preference as "risk_preference: RiskTolerence",
            food_allergies,
            disabilities
        FROM accounts
        WHERE email = $1
        "#,
        payload.email
    )
    .fetch_one(&pool)
    .await;

    match user_result {
        Ok(result) => {
            // Verify password
            let parsed_hash = PasswordHash::new(&result.password)
                .map_err(|e| AppError::from(e))?;

            // Attempt to match the password hashes
            if let Err(_) =
                Argon2::default().verify_password(payload.password.as_bytes(), &parsed_hash)
            {
                return Err(AppError::BadRequest("invalid credentials".to_string()));
            }

            set_cookie(result.id, false, &cookies, &key);

            return Ok(());
        }
        Err(e) => {
            return Err(AppError::from(e));
        }
    }
}

/// Returns whether the user has a valid auth token.
/// Hit this route to validate the `auth-token` private cookie.
///
/// # Method
/// `GET /api/account/validate`
///
/// # Auth
/// Protected by `auth_middleware` which validates the `auth-token` private cookie,
/// checks expiration, and injects `Extension<AuthUser>`.
///
/// # Responses
/// - `200 OK` - user has a valid auth token
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
pub async fn api_validate(Extension(user): Extension<AuthUser>) -> ApiResult<()> {
    debug!(
        "HANDLER ->> /api/account/validate 'api_validate' - User ID: {}",
        user.id
    );
    Ok(())
}

/// Get information about the user
///
/// # Method
/// `GET /api/account/current`
///
/// # Responses
/// - `200 OK` - with body: [CurrentResponse]
/// - `401 UNAUTHORIZED` - Invalid credentials (public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X GET http://localhost:3001/api/account/current
///   -H "Content-Type: application/json"
/// ```
pub async fn api_current(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
) -> ApiResult<Json<CurrentResponse>> {
    debug!(
        "HANDLER ->> /api/account/current 'api_current' - User ID: {}",
        user.id
    );
    // Load current user's full account row
    let account = sqlx::query_as!(
        CurrentResponse,
        r#"
        SELECT
            email,
            first_name,
            last_name,
            budget_preference as "budget_preference: BudgetBucket",
            risk_preference as "risk_preference: RiskTolerence",
            food_allergies,
            disabilities
        FROM accounts
        WHERE id = $1
        "#,
        user.id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::from(e))?;

    Ok(Json(account))
}

/// Update information about the user
///
/// # Method
/// `POST /api/account/update`
///
/// # Request Body
/// - `email`: A valid email address (string).
/// - 'first_name': The user's first name (string).
/// - 'last_name': The user's last name (string).
/// - 'password': The user's password (string).
/// - 'budget_preference': The user's budget preference (string).
/// - 'risk_preference': The user's risk preference (string).
/// - 'food_allergies': The user's allergies (string).
/// - 'disabilities': The user's disabilities (string).
///
/// # Responses
/// - `200 OK` - with body: [UpdateResponse]
/// - `401 UNAUTHORIZED` - Invalid credentials (public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X POST http://localhost:3001/api/account/update
///   -H "Content-Type: application/json"
///   -d '{
///         "email": "",
///         "first_name": "",
///         "last_name": "",
///         "password": "",
///         "budget_preference": "",
///         "risk_preference": "",
///         "food_allergies": "",
///         "disabilities": ""
///       }'
/// ```
pub async fn api_update(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<AuthUser>,
    Json(payload): Json<UpdatePayload>
) -> ApiResult<Json<UpdateResponse>> {
    debug!(
        "HANDLER ->> /api/account/update 'api_update' - User ID: {} Payload: {:?}",
        user.id, payload
    );

    // If password provided, hash it before update
    let hashed_password: Option<String> = if let Some(pw) = &payload.password {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Some(
            argon2
                .hash_password(pw.as_bytes(), &salt)
                .map_err(|e| AppError::from(e))?
                .to_string(),
        )
    } else {
        None
    };

    let account = sqlx::query_as!(
        UpdateResponse,
        r#"
        UPDATE accounts SET
            email = COALESCE($1, email),
            first_name = COALESCE($2, first_name),
            last_name = COALESCE($3, last_name),
            password = COALESCE($4, password),
            budget_preference = COALESCE($5, budget_preference),
            risk_preference = COALESCE($6, risk_preference),
            food_allergies = COALESCE($7, food_allergies),
            disabilities = COALESCE($8, disabilities)
        WHERE id = $9
        RETURNING
            id,
            email,
            first_name,
            last_name,
            budget_preference as "budget_preference: BudgetBucket",
            risk_preference as "risk_preference: RiskTolerence",
            food_allergies,
            disabilities
        "#,
        payload.email,
        payload.first_name,
        payload.last_name,
        hashed_password,
        payload.budget_preference as Option<BudgetBucket>,
        payload.risk_preference as Option<RiskTolerence>,
        payload.food_allergies,
        payload.disabilities,
        user.id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::from(e))?;

    Ok(Json(account))
}

/// Logout by setting cookie to expired.
///
/// # Method
/// `GET /api/account/logout`
///
/// # Responses
/// - `200 OK` - with body: [UpdateResponse]
/// - `401 UNAUTHORIZED` - Invalid credentials (public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X GET http://localhost:3001/api/account/logout
///   -H "Content-Type: application/json"
/// ```
async fn api_logout(
	cookies: Cookies,
    Extension(key): Extension<Key>,
    Extension(user): Extension<AuthUser>
) -> ApiResult<()> {
	debug!(
        "HANDLER ->> /api/account/logout 'api_logout' - User ID: {}",
        user.id
    );
	set_cookie(user.id, true, &cookies, &key);
	Ok(())
}

/// Create the account routes with authentication middleware.
///
/// # Routes
/// ## Protected Routes (require authentication)
/// - `POST /update` - Update user account information
/// - `GET /current` - Get current user's account details
/// - `POST /validate` - Validate authentication token
/// - `GET /logout` - Logout by making cookie expired
///
/// ## Public Routes (no authentication required)
/// - `POST /signup` - Create a new user account
/// - `POST /login` - Authenticate user and set auth cookie
///
/// # Middleware
/// Protected routes are secured by `middleware_auth` which validates the `auth-token` cookie.
/// Public routes (signup/login) are accessible without authentication.
///
pub fn account_routes() -> Router {
    Router::new()
        .route("/update", post(api_update))
        .route("/current", get(api_current))
        .route("/validate", get(api_validate))
        .route("/logout", get(api_logout))
        .route_layer(axum::middleware::from_fn(middleware_auth))
        .route("/signup", post(api_signup))
        .route("/login", post(api_login))
}