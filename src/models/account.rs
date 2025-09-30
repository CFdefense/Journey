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

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    // TODO: More Preferences...
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SignupPayload {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
}

// TODO: More Payloads...
