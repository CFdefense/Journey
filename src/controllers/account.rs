/*
 * src/models/Account.rs
 *
 * File for Account table models
 *
 * Purpose:
 *   Models for the account table and payloads which interact with it.
 *
 * Include:
 *   Account            - Model representing an instance of the Account table
 *   LoginPayload       - Model representing the payload for a login
 *   SignupPayload      - Model representing the payload for a signup
 */

use axum::{routing::{get, post}, Json, Router};
use serde_json::{json, Value};
pub async fn api_signup() {
}

pub async fn api_login() {
}

pub async fn test_account() -> Json<Value> {
    Json(json!({
        "message": "test endpoint"
    }))
}

pub fn account_routes() -> Router {
    Router::new()
    .route("/signup", post(api_signup))
    .route("/login", post(api_login))
    .route("/test", get(test_account))
}