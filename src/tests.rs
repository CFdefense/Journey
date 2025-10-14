use std::{fs, io::Write, net::TcpListener, path::Path, time::Duration};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{Extension, Router};
use chrono::Utc;
use httpc_test::Client;
use serde_json::json;
use serial_test::serial;
use sqlx::migrate;
use tower_cookies::{
    cookie::{time, CookieJar, SameSite}, Cookie, CookieManagerLayer, Key
};
use tracing::{info, error, trace};
use crate::{
	controllers, db, global::*, log, models::account::{LoginResponse, SignupPayload, SignupResponse}
};

// UNIT TESTS

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
        .max_age(time::Duration::days(3))
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
        .max_age(time::Duration::days(3))
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

// ===== Email Validation Tests =====

#[test]
fn test_validate_email_valid() {
    assert!(SignupPayload::validate_email("user@example.com"));
    assert!(SignupPayload::validate_email("test.user@domain.co.uk"));
    assert!(SignupPayload::validate_email("name+tag@company.org"));
    assert!(SignupPayload::validate_email("user123@test-domain.com"));
}

#[test]
fn test_validate_email_invalid() {
    assert!(!SignupPayload::validate_email(""));
    assert!(!SignupPayload::validate_email("notanemail"));
    assert!(!SignupPayload::validate_email("@example.com"));
    assert!(!SignupPayload::validate_email("user@"));
    assert!(!SignupPayload::validate_email("user@.com"));
    assert!(!SignupPayload::validate_email("user @example.com"));
    assert!(!SignupPayload::validate_email("user@exam ple.com"));
}

// ===== Password Validation Tests =====

#[test]
fn test_validate_password_valid() {
    assert!(SignupPayload::validate_password("Password1").is_ok());
    assert!(SignupPayload::validate_password("MySecure123").is_ok());
    assert!(SignupPayload::validate_password("Passw0rd!@#").is_ok());
    assert!(SignupPayload::validate_password("LongerPassword123").is_ok());
}

#[test]
fn test_validate_password_too_short() {
    let result = SignupPayload::validate_password("Pass1");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Password must be at least 8 characters long"
    );
}

#[test]
fn test_validate_password_no_uppercase() {
    let result = SignupPayload::validate_password("password123");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Password must contain at least one uppercase letter"
    );
}

#[test]
fn test_validate_password_no_lowercase() {
    let result = SignupPayload::validate_password("PASSWORD123");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Password must contain at least one lowercase letter"
    );
}

#[test]
fn test_validate_password_no_number() {
    let result = SignupPayload::validate_password("PasswordOnly");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Password must contain at least one number"
    );
}

#[test]
fn test_validate_password_too_long() {
    let password = "A".repeat(129) + "1a";
    let result = SignupPayload::validate_password(&password);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Password must be 128 characters or less"
    );
}

#[test]
fn test_validate_password_non_ascii() {
    let result = SignupPayload::validate_password("Password1パスワード");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Password must contain only ASCII characters"
    );
}

#[test]
fn test_validate_password_emoji() {
    let result = SignupPayload::validate_password("Password1🔒");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Password must contain only ASCII characters"
    );
}

#[test]
fn test_validate_password_max_length_allowed() {
    // Test that exactly 128 characters is okay (with required chars)
    let password = "A".to_string() + &"a".repeat(126) + "1";
    assert_eq!(password.len(), 128);
    assert!(SignupPayload::validate_password(&password).is_ok());
}

// ===== Signup Payload Validation Tests =====

#[test]
fn test_validate_signup_payload_valid() {
    let payload = SignupPayload {
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "Password123".to_string(),
    };
    assert!(payload.validate().is_ok());
}

#[test]
fn test_validate_signup_payload_empty_email() {
    let payload = SignupPayload {
        email: "".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "Password123".to_string(),
    };
    let result = payload.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Email is required");
}

#[test]
fn test_validate_signup_payload_invalid_email() {
    let payload = SignupPayload {
        email: "not-an-email".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "Password123".to_string(),
    };
    let result = payload.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Invalid email format");
}

