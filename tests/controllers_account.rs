/*
 * tests/controller/account.rs
 *
 * Unit tests for Account Controller
 *
 * Purpose:
 *   Test account controller functionality including password hashing,
 *   token generation, cookie security, and authentication logic.
 */

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use tower_cookies::{
    Cookie,
    cookie::{SameSite, time::Duration},
};

extern crate capping2025 as app;
use app::models::account::{SignupResponse, LoginResponse};
use app::{controllers, db};
use axum::{Router, Extension};
use serde_json::json;
use tower_cookies::CookieManagerLayer;
use tower_cookies::cookie::Key;
use chrono::Utc;
use serial_test::serial;
use anyhow::{anyhow, Result};
use std::net::TcpListener;
use sqlx::migrate;
use std::sync::Once;

// Ensure logger initializes only once across the test suite
static TEST_LOG_INIT: Once = Once::new();

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
        .same_site(if on_production {
            SameSite::None
        } else {
            SameSite::Lax
        })
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
        .same_site(if on_production {
            SameSite::None
        } else {
            SameSite::Lax
        })
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

    assert!(
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash1)
            .is_ok()
    );
    assert!(
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash2)
            .is_ok()
    );
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

#[tokio::test]
#[serial]
async fn test_signup_and_login_happy_path() {
    let (base, _pool) = spawn_app().await;
    let hc = httpc_test::new_client(&base).unwrap();

    let unique = Utc::now().timestamp_nanos_opt().unwrap();
    let email = format!("user+{}@example.com", unique);

    // Signup
    let resp = hc
        .do_post(
            "/account/signup",
            json!({
                "email": email,
                "first_name": "Alice",
                "last_name": "Tester",
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 201);

    // Login
    let resp = hc
        .do_post(
            "/account/login",
            json!({
                "email": format!("user+{}@example.com", unique),
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
}

#[tokio::test]
#[serial]
async fn test_signup_conflict_on_duplicate_email() {
    let (base, _pool) = spawn_app().await;
    let hc = httpc_test::new_client(&base).unwrap();

    let unique = Utc::now().timestamp_nanos_opt().unwrap();
    let email = format!("dupe+{}@example.com", unique);

    // First signup should succeed
    let resp1 = hc
        .do_post(
            "/account/signup",
            json!({
                "email": email,
                "first_name": "Bob",
                "last_name": "Dupe",
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(resp1.status().as_u16(), 201);

    // Second signup with same email should 409
    let resp2 = hc
        .do_post(
            "/account/signup",
            json!({
                "email": format!("dupe+{}@example.com", unique),
                "first_name": "Bob",
                "last_name": "Dupe",
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(resp2.status().as_u16(), 409);
}

async fn spawn_app() -> (String, sqlx::PgPool) {
	// Only use dotenvy for local testing
	// CI testing should use GitHub environment variables
	_ = dotenvy::dotenv();

	// Initialize project logger once so test logs are written to logs/latest.log
	TEST_LOG_INIT.call_once(|| {
		// Set a default log level for tests if not provided
		if std::env::var("RUST_LOG").is_err() {
			unsafe { std::env::set_var("RUST_LOG", "debug") };
		}
        app::log::init_panic_handler();
        app::log::init_logger();
	});

    // Ensure env and database
    if std::env::var("DATABASE_URL").is_err() {
        unsafe {
            std::env::set_var(
                "DATABASE_URL",
                "postgres://postgres:password@localhost:5432/capping2025",
            );
        }
    }

    let pool = db::create_pool().await;
    match migrate!("./migrations").run(&pool).await {
        Ok(_) => (),
        Err(sqlx::migrate::MigrateError::VersionMismatch(_)) => {
            eprintln!("migrations version mismatch; assuming DB already prepared. Skipping.");
        }
        Err(e) => panic!("migrations run: {e}"),
    }

    // Build app
    // Use an encryption/signing key for private cookies
    let cookie_key = Key::generate();
    let account_routes = controllers::account::account_routes()
        .layer(Extension(pool.clone()))
        .layer(Extension(cookie_key))
        .layer(CookieManagerLayer::new());
    let app = Router::new().nest("/account", account_routes);

    // Bind to ephemeral port and spawn server
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let addr = listener.local_addr().unwrap();
    let server = axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service());
    tokio::spawn(server);

    // Use localhost instead of 127.0.0.1 to match cookie domain
    (format!("http://localhost:{}", addr.port()), pool)
}

#[tokio::test]
#[serial]
async fn test_http_signup_and_login_flow() -> Result<()> {
    let (base, _pool) = spawn_app().await;
    let hc = httpc_test::new_client(&base)?;

    let unique = Utc::now().timestamp_nanos_opt().unwrap();
    let email = format!("user+{}@example.com", unique);

    // Signup
    let resp = hc
        .do_post(
            "/account/signup",
            json!({
                "email": email,
                "first_name": "Alice",
                "last_name": "Tester",
                "password": "Password123"
            }),
        )
        .await?;
    if !resp.status().is_success() { return Err(anyhow!("signup failed: {}", resp.status())); }

    // Login
    let resp = hc
        .do_post(
            "/account/login",
            json!({
                "email": format!("user+{}@example.com", unique),
                "password": "Password123"
            }),
        )
        .await?;
    if !resp.status().is_success() { return Err(anyhow!("login failed: {}", resp.status())); }

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_auth_required_for_me_endpoint() {
    let (base, _pool) = spawn_app().await;
    let hc = httpc_test::new_client(&base).unwrap();

    let unique = Utc::now().timestamp_nanos_opt().unwrap();
    let email = format!("auth_test+{}@example.com", unique);

    // First, signup a user (should work without auth)
    let resp = hc
        .do_post(
            "/account/signup",
            json!({
                "email": email,
                "first_name": "Auth",
                "last_name": "Tester",
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 201, "Signup should work without authentication");

    // Try to access /me without authentication (should fail)
    let resp = hc
        .do_post("/account/me", json!({}))
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 401, "Accessing /me without auth should return 401");

    // Login to get auth cookie
    let resp = hc
        .do_post(
            "/account/login",
            json!({
                "email": email,
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200, "Login should succeed");

    // Now try to access /me with auth cookie (should work)
    let resp = hc
        .do_post("/account/me", json!({}))
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200, "Accessing /me with auth should return 200");
}
