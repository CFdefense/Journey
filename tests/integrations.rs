/*
 * tests/integrations.rs
 *
 * Integration tests for the server
 *
 * Purpose:
 *   Spawning one instance of the server and calling the endpoints over concurrent HTTP requests.
 *
 * How to make new tests:
 *   Create a regular async function WITHOUT `#[tokio::test]`.
 *   In test_endpoints below, there is a tokio::join macro at the bottom of the function which contains all the tests.
 *   Just call your function in that join macro similar to the others.
 */

extern crate capping2025 as app;
use app::{controllers, db};
use axum::{Router, Extension};
use chrono::Utc;
use httpc_test::Client;
use serde_json::json;
use tower_cookies::{cookie::{CookieJar, Key}, Cookie, CookieManagerLayer};
use std::net::TcpListener;
use sqlx::migrate;
use std::sync::Once;

static TEST_LOG_INIT: Once = Once::new();

#[tokio::test]
async fn test_endpoints() {
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
        .layer(Extension(cookie_key.clone()))
        .layer(CookieManagerLayer::new());
    let api_routes = Router::new()
        .nest("/account", account_routes);
    let app = Router::new().nest("/api", api_routes);

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
        .do_post("/api/account/me", json!({}))
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
        .do_post("/api/account/me", json!({}))
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200, "Accessing /me with auth should return 200");
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