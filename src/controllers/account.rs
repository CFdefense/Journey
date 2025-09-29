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
    Json, Router,
    routing::{get, post},
};
use serde_json::{Value, json};

/// /account/signup
/// Logic goes here
///
///
pub async fn api_signup() {}

/// /account/login
/// Logic goes here
///
///
pub async fn api_login() {}

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
