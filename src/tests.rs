use crate::{
	agent, controllers, db,
	global::*,
	http_models::{
		account::{LoginRequest, SignupRequest, UpdateRequest},
		chat_session::RenameRequest,
		event::{SearchEventRequest, UserEventRequest, UserEventResponse},
		itinerary::{Itinerary, UnsaveRequest},
		message::{MessagePageRequest, SendMessageRequest, UpdateMessageRequest},
	},
	log,
	middleware::AuthUser,
	sql_models::{BudgetBucket, RiskTolerence},
};
use argon2::{
	Argon2,
	password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{Extension, Json, Router};
use chrono::{NaiveDate, NaiveDateTime, Utc};
use serde_json::json;
use serial_test::serial;
use sqlx::{PgPool, migrate};
use std::sync::Arc;
use std::{
	fs,
	io::Write,
	path::Path,
	time::{Duration, SystemTime},
};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_cookies::{
	Cookie, CookieManagerLayer, Key,
	cookie::{CookieJar, SameSite, time},
};
use tracing::{error, info, trace};

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

	let cookie = Cookie::build(("auth-token", token_value))
		.domain(domain.to_string())
		.path("/")
		.secure(on_production)
		.http_only(true)
		.same_site(if on_production {
			SameSite::Strict
		} else {
			SameSite::Lax
		})
		.max_age(time::Duration::days(3))
		.build();

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

	let cookie = Cookie::build(("auth-token", token_value))
		.domain(domain.to_string())
		.path("/")
		.secure(on_production)
		.http_only(true)
		.same_site(if on_production {
			SameSite::Strict
		} else {
			SameSite::Lax
		})
		.max_age(time::Duration::days(3))
		.build();

	assert_eq!(cookie.name(), "auth-token");
	assert_eq!(cookie.value(), token_value);
	assert_eq!(cookie.http_only(), Some(true));
	assert_eq!(cookie.same_site(), Some(SameSite::Strict));
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

#[test]
fn test_validate_email() {
	// valid
	assert!(SignupRequest::validate_email("user@example.com"));
	assert!(SignupRequest::validate_email("test.user@domain.co.uk"));
	assert!(SignupRequest::validate_email("name+tag@company.org"));
	assert!(SignupRequest::validate_email("user123@test-domain.com"));

	// invalid
	assert!(!SignupRequest::validate_email(""));
	assert!(!SignupRequest::validate_email("notanemail"));
	assert!(!SignupRequest::validate_email("@example.com"));
	assert!(!SignupRequest::validate_email("user@"));
	assert!(!SignupRequest::validate_email("user@.com"));
	assert!(!SignupRequest::validate_email("user @example.com"));
	assert!(!SignupRequest::validate_email("user@exam ple.com"));
}

#[test]
fn test_validate_password() {
	// valid
	assert!(SignupRequest::validate_password("Password1").is_ok());
	assert!(SignupRequest::validate_password("MySecure123").is_ok());
	assert!(SignupRequest::validate_password("Passw0rd!@#").is_ok());
	assert!(SignupRequest::validate_password("LongerPassword123").is_ok());

	// too short
	let result = SignupRequest::validate_password("Pass1");
	assert!(result.is_err());
	assert_eq!(
		result.unwrap_err(),
		"Password must be at least 8 characters long"
	);

	// no uppercase
	let result = SignupRequest::validate_password("password123");
	assert!(result.is_err());
	assert_eq!(
		result.unwrap_err(),
		"Password must contain at least one uppercase letter"
	);

	// no lowercase
	let result = SignupRequest::validate_password("PASSWORD123");
	assert!(result.is_err());
	assert_eq!(
		result.unwrap_err(),
		"Password must contain at least one lowercase letter"
	);

	// no number
	let result = SignupRequest::validate_password("PasswordOnly");
	assert!(result.is_err());
	assert_eq!(
		result.unwrap_err(),
		"Password must contain at least one number"
	);

	// too long
	let password = "A".repeat(129) + "1a";
	let result = SignupRequest::validate_password(&password);
	assert!(result.is_err());
	assert_eq!(
		result.unwrap_err(),
		"Password must be 128 characters or less"
	);

	// non ascii
	let result = SignupRequest::validate_password("Password1パスワード");
	assert!(result.is_err());
	assert_eq!(
		result.unwrap_err(),
		"Password must contain only ASCII characters"
	);

	// emoji
	let result = SignupRequest::validate_password("Password1🔒");
	assert!(result.is_err());
	assert_eq!(
		result.unwrap_err(),
		"Password must contain only ASCII characters"
	);

	// max length allowed
	// Test that exactly 128 characters is okay (with required chars)
	let password = "A".to_string() + &"a".repeat(126) + "1";
	assert_eq!(password.len(), 128);
	assert!(SignupRequest::validate_password(&password).is_ok());
}

#[test]
fn test_validate_signup_payload() {
	// valid
	let payload = SignupRequest {
		email: "test@example.com".to_string(),
		first_name: "John".to_string(),
		last_name: "Doe".to_string(),
		password: "Password123".to_string(),
	};
	assert!(payload.validate().is_ok());

	// empty email
	let payload = SignupRequest {
		email: "".to_string(),
		first_name: "John".to_string(),
		last_name: "Doe".to_string(),
		password: "Password123".to_string(),
	};
	let result = payload.validate();
	assert!(result.is_err());
	assert_eq!(result.unwrap_err(), "Email is required");

	// invalid email
	let payload = SignupRequest {
		email: "not-an-email".to_string(),
		first_name: "John".to_string(),
		last_name: "Doe".to_string(),
		password: "Password123".to_string(),
	};
	let result = payload.validate();
	assert!(result.is_err());
	assert_eq!(result.unwrap_err(), "Invalid email format");

	// empty first name
	let payload = SignupRequest {
		email: "test@example.com".to_string(),
		first_name: "".to_string(),
		last_name: "Doe".to_string(),
		password: "Password123".to_string(),
	};
	let result = payload.validate();
	assert!(result.is_err());
	assert_eq!(result.unwrap_err(), "First name is required");

	// empty last name
	let payload = SignupRequest {
		email: "test@example.com".to_string(),
		first_name: "John".to_string(),
		last_name: "".to_string(),
		password: "Password123".to_string(),
	};
	let result = payload.validate();
	assert!(result.is_err());
	assert_eq!(result.unwrap_err(), "Last name is required");

	// first name too long
	let payload = SignupRequest {
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

	// last name too long
	let payload = SignupRequest {
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

	// weak password
	let payload = SignupRequest {
		email: "test@example.com".to_string(),
		first_name: "John".to_string(),
		last_name: "Doe".to_string(),
		password: "weak".to_string(),
	};
	let result = payload.validate();
	assert!(result.is_err());
	assert!(result.unwrap_err().contains("Password"));

	// whitespace trimming
	let payload = SignupRequest {
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
#[serial(db)]
fn test_db_pool_panics_without_env() {
	// Save and clear DATABASE_URL
	let prev = std::env::var("DATABASE_URL").ok();
	unsafe {
		std::env::remove_var("DATABASE_URL");
	}

	let result = std::panic::catch_unwind(|| {
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
#[serial(db)]
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
	unsafe {
		std::env::set_var("DATABASE_URL", database_url);
	}

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
#[serial(log)]
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
#[serial(panic_log)]
fn test_panic_handler() {
	log::init_panic_handler();
	std::panic::catch_unwind(|| {
		panic!("Test panic");
	})
	.unwrap_err();
	let content = fs::read_to_string(Path::new(LOG_DIR).join(CRASH_LOG)).unwrap();
	assert!(content.len() > 0);
}

/// It's easier to have all these in 1 test to share a db pool, and we don't have to spin up a server
#[tokio::test]
#[serial(db)]
async fn test_controllers() {
	_ = dotenvy::dotenv();
	let cookies = CookieJar::new();
	let key = Extension(Key::derive_from(&[0u8; 32]));
	let pool = Extension(db::create_pool().await);

	_ = tokio::join!(
		test_signup_conflict_on_duplicate_email(cookies.clone(), key.clone(), pool.clone()),
		test_http_login_invalid_credentials(cookies.clone(), key.clone(), pool.clone()),
		test_current_endpoint_returns_account(cookies.clone(), key.clone(), pool.clone()),
		test_update_endpoint_returns_account(cookies.clone(), key.clone(), pool.clone()),
		test_update_endpoint_partial_fields(cookies.clone(), key.clone(), pool.clone()),
		test_update_endpoint_with_preferences(cookies.clone(), key.clone(), pool.clone()),
		test_get_itinerary_id_not_found(cookies.clone(), key.clone(), pool.clone()),
		test_invalid_signup_email(cookies.clone(), key.clone(), pool.clone()),
		test_saved_itineraries_endpoint(cookies.clone(), key.clone(), pool.clone()),
		test_save_itineraries(cookies.clone(), key.clone(), pool.clone()),
		test_chat_flow(cookies.clone(), key.clone(), pool.clone()),
		test_user_event_flow(cookies.clone(), key.clone(), pool.clone()),
		test_unsave_itinerary_success(cookies.clone(), key.clone(), pool.clone()),
		test_unsave_itinerary_not_found(cookies.clone(), key.clone(), pool.clone()),
		test_unsave_already_unsaved_itinerary(cookies.clone(), key.clone(), pool.clone()),
	);
}

async fn test_signup_conflict_on_duplicate_email(
	mut cookies: CookieJar,
	key: Extension<Key>,
	pool: Extension<PgPool>,
) {
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
	let email = format!("dupe+{}@example.com", unique);
	let json = Json(SignupRequest {
		email,
		first_name: String::from("Bob"),
		last_name: String::from("Dupe"),
		password: String::from("Password123"),
	});
	// First signup should succeed
	controllers::account::api_signup(&mut cookies, key.clone(), pool.clone(), json.clone())
		.await
		.unwrap();
	// Second signup with same email should 409
	assert_eq!(
		controllers::account::api_signup(&mut cookies, key, pool, json)
			.await
			.unwrap_err()
			.status_code()
			.as_u16(),
		409
	);
}

async fn test_http_login_invalid_credentials(
	mut cookies: CookieJar,
	key: Extension<Key>,
	pool: Extension<PgPool>,
) {
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
	let email = format!("badEmail+{}@example.com", unique);
	let json = Json(LoginRequest {
		email,
		password: String::from("Password123"),
	});
	// attempt to login with nonexistant email
	assert_eq!(
		controllers::account::api_login(&mut cookies, key.clone(), pool.clone(), json)
			.await
			.unwrap_err()
			.status_code()
			.as_u16(),
		400
	);

	let email = format!("goodEmail+{}@example.com", unique);
	let json = Json(SignupRequest {
		email: email.clone(),
		first_name: String::from("Alice"),
		last_name: String::from("Tester"),
		password: String::from("Password123"),
	});
	// signup
	controllers::account::api_signup(&mut cookies, key.clone(), pool.clone(), json)
		.await
		.unwrap();

	let json = Json(LoginRequest {
		email,
		password: String::from("ChickenNugget1234"),
	});
	// attempt to login with a correct email, but the wrong password
	assert_eq!(
		controllers::account::api_login(&mut cookies, key, pool, json)
			.await
			.unwrap_err()
			.status_code()
			.as_u16(),
		400
	);
}

async fn test_current_endpoint_returns_account(
	mut cookies: CookieJar,
	key: Extension<Key>,
	pool: Extension<PgPool>,
) {
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
	let email = format!("current+{}@example.com", unique);
	let json = Json(SignupRequest {
		email,
		first_name: String::from("Current"),
		last_name: String::from("Tester"),
		password: String::from("Password123"),
	});
	// Signup user
	controllers::account::api_signup(&mut cookies, key.clone(), pool.clone(), json)
		.await
		.unwrap();

	let cookie = cookies.get("auth-token").unwrap();
	let parts: Vec<&str> = cookie.value().split(&['-', '.']).collect();
	let user = Extension(AuthUser {
		id: parts[1].parse().unwrap(),
	});
	// Test /current endpoint returns Account struct
	_ = controllers::account::api_current(pool.clone(), user)
		.await
		.unwrap();
}

async fn test_update_endpoint_returns_account(
	mut cookies: CookieJar,
	key: Extension<Key>,
	pool: Extension<PgPool>,
) {
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
	let email = format!("update+{}@example.com", unique);
	let json = Json(SignupRequest {
		email,
		first_name: String::from("Update"),
		last_name: String::from("Tester"),
		password: String::from("Password123"),
	});
	// Signup user
	controllers::account::api_signup(&mut cookies, key.clone(), pool.clone(), json)
		.await
		.unwrap();

	// Test /update endpoint with all fields
	let cookie = cookies.get("auth-token").unwrap();
	let parts: Vec<&str> = cookie.value().split(&['-', '.']).collect();
	let user = Extension(AuthUser {
		id: parts[1].parse().unwrap(),
	});
	let json = Json(UpdateRequest {
		email: Some(format!("updated+{}@example.com", unique)),
		first_name: Some(String::from("Updated")),
		last_name: Some(String::from("User")),
		password: Some(String::from("NewPassword123")),
		current_password: Some(String::from("Password123")),
		budget_preference: Some(BudgetBucket::HighBudget),
		risk_preference: Some(RiskTolerence::Adventurer),
		food_allergies: Some(String::from("Peanuts, shellfish")),
		disabilities: Some(String::from("Wheelchair accessible")),
	});
	_ = controllers::account::api_update(pool, user, json)
		.await
		.unwrap();
}

async fn test_update_endpoint_partial_fields(
	mut cookies: CookieJar,
	key: Extension<Key>,
	pool: Extension<PgPool>,
) {
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
	let email = format!("partial+{}@example.com", unique);
	let json = Json(SignupRequest {
		email,
		first_name: String::from("Partial"),
		last_name: String::from("Tester"),
		password: String::from("Password123"),
	});
	// Signup user
	controllers::account::api_signup(&mut cookies, key.clone(), pool.clone(), json)
		.await
		.unwrap();

	// Test /update endpoint with only some fields
	let cookie = cookies.get("auth-token").unwrap();
	let parts: Vec<&str> = cookie.value().split(&['-', '.']).collect();
	let user = Extension(AuthUser {
		id: parts[1].parse().unwrap(),
	});
	let json = Json(UpdateRequest {
		email: None,
		first_name: Some(String::from("PartiallyUpdated")),
		last_name: None,
		password: None,
		current_password: None,
		budget_preference: None,
		risk_preference: None,
		food_allergies: Some(String::from("Gluten")),
		disabilities: None,
	});
	_ = controllers::account::api_update(pool, user, json)
		.await
		.unwrap();
}

async fn test_update_endpoint_with_preferences(
	mut cookies: CookieJar,
	key: Extension<Key>,
	pool: Extension<PgPool>,
) {
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
	let email = format!("prefs+{}@example.com", unique);
	let json = Json(SignupRequest {
		email,
		first_name: String::from("Prefs"),
		last_name: String::from("Tester"),
		password: String::from("Password123"),
	});
	// Signup user
	controllers::account::api_signup(&mut cookies, key.clone(), pool.clone(), json)
		.await
		.unwrap();

	// Test /update endpoint with enum preferences
	let cookie = cookies.get("auth-token").unwrap();
	let parts: Vec<&str> = cookie.value().split(&['-', '.']).collect();
	let user = Extension(AuthUser {
		id: parts[1].parse().unwrap(),
	});
	let json = Json(UpdateRequest {
		email: None,
		first_name: None,
		last_name: None,
		password: None,
		current_password: None,
		budget_preference: Some(BudgetBucket::LuxuryBudget),
		risk_preference: Some(RiskTolerence::RiskTaker),
		food_allergies: None,
		disabilities: None,
	});
	_ = controllers::account::api_update(pool, user, json)
		.await
		.unwrap();
}

async fn test_get_itinerary_id_not_found(
	mut cookies: CookieJar,
	key: Extension<Key>,
	pool: Extension<PgPool>,
) {
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
	let email = format!("get_itinerary+{}@example.com", unique);
	let json = Json(SignupRequest {
		email,
		first_name: String::from("Get"),
		last_name: String::from("Itinerary"),
		password: String::from("Password123"),
	});
	// Signup user
	controllers::account::api_signup(&mut cookies, key.clone(), pool.clone(), json)
		.await
		.unwrap();

	// Test /{id} endpoint with non-existent itinerary (should return 404)
	let cookie = cookies.get("auth-token").unwrap();
	let parts: Vec<&str> = cookie.value().split(&['-', '.']).collect();
	let user = Extension(AuthUser {
		id: parts[1].parse().unwrap(),
	});
	assert_eq!(
		controllers::itinerary::api_get_itinerary(user, axum::extract::Path(999999), pool.clone())
			.await
			.unwrap_err()
			.status_code()
			.as_u16(),
		404
	);
}

async fn test_invalid_signup_email(
	mut cookies: CookieJar,
	key: Extension<Key>,
	pool: Extension<PgPool>,
) {
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
	let email = format!("invalid_email_{}", unique);
	let json = Json(SignupRequest {
		email,
		first_name: String::from("Get"),
		last_name: String::from("Event"),
		password: String::from("Password123"),
	});
	// Signup user
	assert_eq!(
		controllers::account::api_signup(&mut cookies, key.clone(), pool.clone(), json)
			.await
			.unwrap_err()
			.status_code()
			.as_u16(),
		400
	);
}

async fn test_saved_itineraries_endpoint(
	mut cookies: CookieJar,
	key: Extension<Key>,
	pool: Extension<PgPool>,
) {
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
	let email = format!("saved_itineraries+{}@example.com", unique);
	let json = Json(SignupRequest {
		email,
		first_name: String::from("Saved"),
		last_name: String::from("Itineraries"),
		password: String::from("Password123"),
	});
	// Signup user
	controllers::account::api_signup(&mut cookies, key.clone(), pool.clone(), json)
		.await
		.unwrap();

	// Test /saved endpoint returns user's itineraries
	let cookie = cookies.get("auth-token").unwrap();
	let parts: Vec<&str> = cookie.value().split(&['-', '.']).collect();
	let user = Extension(AuthUser {
		id: parts[1].parse().unwrap(),
	});
	_ = controllers::itinerary::api_saved_itineraries(user, pool)
		.await
		.unwrap();
}

async fn test_save_itineraries(
	mut cookies: CookieJar,
	key: Extension<Key>,
	pool: Extension<PgPool>,
) {
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
	let email = format!("test_save_itinerary_new+{}@example.com", unique);
	let json = Json(SignupRequest {
		email,
		first_name: String::from("Saved"),
		last_name: String::from("Itineraries"),
		password: String::from("Password123"),
	});
	// Signup user
	controllers::account::api_signup(&mut cookies, key.clone(), pool.clone(), json)
		.await
		.unwrap();

	// save itinerary with id not in db
	let cookie = cookies.get("auth-token").unwrap();
	let parts: Vec<&str> = cookie.value().split(&['-', '.']).collect();
	let user = Extension(AuthUser {
		id: parts[1].parse().unwrap(),
	});
	let json = Json(Itinerary {
		id: 0,
		start_date: NaiveDate::parse_from_str("2025-01-01", "%Y-%m-%d").unwrap(),
		end_date: NaiveDate::parse_from_str("2025-12-31", "%Y-%m-%d").unwrap(),
		event_days: vec![],
		unassigned_events: vec![],
		chat_session_id: None,
		title: String::from("Updated Title"),
	});
	let itinerary_id = controllers::itinerary::api_save(user, pool.clone(), json)
		.await
		.unwrap()
		.id;
	assert_ne!(itinerary_id, 0);

	// save itinerary with a matching id already in db
	let json = Json(Itinerary {
		id: itinerary_id,
		start_date: NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
		end_date: NaiveDate::parse_from_str("2026-12-31", "%Y-%m-%d").unwrap(),
		event_days: vec![],
		unassigned_events: vec![],
		chat_session_id: None,
		title: String::from("2nd Updated Title"),
	});
	assert_eq!(
		controllers::itinerary::api_save(user, pool, json)
			.await
			.unwrap()
			.id,
		itinerary_id
	);
}

async fn test_chat_flow(mut cookies: CookieJar, key: Extension<Key>, pool: Extension<PgPool>) {
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
	let email = format!("test_latest_message_page+{}@example.com", unique);
	let json = Json(SignupRequest {
		email,
		first_name: String::from("Saved"),
		last_name: String::from("Itineraries"),
		password: String::from("Password123"),
	});
	// Signup user
	controllers::account::api_signup(&mut cookies, key.clone(), pool.clone(), json)
		.await
		.unwrap();

	// Create agent for testing - use dummy agent if DEPLOY_LLM != "1"
	// Create the research, constraint, and optimize agents
	let research_agent = Arc::new(Mutex::new(
		agent::configs::research::create_research_agent().unwrap(),
	));
	let constraint_agent = Arc::new(Mutex::new(
		agent::configs::constraint::create_constraint_agent().unwrap(),
	));
	let optimize_agent = Arc::new(Mutex::new(
		agent::configs::optimizer::create_optimize_agent().unwrap(),
	));

	// Create the orchestrator agent with references to the research, constraint, and optimize agents
	let agent = if std::env::var("DEPLOY_LLM").unwrap_or_default() == "1" {
		// Real agent - requires valid OPENAI_API_KEY
		tokio::time::timeout(
			Duration::from_secs(5),
			tokio::task::spawn_blocking(move || {
				agent::configs::orchestrator::create_orchestrator_agent(
					research_agent.clone(),
					constraint_agent.clone(),
					optimize_agent.clone(),
				)
				.unwrap_or_else(|e| {
					panic!(
						"Agent creation failed in test_chat_flow: {}. Check your OPENAI_API_KEY.",
						e
					);
				})
			}),
		)
		.await
		.expect("Agent creation timed out after 5 seconds. Check your OpenAI API key.")
		.expect("Agent creation task panicked")
	} else {
		// Dummy agent - won't be invoked when DEPLOY_LLM is not set
		agent::configs::orchestrator::create_dummy_orchestrator_agent()
			.expect("Dummy agent creation failed")
	};

	let agent = Extension(std::sync::Arc::new(tokio::sync::Mutex::new(agent)));

	// create new chat
	let cookie = cookies.get("auth-token").unwrap();
	let parts: Vec<&str> = cookie.value().split(&['-', '.']).collect();
	let user = Extension(AuthUser {
		id: parts[1].parse().unwrap(),
	});
	let first_chat_session_id = controllers::chat::api_new_chat(user, pool.clone())
		.await
		.unwrap()
		.chat_session_id;
	assert_ne!(first_chat_session_id, 0);

	// create chat session - reusing first one because it's empty
	let chat_session_id = controllers::chat::api_new_chat(user, pool.clone())
		.await
		.unwrap()
		.chat_session_id;
	assert_eq!(first_chat_session_id, chat_session_id);

	// send a bunch of messages
	let mut message_ids = [0; MESSAGE_PAGE_LEN as usize + 5];
	for i in 0..MESSAGE_PAGE_LEN as usize + 5 {
		let json = Json(SendMessageRequest {
			chat_session_id,
			text: format!("Test msg {}", i),
			itinerary_id: None,
		});
		message_ids[i] =
			controllers::chat::api_send_message(user, pool.clone(), agent.clone(), json)
				.await
				.unwrap()
				.user_message_id;
		assert_ne!(message_ids[i], 0);
	}

	// send empty message
	let json = Json(SendMessageRequest {
		chat_session_id,
		text: String::new(),
		itinerary_id: None,
	});
	assert_eq!(
		controllers::chat::api_send_message(user, pool.clone(), agent.clone(), json)
			.await
			.unwrap_err()
			.status_code()
			.as_u16(),
		400
	);

	// send message invalid chat session
	let json = Json(SendMessageRequest {
		chat_session_id: 0,
		text: String::from("Test msg invalid chat session id"),
		itinerary_id: None,
	});
	assert_eq!(
		controllers::chat::api_send_message(user, pool.clone(), agent.clone(), json)
			.await
			.unwrap_err()
			.status_code()
			.as_u16(),
		404
	);

	// get latest messages and make sure messages are in chronological order
	let chat_session = controllers::chat::api_chats(user, pool.clone())
		.await
		.unwrap();
	let chat_session = chat_session.0.chat_sessions.first().unwrap();
	let json = Json(MessagePageRequest {
		chat_session_id: chat_session.id,
		message_id: None,
	});
	let latest_page = controllers::chat::api_message_page(user, pool.clone(), json)
		.await
		.unwrap();
	assert!(
		latest_page
			.0
			.message_page
			.is_sorted_by(|a, b| a.timestamp < b.timestamp)
	);

	// get specific messages and make sure messages are in chronological order
	let json = Json(MessagePageRequest {
		chat_session_id: chat_session.id,
		message_id: Some(latest_page.message_page[0].id),
	});
	let next_page = controllers::chat::api_message_page(user, pool.clone(), json)
		.await
		.unwrap();
	assert!(
		next_page
			.0
			.message_page
			.is_sorted_by(|a, b| a.timestamp < b.timestamp)
	);
	assert_eq!(
		latest_page.message_page[0].id,
		next_page.message_page.last().unwrap().id
	);

	// get page with invalid message id
	let json = Json(MessagePageRequest {
		chat_session_id: chat_session.id,
		message_id: Some(0),
	});
	let empty_page = controllers::chat::api_message_page(user, pool.clone(), json)
		.await
		.unwrap();
	assert_eq!(empty_page.message_page.len(), 0);
	assert_eq!(empty_page.prev_message_id, None);

	// get page with invalid chat session id

	// update message with empty text
	let json = Json(UpdateMessageRequest {
		message_id: message_ids[0],
		new_text: String::new(),
		itinerary_id: None,
	});
	assert_eq!(
		controllers::chat::api_update_message(user, pool.clone(), agent.clone(), json)
			.await
			.unwrap_err()
			.status_code()
			.as_u16(),
		400
	);

	// update message with invalid message id
	let json = Json(UpdateMessageRequest {
		message_id: 0,
		new_text: String::from("Updated message"),
		itinerary_id: None,
	});
	assert_eq!(
		controllers::chat::api_update_message(user, pool.clone(), agent.clone(), json)
			.await
			.unwrap_err()
			.status_code()
			.as_u16(),
		404
	);

	// update message
	let json = Json(UpdateMessageRequest {
		message_id: message_ids[0],
		new_text: String::from("Updated message"),
		itinerary_id: None,
	});
	_ = controllers::chat::api_update_message(user, pool.clone(), agent.clone(), json)
		.await
		.unwrap();
	let json = Json(MessagePageRequest {
		chat_session_id: chat_session.id,
		message_id: None,
	});
	let latest_page = controllers::chat::api_message_page(user, pool.clone(), json)
		.await
		.unwrap();
	assert_eq!(latest_page.prev_message_id, None);
	assert_eq!(latest_page.message_page.len(), 2);

	//rename chat empty string
	let json = Json(RenameRequest {
		new_title: String::from(""),
		id: chat_session.id,
	});
	assert_eq!(
		controllers::chat::api_rename(user, pool.clone(), json)
			.await
			.unwrap_err()
			.status_code()
			.as_u16(),
		400
	);

	//rename chat
	let new_title = String::from("Updated Title");
	let json = Json(RenameRequest {
		new_title: new_title.clone(),
		id: chat_session.id,
	});
	controllers::chat::api_rename(user, pool.clone(), json)
		.await
		.unwrap();
	let Json(chats) = controllers::chat::api_chats(user, pool.clone())
		.await
		.unwrap();
	assert!(
		chats
			.chat_sessions
			.iter()
			.any(move |chat| chat.id == chat_session.id && chat.title == new_title)
	);

	//delete chat session
	controllers::chat::api_delete_chat(user, pool.clone(), axum::extract::Path(chat_session_id))
		.await
		.unwrap();
	let json = Json(MessagePageRequest {
		chat_session_id: chat_session.id,
		message_id: None,
	});
	let latest_page = controllers::chat::api_message_page(user, pool, json)
		.await
		.unwrap();
	assert_eq!(latest_page.prev_message_id, None);
	assert_eq!(latest_page.message_page.len(), 0);
}

async fn test_user_event_flow(
	mut cookies: CookieJar,
	key: Extension<Key>,
	pool: Extension<PgPool>,
) {
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
	let email = format!("test_user_event_flow+{}@example.com", unique);
	let json = Json(SignupRequest {
		email,
		first_name: String::from("Saved"),
		last_name: String::from("Itineraries"),
		password: String::from("Password123"),
	});
	// Signup user
	controllers::account::api_signup(&mut cookies, key.clone(), pool.clone(), json)
		.await
		.unwrap();

	// create event
	let cookie = cookies.get("auth-token").unwrap();
	let parts: Vec<&str> = cookie.value().split(&['-', '.']).collect();
	let user = Extension(AuthUser {
		id: parts[1].parse().unwrap(),
	});
	let test = String::from("test");
	let description = String::from("Unit test event");
	let json = Json(UserEventRequest {
		id: None,
		event_name: test.clone(),
		street_address: Some(test.clone()),
		postal_code: Some(1),
		city: Some(test.clone()),
		country: Some(test.clone()),
		event_type: Some(test.clone()),
		event_description: Some(description.clone()),
		hard_start: Some(
			NaiveDateTime::parse_from_str("2015-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap(),
		),
		hard_end: Some(
			NaiveDateTime::parse_from_str("2025-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap(),
		),
		timezone: Some(String::from("UTC")),
	});
	let Json(UserEventResponse { id }) =
		controllers::itinerary::api_user_event(user, pool.clone(), json)
			.await
			.unwrap();

	// update event
	let update_str = String::from("test updated");
	let json = Json(UserEventRequest {
		id: Some(id),
		event_name: update_str.clone(),
		event_description: Some(description),
		street_address: Some(test.clone()),
		postal_code: Some(1),
		city: Some(test.clone()),
		country: Some(test.clone()),
		event_type: Some(test.clone()),
		hard_start: Some(
			NaiveDateTime::parse_from_str("2015-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap(),
		),
		hard_end: Some(
			NaiveDateTime::parse_from_str("2025-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap(),
		),
		timezone: Some(String::from("UTC")),
	});
	let Json(res) = controllers::itinerary::api_user_event(user, pool.clone(), json)
		.await
		.unwrap();
	assert_eq!(id, res.id);

	// search event
	let json = Json(SearchEventRequest {
		id: Some(id),
		..Default::default()
	});
	let Json(res) = controllers::itinerary::api_search_event(user, pool.clone(), json)
		.await
		.unwrap();
	assert!(res.events.iter().any(|e| e.event_name == update_str));

	// comprehensive search
	let json = Json(SearchEventRequest {
		id: Some(id),
		street_address: Some(test.clone()),
		postal_code: Some(1),
		city: Some(test.clone()),
		country: Some(test.clone()),
		event_type: Some(test.clone()),
		event_description: Some(test.clone()),
		event_name: Some(test.clone()),
		hard_start_before: Some(
			NaiveDateTime::parse_from_str("2020-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap(),
		),
		hard_start_after: Some(
			NaiveDateTime::parse_from_str("2010-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap(),
		),
		hard_end_before: Some(
			NaiveDateTime::parse_from_str("2030-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap(),
		),
		hard_end_after: Some(
			NaiveDateTime::parse_from_str("2020-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap(),
		),
		timezone: Some(String::from("UTC")),
	});
	let Json(res) = controllers::itinerary::api_search_event(user, pool.clone(), json)
		.await
		.unwrap();
	assert!(res.events.iter().any(|e| e.event_name == update_str));

	// delete event
	controllers::itinerary::api_delete_user_event(user, pool.clone(), axum::extract::Path(id))
		.await
		.unwrap();

	// verify deletion
	let json = Json(SearchEventRequest {
		id: Some(id),
		..Default::default()
	});
	let Json(res) = controllers::itinerary::api_search_event(user, pool, json)
		.await
		.unwrap();
	assert!(!res.events.iter().any(|e| e.event_name == update_str));
}

// INTEGRATION TESTS

static mut PORT: u16 = 0;

#[tokio::test]
#[serial(db, log, panic_log)]
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

	let pool = db::create_pool().await;
	match migrate!().run(&pool).await {
		Ok(_) => (),
		Err(sqlx::migrate::MigrateError::VersionMismatch(_)) => {
			eprintln!("migrations version mismatch; assuming DB already prepared. Skipping.");
		}
		Err(e) => panic!("migrations run: {e}"),
	}

	// Build app
	// Use an encryption/signing key for private cookies
	let cookie_key = Key::generate();

	// Create agent for tests - use dummy agent if DEPLOY_LLM != "1"
	let research_agent = Arc::new(Mutex::new(
		agent::configs::research::create_research_agent().unwrap(),
	));
	let constraint_agent = Arc::new(Mutex::new(
		agent::configs::constraint::create_constraint_agent().unwrap(),
	));
	let optimize_agent = Arc::new(Mutex::new(
		agent::configs::optimizer::create_optimize_agent().unwrap(),
	));

	// Create the orchestrator agent with references to the research, constraint, and optimize agents
	let agent = if std::env::var("DEPLOY_LLM").unwrap_or_default() == "1" {
		// Real agent - requires valid OPENAI_API_KEY
		tokio::time::timeout(
			Duration::from_secs(5),
			tokio::task::spawn_blocking(move || {
				agent::configs::orchestrator::create_orchestrator_agent(
					research_agent.clone(),
					constraint_agent.clone(),
					optimize_agent.clone(),
				)
				.unwrap_or_else(|e| {
					panic!(
						"Agent creation failed in test: {}. Check your OPENAI_API_KEY.",
						e
					);
				})
			}),
		)
		.await
		.expect("Agent creation timed out after 5 seconds. Check your OpenAI API key.")
		.expect("Agent creation task panicked")
	} else {
		// Dummy agent - won't be invoked when DEPLOY_LLM is not set
		agent::configs::orchestrator::create_dummy_orchestrator_agent()
			.expect("Dummy agent creation failed")
	};

	let account_routes = controllers::account::account_routes();
	let itinerary_routes = controllers::itinerary::itinerary_routes();
	let chat_routes = controllers::chat::chat_routes();
	let api_routes = Router::new()
		.nest("/account", account_routes)
		.nest("/itinerary", itinerary_routes)
		.nest("/chat", chat_routes);
	let app = Router::new()
		.nest("/api", api_routes)
		.layer(Extension(pool.clone()))
		.layer(Extension(cookie_key.clone()))
		.layer(Extension(std::sync::Arc::new(tokio::sync::Mutex::new(
			agent,
		))))
		.layer(CookieManagerLayer::new());

	// Bind to ephemeral port and spawn server
	let listener = TcpListener::bind("127.0.0.1:0")
		.await
		.expect("bind test server");
	unsafe { PORT = listener.local_addr().unwrap().port() };
	let server = axum::serve(listener, app.into_make_service()).into_future();
	tokio::spawn(server);

	// Any unit tests that test cookies or middleware, or any integration tests should go here.
	// Any other unit test should not go here. Instead, run it as a separate unit test and just invoke the controller directly.
	tokio::join!(
		test_signup_and_login_happy_path(&cookie_key),
		test_auth_for_all_required(),
		test_http_signup_and_login_flow(),
		test_validate_with_bad_and_good_cookie(),
		test_get_itinerary_invalid_format(),
		test_signup_logout(),
		test_cookie_exp_extended(),
		// just throw all the tests in here
	);
}

async fn test_signup_and_login_happy_path(key: &Key) {
	let hc = httpc_test::new_client(format!("http://localhost:{}", unsafe { PORT })).unwrap();
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
	assert_eq!(resp.status().as_u16(), 200);

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

async fn test_auth_for_all_required() {
	let hc = httpc_test::new_client(format!("http://localhost:{}", unsafe { PORT })).unwrap();

	let account_update_payload = json!({});
	let chat_message_page_payload = json!({
		"chat_session_id": 1,
		"message_id": 1
	});
	let chat_update_message_payload = json!({
		"message_id": 1,
		"new_text": "test"
	});
	let chat_send_message_payload = json!({
		"chat_session_id": 1,
		"text": "test"
	});
	let chat_rename_payload = json!({
		"new_title": "Updated Title",
		"id": 1
	});
	let itinerary_save_payload = json!({
		"id": 1,
		"start_date": "2025-11-05 00:00:00",
		"end_date": "2025-11-10 00:00:00",
		"morning_events": [],
		"noon_events": [],
		"afternoon_events": [],
		"evening_events": []
	});
	let itinerary_user_event_payload = json!({
		"event_name": "Test Event"
	});
	let itinerary_search_event_payload = json!({
		"event_description": "test"
	});

	for res in futures::future::join_all([
		hc.do_get("/api/account/current"),
		hc.do_get("/api/account/validate"),
		hc.do_get("/api/account/logout"),
		hc.do_get("/api/chat/chats"),
		hc.do_get("/api/chat/newChat"),
		hc.do_get("/api/itinerary/saved"),
		hc.do_get("/api/itinerary/:id"),
	])
	.await
	.iter()
	{
		assert_eq!(
			res.as_ref().unwrap().status().as_u16(),
			401,
			"Protected route should require authentication"
		);
	}

	for res in futures::future::join_all([
		hc.do_post("/api/account/update", account_update_payload),
		hc.do_post("/api/chat/messagePage", chat_message_page_payload),
		hc.do_post("/api/chat/updateMessage", chat_update_message_payload),
		hc.do_post("/api/chat/sendMessage", chat_send_message_payload),
		hc.do_post("/api/chat/rename", chat_rename_payload),
		hc.do_post("/api/itinerary/save", itinerary_save_payload),
		hc.do_post("/api/itinerary/userEvent", itinerary_user_event_payload),
		hc.do_post("/api/itinerary/searchEvent", itinerary_search_event_payload),
	])
	.await
	.iter()
	{
		assert_eq!(
			res.as_ref().unwrap().status().as_u16(),
			401,
			"Protected route should require authentication"
		);
	}

	for res in futures::future::join_all([
		hc.do_delete("/api/itinerary/userEvent/1"),
		hc.do_delete("/api/chat/1"),
	])
	.await
	.iter()
	{
		assert_eq!(
			res.as_ref().unwrap().status().as_u16(),
			401,
			"Protected route should require authentication"
		);
	}
}

async fn test_http_signup_and_login_flow() {
	let hc = httpc_test::new_client(format!("http://localhost:{}", unsafe { PORT })).unwrap();
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
	assert!(
		resp.status().is_success(),
		"signup failed: {}",
		resp.status()
	);

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
	assert!(
		resp.status().is_success(),
		"login failed: {}",
		resp.status()
	);
}

async fn test_validate_with_bad_and_good_cookie() {
	let hc = httpc_test::new_client(format!("http://localhost:{}", unsafe { PORT })).unwrap();
	// No cookie (treated similarly to bad/invalid cookie): expect unauthorized
	let resp = hc.do_get("/api/account/validate").await.unwrap();
	assert_eq!(
		resp.status().as_u16(),
		401,
		"Missing/invalid cookie should return 401"
	);

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
	assert_eq!(signup.status().as_u16(), 200);

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
	let resp = hc.do_get("/api/account/validate").await.unwrap();
	assert_eq!(
		resp.status().as_u16(),
		200,
		"/validate with good cookie should return 200"
	);
}

async fn test_get_itinerary_invalid_format() {
	let hc = httpc_test::new_client(format!("http://localhost:{}", unsafe { PORT })).unwrap();
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
	assert_eq!(signup_resp.status().as_u16(), 200);

	// Test with invalid ID format (should return 400)
	let invalid_resp = hc.do_get("/api/itinerary/invalid").await.unwrap();
	assert_eq!(invalid_resp.status().as_u16(), 400);
}

async fn test_signup_logout() {
	let hc = httpc_test::new_client(format!("http://localhost:{}", unsafe { PORT })).unwrap();
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
	let email = format!("login_then_logout+{}@example.com", unique);

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
	assert_eq!(signup_resp.status().as_u16(), 200);

	let cookie = signup_resp.res_cookie("auth-token").unwrap();
	assert!(cookie.expires.unwrap() > SystemTime::now());

	// Logout
	let logout_resp = hc.do_get("/api/account/logout").await.unwrap();
	assert_eq!(logout_resp.status().as_u16(), 200);

	let cookie = logout_resp.res_cookie("auth-token").unwrap();
	assert!(cookie.expires.unwrap() < SystemTime::now());

	// Hit any protected route
	let validate_res = hc.do_get("/api/account/validate").await.unwrap();
	assert_eq!(
		validate_res.status().as_u16(),
		401,
		"Missing/invalid cookie should return 401"
	);
}

async fn test_cookie_exp_extended() {
	let hc = httpc_test::new_client(format!("http://localhost:{}", unsafe { PORT })).unwrap();
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
	let email = format!("test_cookie_exp_extended+{}@example.com", unique);

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
	assert_eq!(signup_resp.status().as_u16(), 200);

	let cookie = signup_resp.res_cookie("auth-token").unwrap();
	assert!(cookie.expires.unwrap() > SystemTime::now());
	assert!(
		cookie.expires.unwrap()
			< SystemTime::now()
				.checked_add(Duration::from_secs(TEST_COOKIE_EXP_SECONDS as u64))
				.unwrap()
	);

	// Hit any protected route
	let validate_resp = hc.do_get("/api/account/validate").await.unwrap();
	assert_eq!(validate_resp.status().as_u16(), 200);

	let cookie = validate_resp.res_cookie("auth-token").unwrap();
	assert!(
		cookie.expires.unwrap()
			> SystemTime::now()
				.checked_add(Duration::from_secs(TEST_COOKIE_EXP_SECONDS as u64))
				.unwrap()
	);
	assert!(
		cookie.expires.unwrap()
			< SystemTime::now()
				.checked_add(Duration::from_secs(3600))
				.unwrap()
	);
}

async fn test_unsave_itinerary_success(
	mut cookies: CookieJar,
	key: Extension<Key>,
	pool: Extension<PgPool>,
) {
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
	let email = format!("test_unsave_success+{}@example.com", unique);
	let json = Json(SignupRequest {
		email,
		first_name: String::from("Unsave"),
		last_name: String::from("Success"),
		password: String::from("Password123"),
	});
	// Signup user
	controllers::account::api_signup(&mut cookies, key.clone(), pool.clone(), json)
		.await
		.unwrap();

	let cookie = cookies.get("auth-token").unwrap();
	let parts: Vec<&str> = cookie.value().split(&['-', '.']).collect();
	let user = Extension(AuthUser {
		id: parts[1].parse().unwrap(),
	});

	// First, save an itinerary
	let json = Json(Itinerary {
		id: 0,
		start_date: NaiveDate::parse_from_str("2025-01-01", "%Y-%m-%d").unwrap(),
		end_date: NaiveDate::parse_from_str("2025-12-31", "%Y-%m-%d").unwrap(),
		event_days: vec![],
		unassigned_events: vec![],
		chat_session_id: None,
		title: String::from("Test Itinerary to Unsave"),
	});
	let itinerary_id = controllers::itinerary::api_save(user, pool.clone(), json)
		.await
		.unwrap()
		.id;
	assert_ne!(itinerary_id, 0);

	// Now unsave it
	let json = Json(UnsaveRequest { id: itinerary_id });
	controllers::itinerary::api_unsave(user, pool.clone(), json)
		.await
		.unwrap();

	// Verify it's no longer in saved itineraries
	let saved = controllers::itinerary::api_saved_itineraries(user, pool)
		.await
		.unwrap();
	assert!(!saved.itineraries.iter().any(|i| i.id == itinerary_id));
}

async fn test_unsave_itinerary_not_found(
	mut cookies: CookieJar,
	key: Extension<Key>,
	pool: Extension<PgPool>,
) {
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
	let email = format!("test_unsave_not_found+{}@example.com", unique);
	let json = Json(SignupRequest {
		email,
		first_name: String::from("Unsave"),
		last_name: String::from("NotFound"),
		password: String::from("Password123"),
	});
	// Signup user
	controllers::account::api_signup(&mut cookies, key.clone(), pool.clone(), json)
		.await
		.unwrap();

	let cookie = cookies.get("auth-token").unwrap();
	let parts: Vec<&str> = cookie.value().split(&['-', '.']).collect();
	let user = Extension(AuthUser {
		id: parts[1].parse().unwrap(),
	});

	// Try to unsave a non-existent itinerary
	let json = Json(UnsaveRequest { id: 999999 });
	assert_eq!(
		controllers::itinerary::api_unsave(user, pool, json)
			.await
			.unwrap_err()
			.status_code()
			.as_u16(),
		404
	);
}

async fn test_unsave_already_unsaved_itinerary(
	mut cookies: CookieJar,
	key: Extension<Key>,
	pool: Extension<PgPool>,
) {
	let unique = Utc::now().timestamp_nanos_opt().unwrap();
	let email = format!("test_unsave_already_unsaved+{}@example.com", unique);
	let json = Json(SignupRequest {
		email,
		first_name: String::from("Unsave"),
		last_name: String::from("Already"),
		password: String::from("Password123"),
	});
	// Signup user
	controllers::account::api_signup(&mut cookies, key.clone(), pool.clone(), json)
		.await
		.unwrap();

	let cookie = cookies.get("auth-token").unwrap();
	let parts: Vec<&str> = cookie.value().split(&['-', '.']).collect();
	let user = Extension(AuthUser {
		id: parts[1].parse().unwrap(),
	});

	// Save an itinerary
	let json = Json(Itinerary {
		id: 0,
		start_date: NaiveDate::parse_from_str("2025-01-01", "%Y-%m-%d").unwrap(),
		end_date: NaiveDate::parse_from_str("2025-12-31", "%Y-%m-%d").unwrap(),
		event_days: vec![],
		unassigned_events: vec![],
		chat_session_id: None,
		title: String::from("Test Itinerary"),
	});
	let itinerary_id = controllers::itinerary::api_save(user, pool.clone(), json)
		.await
		.unwrap()
		.id;

	// Unsave it once
	let json = Json(UnsaveRequest { id: itinerary_id });
	controllers::itinerary::api_unsave(user, pool.clone(), json)
		.await
		.unwrap();

	// Try to unsave it again
	let json = Json(UnsaveRequest { id: itinerary_id });
	controllers::itinerary::api_unsave(user, pool, json)
		.await
		.unwrap();
}
