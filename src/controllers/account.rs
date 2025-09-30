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
    routing::{get, post},
};
use serde_json::{Value, json};
use sqlx::PgPool;
use tracing::{error, info, trace};

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
/// - `201 Created` with JSON body `{ "id": "..."}`
/// - `400 Bad Request` if the input is invalid.
/// - `409 Conflict` if the email already exists.
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
    payload: SignupPayload,
) -> Result<Json<Value>> {
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
/// - `201 Login Sucess` with JSON body `{ "id": "...", "token": "..."}`
/// - `400 Bad Request` if the input is invalid.
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
    Extension(pool): Extension<PgPool>,
    payload: LoginPayload,
) -> Result<Json<Account>> {
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
