/*
 * src/models/Account.rs
 *
 * File for Account table models
 *
 * Purpose:
 *   Models for the account table and payloads which interact with it.
 *
 * Include:
 *   Account            - Model representing an intance of the Account table
 *   LoginPayload       - Model representing the payload for a login
 *   SignupPayload      - Model representing the payload for a signup
 */
pub struct Account {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    // Preferences...
}

pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

pub struct SignupPayload {
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
}
