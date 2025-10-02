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
    routing::{get, post},
};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use tower_cookies::{Cookie, Cookies, cookie::{SameSite, time::Duration}};

use serde_json::{Value, json};
use sqlx::PgPool;
use tracing::{error, info};

use crate::models::account::*;
use crate::error::ApiResult;

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
    let existing_user_result = sqlx::query!(
        "SELECT id FROM accounts WHERE email = $1",
        payload.email
    )
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
            error!("ERROR ->> /api/signup 'api_signup' REASON: Failed to hash password: {:?}", e);
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
            let token_value = format!("user-{}.exp.sign", result.id);

            info!("INFO ->> /api/login 'api_login' - Generated token value: {}. Production is: {}", token_value, on_production);

            // Build the cookie with enhanced security
            let cookie = Cookie::build("auth-token", token_value.clone())
                .domain(domain.to_string())
                .path("/")
                .secure(on_production)
                .http_only(true)
                .same_site(if on_production { SameSite::None } else { SameSite::Lax })
                .max_age(Duration::days(3))
                .finish();

            // Add the cookie
            cookies.add(cookie);

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

pub async fn api_test() -> Json<Value> {
    Json(json!({
        "message": "test endpoint"
    }))
}

pub fn account_routes() -> Router {
    Router::new()
        .route("/signup", post(api_signup))
        .route("/login", post(api_login))
        .route("/test", get(api_test))
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    /// Test password verification logic
    #[test]
    fn test_password_verification() {
        let password = "test_password123";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        // Hash the password
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        // Verify correct password
        let parsed_hash = PasswordHash::new(&password_hash).unwrap();
        assert!(
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok()
        );

        // Verify incorrect password fails
        assert!(
            Argon2::default()
                .verify_password(b"wrong_password", &parsed_hash)
                .is_err()
        );
    }

    /// Test token generation format
    #[test]
    fn test_token_generation() {
        let user_id = 42;
        let token = format!("user-{}.exp.sign", user_id);
        assert_eq!(token, "user-42.exp.sign");
        assert!(token.starts_with("user-"));
        assert!(token.ends_with(".exp.sign"));
    }

    /// Test cookie security settings
    #[test]
    fn test_cookie_security_development() {
        let token_value = "test-token-123";
        let on_production = false;
        let domain = "localhost";

        let cookie = Cookie::build("auth-token", token_value)
            .domain(domain.to_string())
            .path("/")
            .secure(on_production)
            .http_only(true)
            .same_site(if on_production { SameSite::None } else { SameSite::Lax })
            .max_age(Duration::days(3))
            .finish();

        assert_eq!(cookie.name(), "auth-token");
        assert_eq!(cookie.value(), token_value);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
        assert!(!cookie.secure().unwrap_or(false));
    }

    /// Test cookie security settings for production
    #[test]
    fn test_cookie_security_production() {
        let token_value = "test-token-456";
        let on_production = true;
        let domain = "example.com";

        let cookie = Cookie::build("auth-token", token_value)
            .domain(domain.to_string())
            .path("/")
            .secure(on_production)
            .http_only(true)
            .same_site(if on_production { SameSite::None } else { SameSite::Lax })
            .max_age(Duration::days(3))
            .finish();

        assert_eq!(cookie.name(), "auth-token");
        assert_eq!(cookie.value(), token_value);
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::None));
        assert!(cookie.secure().unwrap_or(false));
    }

    /// Test password hashing for signup
    #[test]
    fn test_signup_password_hashing() {
        let password = "secure_password_123";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        // Hash the password (as done in signup)
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        // Verify the hash is not the same as the plain password
        assert_ne!(password_hash, password);

        // Verify the hash starts with expected format
        assert!(password_hash.starts_with("$argon2"));

        // Verify we can verify the password later (as in login)
        let parsed_hash = PasswordHash::new(&password_hash).unwrap();
        assert!(
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok()
        );
    }

    /// Test that different salts produce different hashes
    #[test]
    fn test_signup_different_salts() {
        let password = "same_password";
        let argon2 = Argon2::default();

        // Generate two hashes with different salts
        let salt1 = SaltString::generate(&mut OsRng);
        let hash1 = argon2
            .hash_password(password.as_bytes(), &salt1)
            .unwrap()
            .to_string();

        let salt2 = SaltString::generate(&mut OsRng);
        let hash2 = argon2
            .hash_password(password.as_bytes(), &salt2)
            .unwrap()
            .to_string();

        // Hashes should be different due to different salts
        assert_ne!(hash1, hash2);

        // But both should verify the same password
        let parsed_hash1 = PasswordHash::new(&hash1).unwrap();
        let parsed_hash2 = PasswordHash::new(&hash2).unwrap();
        
        assert!(Argon2::default().verify_password(password.as_bytes(), &parsed_hash1).is_ok());
        assert!(Argon2::default().verify_password(password.as_bytes(), &parsed_hash2).is_ok());
    }

    /// Test that password hash cannot be reversed to plain text
    #[test]
    fn test_signup_hash_irreversibility() {
        let password = "my_secret_password_456";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        // Hash should not contain the plain password
        assert!(!password_hash.contains(password));
        
        // Hash should be significantly longer than input
        assert!(password_hash.len() > password.len() * 2);
    }

    /// Test signup response structure
    #[test]
    fn test_signup_response_structure() {
        use crate::models::account::SignupResponse;
        
        let response = SignupResponse {
            id: 123,
            email: "test@example.com".to_string(),
        };

        assert_eq!(response.id, 123);
        assert_eq!(response.email, "test@example.com");
    }

    /// Test login response structure
    #[test]
    fn test_login_response_structure() {
        use crate::models::account::LoginResponse;
        
        let response = LoginResponse {
            id: 456,
            token: "user-456.exp.sign".to_string(),
        };

        assert_eq!(response.id, 456);
        assert_eq!(response.token, "user-456.exp.sign");
    }

    /// Test that any password can be hashed (even if it wouldn't pass validation)
    #[test]
    fn test_password_hashing_mechanism() {
        // Test that the hashing algorithm works with any input
        let test_password = "abc";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        // Hashing mechanism should work regardless of validation rules
        let result = argon2.hash_password(test_password.as_bytes(), &salt);
        assert!(result.is_ok());

        let password_hash = result.unwrap().to_string();
        
        // Verify the hash works
        let parsed_hash = PasswordHash::new(&password_hash).unwrap();
        assert!(
            Argon2::default()
                .verify_password(test_password.as_bytes(), &parsed_hash)
                .is_ok()
        );
        
        // But wrong password should fail
        assert!(
            Argon2::default()
                .verify_password(b"wrong", &parsed_hash)
                .is_err()
        );
    }

    /// Test password with special ASCII characters
    #[test]
    fn test_signup_special_characters_password() {
        // Only ASCII special characters are allowed
        let special_password = "Passw0rd!@#$%";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(special_password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        // Verify special characters are handled correctly
        let parsed_hash = PasswordHash::new(&password_hash).unwrap();
        assert!(
            Argon2::default()
                .verify_password(special_password.as_bytes(), &parsed_hash)
                .is_ok()
        );

        // Wrong password should fail
        assert!(
            Argon2::default()
                .verify_password(b"Passw0rd!@#$", &parsed_hash)
                .is_err()
        );
    }

    /// Test maximum allowed password length (128 chars)
    #[test]
    fn test_signup_max_password_length() {
        // 128 characters with required complexity
        let max_password = "A".to_string() + &"a".repeat(126) + "1";
        assert_eq!(max_password.len(), 128);
        
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(max_password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        // Verify max length password works
        let parsed_hash = PasswordHash::new(&password_hash).unwrap();
        assert!(
            Argon2::default()
                .verify_password(max_password.as_bytes(), &parsed_hash)
                .is_ok()
        );
    }

}