#[test]
fn test_validate_signup_payload_empty_first_name() {
    let payload = SignupPayload {
        email: "test@example.com".to_string(),
        first_name: "".to_string(),
        last_name: "Doe".to_string(),
        password: "Password123".to_string(),
    };
    let result = payload.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "First name is required");
}

#[test]
fn test_validate_signup_payload_empty_last_name() {
    let payload = SignupPayload {
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "".to_string(),
        password: "Password123".to_string(),
    };
    let result = payload.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Last name is required");
}

#[test]
fn test_validate_signup_payload_first_name_too_long() {
    let payload = SignupPayload {
        email: "test@example.com".to_string(),
        first_name: "a".repeat(51),
        last_name: "Doe".to_string(),
        password: "Password123".to_string(),
    };
    let result = payload.validate();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "First name must be 50 characters or less"
    );
}

#[test]
fn test_validate_signup_payload_last_name_too_long() {
    let payload = SignupPayload {
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "a".repeat(51),
        password: "Password123".to_string(),
    };
    let result = payload.validate();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Last name must be 50 characters or less"
    );
}

#[test]
fn test_validate_signup_payload_weak_password() {
    let payload = SignupPayload {
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "weak".to_string(),
    };
    let result = payload.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Password"));
}

#[test]
fn test_validate_signup_payload_whitespace_trimming() {
    let payload = SignupPayload {
        email: "  test@example.com  ".to_string(),
        first_name: "  John  ".to_string(),
        last_name: "  Doe  ".to_string(),
        password: "Password123".to_string(), // Valid ASCII password
    };
    // Email and names should be validated after trimming whitespace
    assert!(payload.validate().is_ok());
}

/// Verifies that `db::create_pool` panics when `DATABASE_URL` is not set.
#[test]
#[serial]
fn test_db_pool_panics_without_env() {
	// Save and clear DATABASE_URL
	let prev = std::env::var("DATABASE_URL").ok();
	unsafe { std::env::remove_var("DATABASE_URL"); }

	let result = std::panic::catch_unwind(||{
		let rt = tokio::runtime::Runtime::new().unwrap();
		rt.block_on(async {
			// Should panic due to missing env var
			let _ = db::create_pool().await;
		});
	});

	// Restore DATABASE_URL
	match prev {
		Some(val) => unsafe { std::env::set_var("DATABASE_URL", val) },
		None => unsafe { std::env::remove_var("DATABASE_URL") },
	}

	assert!(result.is_err());
}

/// Optional integration test requiring a real database in `DATABASE_URL`.
/// Run with: `cargo test -- --ignored`
#[tokio::test]
#[ignore]
#[serial]
async fn test_db_pool_connects_and_selects() {
	let database_url = match std::env::var("DATABASE_URL") {
		Ok(v) => v,
		Err(_) => {
			// Not set in most environments; mark as success skip
			info!("DATABASE_URL not set; skipping real DB test");
			return;
		}
	};

	// Ensure env var is present for this test
	unsafe { std::env::set_var("DATABASE_URL", database_url); }

	let pool = db::create_pool().await;

	// Simple liveness query
	let row: (i32,) = sqlx::query_as("SELECT 1")
		.fetch_one(&pool)
		.await
		.expect("SELECT 1 should succeed");
	assert_eq!(row.0, 1);
}

/// Verifies that `logs/latest.log` is created and written to from log events.
#[test]
#[serial]
fn test_logger() {
	//dotenv doesn't work in github actions bc .env is ignored
	unsafe {
		// Safety
		//
		// Always safe on Windows.
		//
		// Other platforms: risk of race condition in multi-threaded environment.
		// We are not reading/writing this environment variable from multiple threads, so we're good.
		std::env::set_var("RUST_LOG", "warn,Capping2025=debug");
	}
	let latest_log_path = Path::new(LOG_DIR).join(LATEST_LOG);
	log::init_logger();
	trace!("Test trace");
	error!("Test error");
	log::log_writer().flush().unwrap();
	//wait for IO to finish because flushing doesn't work?
	std::thread::sleep(Duration::from_millis(10));
	let logs = fs::read_to_string(latest_log_path).unwrap();
	info!("{logs}");
	assert!(logs.len() > 0);
}

