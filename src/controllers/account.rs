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

/// /account/signup
/// Will get a SignupPayload from frontend
/// Validate Payload Contents
/// Create a New Account in Database
///
pub async fn api_signup(
    Extension(pool): Extension<PgPool>,
    payload: SignupPayload,
) -> Result<Json<Value>> {
}

/// /account/login
/// Will get a LoginPayload from frontend
/// Validate Login Credentials
/// Redirect Accordingly
/// Set Useer Cookies
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