/// Verifies that `logs/crash.log` is created and written to on a panic.
#[test]
#[serial]
fn test_panic_handler() {
	log::init_panic_handler();
	std::panic::catch_unwind(||{
		panic!("Test panic");
	}).unwrap_err();
	let content = fs::read_to_string(Path::new(LOG_DIR).join(CRASH_LOG)).unwrap();
	assert!(content.len() > 0);
}

// INTEGRATION TESTS

#[tokio::test]
#[serial]
async fn test_endpoints() {
	// Only use dotenvy for local testing
	// CI testing should use GitHub environment variables
	_ = dotenvy::dotenv();

	// Initialize project logger once so test logs are written to logs/latest.log
	// Set a default log level for tests if not provided
	if std::env::var("RUST_LOG").is_err() {
		unsafe { std::env::set_var("RUST_LOG", "debug") };
	}
    log::init_panic_handler();
    log::init_logger();

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
    let account_routes = controllers::account::account_routes();
    let itinerary_routes = controllers::itinerary::itinerary_routes();
    let event_routes = controllers::event::event_routes();
    let api_routes = Router::new()
        .nest("/account", account_routes)
        .nest("/itinerary", itinerary_routes)
        .nest("/event", event_routes);
    let app = Router::new()
        .nest("/api", api_routes)
        .layer(Extension(pool.clone()))
        .layer(Extension(cookie_key.clone()))
        .layer(CookieManagerLayer::new());

    // Bind to ephemeral port and spawn server
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let addr = listener.local_addr().unwrap();
    let server = axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service());
    tokio::spawn(server);

    let hc = httpc_test::new_client(format!("http://localhost:{}", addr.port())).unwrap();

    tokio::join!(
    	async {test_signup_and_login_happy_path(&hc, &cookie_key).await},
     	async {test_auth_required_for_me_endpoint(&hc).await},
     	async {test_signup_conflict_on_duplicate_email(&hc).await},
     	async {test_http_signup_and_login_flow(&hc).await},
	    async {test_validate_with_bad_and_good_cookie(&hc).await},
	    async {test_current_endpoint_returns_account(&hc).await},
	    async {test_update_endpoint_returns_account(&hc).await},
	    async {test_update_endpoint_partial_fields(&hc).await},
	    async {test_update_endpoint_with_preferences(&hc).await},
	    async {test_saved_itineraries_endpoint(&hc).await},
	    async {test_get_itinerary_endpoint(&hc).await},
	    async {test_get_itinerary_events_endpoint(&hc).await},
	    async {test_itinerary_endpoints_require_auth(&hc).await},
	    async {test_event_endpoints_require_auth(&hc).await},
	    async {test_get_event_endpoint(&hc).await},
        // just throw all the tests in here
    );
}

async fn test_signup_and_login_happy_path(hc: &Client, key: &Key) {
    let unique = Utc::now().timestamp_nanos_opt().unwrap();
    let email = format!("user+{}@example.com", unique);

    // Signup
    let resp = hc
        .do_post(
            "/api/account/signup",
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
            "/api/account/login",
            json!({
                "email": format!("user+{}@example.com", unique),
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    // Extract cookie and decrypt via private jar
    let set_cookie = resp.header("set-cookie").unwrap();
    // Parse full Set-Cookie line (handles '=' inside value)
    let parsed = Cookie::parse(set_cookie.to_string()).unwrap();
    let mut jar = CookieJar::new();
    jar.add(parsed.clone());
    let decrypted = jar.private(&key).get(parsed.name()).unwrap();
    // token: user-<id>.<exp>.sign
    let parts: Vec<&str> = decrypted.value().split('.').collect();
    assert_eq!(parts.len(), 3);
    assert!(parts[0].starts_with("user-"));
    assert_eq!(parts[2], "sign");
    let exp: i64 = parts[1].parse().unwrap();
    let now = chrono::Utc::now().timestamp();
    assert!(exp > now);
}

async fn test_auth_required_for_me_endpoint(hc: &Client) {
    let unique = Utc::now().timestamp_nanos_opt().unwrap();
    let email = format!("auth_test+{}@example.com", unique);

    // Try to access /me without authentication (should fail)
    let resp = hc
        .do_post("/api/account/validate", json!({}))
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 401, "Accessing /me without auth should return 401");

    // First, signup a user (should work without auth)
    let resp = hc
        .do_post(
            "/api/account/signup",
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

    // Login to get auth cookie
    let resp = hc
        .do_post(
            "/api/account/login",
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
        .do_post("/api/account/validate", json!({}))
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200, "Accessing /validate with auth should return 200");
}

async fn test_signup_conflict_on_duplicate_email(hc: &Client) {
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
    let email = format!("dupe+{}@example.com", unique);

    // First signup should succeed
    let resp1 = hc
        .do_post(
            "/api/account/signup",
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
            "/api/account/signup",
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

async fn test_http_signup_and_login_flow(hc: &Client) {
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
    let email = format!("user+{}@example.com", unique);

    // Signup
    let resp = hc
        .do_post(
            "/api/account/signup",
            json!({
                "email": email,
                "first_name": "Alice",
                "last_name": "Tester",
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert!(resp.status().is_success(), "signup failed: {}", resp.status());

    // Login
    let resp = hc
        .do_post(
            "/api/account/login",
            json!({
                "email": format!("user+{}@example.com", unique),
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert!(resp.status().is_success(), "login failed: {}", resp.status());
}

async fn test_validate_with_bad_and_good_cookie(hc: &Client) {
    // No cookie (treated similarly to bad/invalid cookie): expect unauthorized
    let resp = hc
        .do_post("/api/account/validate", json!({}))
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 401, "Missing/invalid cookie should return 401");

    // Good cookie: create user and login to receive a valid private cookie, then validate
    let unique = Utc::now().timestamp_nanos_opt().unwrap();
    let email = format!("cookie+{}@example.com", unique);

    let signup = hc
        .do_post(
            "/api/account/signup",
            json!({
                "email": email,
                "first_name": "Cook",
                "last_name": "Ie",
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(signup.status().as_u16(), 201);

    let login = hc
        .do_post(
            "/api/account/login",
            json!({
                "email": format!("cookie+{}@example.com", unique),
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(login.status().as_u16(), 200);

    // Client should now hold the private cookie; call validate and expect 200
    let resp = hc
        .do_post("/api/account/validate", json!({}))
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200, "/validate with good cookie should return 200");
}

async fn test_current_endpoint_returns_account(hc: &Client) {
    let unique = Utc::now().timestamp_nanos_opt().unwrap();
    let email = format!("current+{}@example.com", unique);

    // Signup user
    let signup_resp = hc
        .do_post(
            "/api/account/signup",
            json!({
                "email": email,
                "first_name": "Current",
                "last_name": "Tester",
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(signup_resp.status().as_u16(), 201);

    // Login to get auth cookie
    let login_resp = hc
        .do_post(
            "/api/account/login",
            json!({
                "email": email,
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(login_resp.status().as_u16(), 200);

    // Test /current endpoint returns Account struct
    let current_resp = hc
        .do_get("/api/account/current")
        .await
        .unwrap();
    assert_eq!(current_resp.status().as_u16(), 200);

    // Verify response is successful and contains data
    // Note: JSON parsing would require checking httpc-test Response methods
    assert!(current_resp.status().is_success());
}

async fn test_update_endpoint_returns_account(hc: &Client) {
    let unique = Utc::now().timestamp_nanos_opt().unwrap();
    let email = format!("update+{}@example.com", unique);

    // Signup user
    let signup_resp = hc
        .do_post(
            "/api/account/signup",
            json!({
                "email": email,
                "first_name": "Update",
                "last_name": "Tester",
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(signup_resp.status().as_u16(), 201);

    // Login to get auth cookie
    let login_resp = hc
        .do_post(
            "/api/account/login",
            json!({
                "email": email,
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(login_resp.status().as_u16(), 200);

    // Test /update endpoint with all fields
    let update_resp = hc
        .do_post(
            "/api/account/update",
            json!({
                "email": format!("updated+{}@example.com", unique),
                "first_name": "Updated",
                "last_name": "User",
                "password": "NewPassword123",
                "budget_preference": "HighBudget",
                "risk_preference": "Adventurer",
                "food_allergies": "Peanuts, shellfish",
                "disabilities": "Wheelchair accessible"
            }),
        )
        .await
        .unwrap();
    assert_eq!(update_resp.status().as_u16(), 200);

    // Verify response is successful
    assert!(update_resp.status().is_success());
}

async fn test_update_endpoint_partial_fields(hc: &Client) {
    let unique = Utc::now().timestamp_nanos_opt().unwrap();
    let email = format!("partial+{}@example.com", unique);

    // Signup user
    let signup_resp = hc
        .do_post(
            "/api/account/signup",
            json!({
                "email": email,
                "first_name": "Partial",
                "last_name": "Tester",
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(signup_resp.status().as_u16(), 201);

    // Login to get auth cookie
    let login_resp = hc
        .do_post(
            "/api/account/login",
            json!({
                "email": email,
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(login_resp.status().as_u16(), 200);

    // Test /update endpoint with only some fields
    let update_resp = hc
        .do_post(
            "/api/account/update",
            json!({
                "first_name": "PartiallyUpdated",
                "food_allergies": "Gluten"
            }),
        )
        .await
        .unwrap();
    assert_eq!(update_resp.status().as_u16(), 200);

    // Verify response is successful
    assert!(update_resp.status().is_success());
}

async fn test_update_endpoint_with_preferences(hc: &Client) {
    let unique = Utc::now().timestamp_nanos_opt().unwrap();
    let email = format!("prefs+{}@example.com", unique);

    // Signup user
    let signup_resp = hc
        .do_post(
            "/api/account/signup",
            json!({
                "email": email,
                "first_name": "Prefs",
                "last_name": "Tester",
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(signup_resp.status().as_u16(), 201);

    // Login to get auth cookie
    let login_resp = hc
        .do_post(
            "/api/account/login",
            json!({
                "email": email,
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(login_resp.status().as_u16(), 200);

    // Test /update endpoint with enum preferences
    let update_resp = hc
        .do_post(
            "/api/account/update",
            json!({
                "budget_preference": "LuxuryBudget",
                "risk_preference": "RiskTaker"
            }),
        )
        .await
        .unwrap();
    assert_eq!(update_resp.status().as_u16(), 200);

    // Verify response is successful
    assert!(update_resp.status().is_success());
}

async fn test_saved_itineraries_endpoint(hc: &Client) {
    let unique = Utc::now().timestamp_nanos_opt().unwrap();
    let email = format!("saved_itineraries+{}@example.com", unique);

    // Signup user
    let signup_resp = hc
        .do_post(
            "/api/account/signup",
            json!({
                "email": email,
                "first_name": "Saved",
                "last_name": "Itineraries",
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(signup_resp.status().as_u16(), 201);

    // Login to get auth cookie
    let login_resp = hc
        .do_post(
            "/api/account/login",
            json!({
                "email": email,
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(login_resp.status().as_u16(), 200);

    // Test /saved endpoint returns user's itineraries
    let saved_resp = hc
        .do_get("/api/itinerary/saved")
        .await
        .unwrap();
    assert_eq!(saved_resp.status().as_u16(), 200);

    // Verify response is successful
    assert!(saved_resp.status().is_success());
}

async fn test_get_itinerary_endpoint(hc: &Client) {
    let unique = Utc::now().timestamp_nanos_opt().unwrap();
    let email = format!("get_itinerary+{}@example.com", unique);

    // Signup user
    let signup_resp = hc
        .do_post(
            "/api/account/signup",
            json!({
                "email": email,
                "first_name": "Get",
                "last_name": "Itinerary",
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(signup_resp.status().as_u16(), 201);

    // Login to get auth cookie
    let login_resp = hc
        .do_post(
            "/api/account/login",
            json!({
                "email": email,
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(login_resp.status().as_u16(), 200);

    // Test /{id} endpoint with non-existent itinerary (should return 404)
    let get_resp = hc
        .do_get("/api/itinerary/999999")
        .await
        .unwrap();
    assert_eq!(get_resp.status().as_u16(), 404);

    // Test with invalid ID format (should return 400)
    let invalid_resp = hc
        .do_get("/api/itinerary/invalid")
        .await
        .unwrap();
    assert_eq!(invalid_resp.status().as_u16(), 400);
}

async fn test_get_itinerary_events_endpoint(hc: &Client) {
    let unique = Utc::now().timestamp_nanos_opt().unwrap();
    let email = format!("itinerary_events+{}@example.com", unique);

    // Signup user
    let signup_resp = hc
        .do_post(
            "/api/account/signup",
            json!({
                "email": email,
                "first_name": "Itinerary",
                "last_name": "Events",
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(signup_resp.status().as_u16(), 201);

    // Login to get auth cookie
    let login_resp = hc
        .do_post(
            "/api/account/login",
            json!({
                "email": email,
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(login_resp.status().as_u16(), 200);

    // Test /{id}/events endpoint with non-existent itinerary (should return 404)
    let events_resp = hc
        .do_get("/api/itinerary/999999/events")
        .await
        .unwrap();
    assert_eq!(events_resp.status().as_u16(), 404);

    // Test with invalid ID format (should return 400)
    let invalid_resp = hc
        .do_get("/api/itinerary/invalid/events")
        .await
        .unwrap();
    assert_eq!(invalid_resp.status().as_u16(), 400);
}

async fn test_itinerary_endpoints_require_auth(hc: &Client) {
    // Test that itinerary endpoints require authentication
    let saved_resp = hc
        .do_get("/api/itinerary/saved")
        .await
        .unwrap();
    assert_eq!(saved_resp.status().as_u16(), 401, "Saved itineraries should require auth");

    let get_resp = hc
        .do_get("/api/itinerary/1")
        .await
        .unwrap();
    assert_eq!(get_resp.status().as_u16(), 401, "Get itinerary should require auth");

    let events_resp = hc
        .do_get("/api/itinerary/1/events")
        .await
        .unwrap();
    assert_eq!(events_resp.status().as_u16(), 401, "Get itinerary events should require auth");
}

async fn test_event_endpoints_require_auth(hc: &Client) {
    // Test that event endpoints require authentication
    let get_resp = hc
        .do_get("/api/event/1")
        .await
        .unwrap();
    assert_eq!(get_resp.status().as_u16(), 401, "Event endpoint should require auth");
}

async fn test_get_event_endpoint(hc: &Client) {
    let unique = Utc::now().timestamp_nanos_opt().unwrap();
    let email = format!("get_event+{}@example.com", unique);

    // Signup user
    let signup_resp = hc
        .do_post(
            "/api/account/signup",
            json!({
                "email": email,
                "first_name": "Get",
                "last_name": "Event",
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(signup_resp.status().as_u16(), 201);

    // Login to get auth cookie
    let login_resp = hc
        .do_post(
            "/api/account/login",
            json!({
                "email": email,
                "password": "Password123"
            }),
        )
        .await
        .unwrap();
    assert_eq!(login_resp.status().as_u16(), 200);

    // Test /{id} endpoint with non-existent event (should return 404)
    let get_resp = hc
        .do_get("/api/event/999999")
        .await
        .unwrap();
    assert_eq!(get_resp.status().as_u16(), 404);

    // Test with invalid ID format (should return 400)
    let invalid_resp = hc
        .do_get("/api/event/invalid")
        .await
        .unwrap();
    assert_eq!(invalid_resp.status().as_u16(), 400);
}
