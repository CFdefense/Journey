/*
 * src/models/account.rs
 *
 * File for Account table models and related payload/response types
 *
 * Purpose:
 *   Strongly-typed models for the `accounts` table
 */

use regex::Regex;
use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};
use crate::sql_models::{BudgetBucket, RiskTolerence};

/// Request payload for POST `/api/account/login`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
	/// Account email
    pub email: String,
	/// Plaintext password submitted by the user
    pub password: String,
}

/// Request payload for POST `/api/account/signup`.
/// Validated server-side before insert.
#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct SignupRequest {
	/// Account email
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    /// Plaintext password submitted by the user
    pub password: String,
}

/// Request payload for POST `/api/account/update`.
/// - Only `Some` fields are updated.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateRequest {
	/// Optional new email
    pub email: Option<String>,
	/// Optional new first name
    pub first_name: Option<String>,
	/// Optional new last name
    pub last_name: Option<String>,
	/// Optional new plaintext password
    pub password: Option<String>,
	/// Optional new budget enum
    pub budget_preference: Option<BudgetBucket>,
	/// Optional new risk enum
    pub risk_preference: Option<RiskTolerence>,
	/// Optional new food and allergies preferences
	/// * String is a comma-separated list of preferences
    pub food_allergies: Option<String>,
	/// Optional new disabilites
	/// * String is a comma-separated list of preferences
    pub disabilities: Option<String>,
}

/// API route response for POST `/api/account/update`.
/// - Contains full updated account profile for convenience.
#[derive(Serialize, ToSchema, ToResponse)]
pub struct UpdateResponse {
	/// User id
    pub id: i32,
	/// Current email
    pub email: String,
	/// Current first name
    pub first_name: String,
	/// Current last name
    pub last_name: String,
	/// Optional budget enum
    pub budget_preference: Option<BudgetBucket>,
	/// Optional risk enum
    pub risk_preference: Option<RiskTolerence>,
	/// Optional food and allergies preferences
	/// * String is a comma-separated list of preferences
    pub food_allergies: Option<String>,
	/// Optional disabilites
	/// * String is a comma-separated list of preferences
    pub disabilities: Option<String>,
}

/// API route response for GET `/api/account/current`.
/// - Safe-to-return account profile for current user
#[derive(Serialize, ToSchema, ToResponse)]
pub struct CurrentResponse {
	/// Email
    pub email: String,
	/// First name
    pub first_name: String,
	/// Last name
    pub last_name: String,
	/// Optional budget enum
    pub budget_preference: Option<BudgetBucket>,
	/// Optional risk enum
    pub risk_preference: Option<RiskTolerence>,
	/// Optional food and allergies preferences
    pub food_allergies: Option<String>,
	/// Optional food and allergies preferences
    pub disabilities: Option<String>
}

impl SignupRequest {
    /// Validate email format using regex.
    /// Validate email format using regex
    pub fn validate_email(email: &str) -> bool {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        email_regex.is_match(email)
    }

    /// Validate password strength
    /// - Minimum 8 characters
    /// - Maximum 128 characters
    /// - At least one uppercase letter
    /// - At least one lowercase letter
    /// - At least one number
    /// - Only ASCII characters allowed (for security and compatibility)
    pub fn validate_password(password: &str) -> Result<(), String> {
        if password.len() < 8 {
            return Err("Password must be at least 8 characters long".to_string());
        }

        if password.len() > 128 {
            return Err("Password must be 128 characters or less".to_string());
        }

        // Only allow ASCII characters (prevents potential encoding issues)
        if !password.is_ascii() {
            return Err("Password must contain only ASCII characters".to_string());
        }

        if !password.chars().any(|c| c.is_uppercase()) {
            return Err("Password must contain at least one uppercase letter".to_string());
        }

        if !password.chars().any(|c| c.is_lowercase()) {
            return Err("Password must contain at least one lowercase letter".to_string());
        }

        if !password.chars().any(|c| c.is_numeric()) {
            return Err("Password must contain at least one number".to_string());
        }

        Ok(())
    }

    /// Validate the entire signup payload
    pub fn validate(&self) -> Result<(), String> {
        // Validate email (trim before checking)
        let email_trimmed = self.email.trim();
        if email_trimmed.is_empty() {
            return Err("Email is required".to_string());
        }

        if !Self::validate_email(email_trimmed) {
            return Err("Invalid email format".to_string());
        }

        // Validate first name (trim before checking)
        let first_name_trimmed = self.first_name.trim();
        if first_name_trimmed.is_empty() {
            return Err("First name is required".to_string());
        }

        if first_name_trimmed.len() > 50 {
            return Err("First name must be 50 characters or less".to_string());
        }

        // Validate last name (trim before checking)
        let last_name_trimmed = self.last_name.trim();
        if last_name_trimmed.is_empty() {
            return Err("Last name is required".to_string());
        }

        if last_name_trimmed.len() > 50 {
            return Err("Last name must be 50 characters or less".to_string());
        }

        // Validate password
        Self::validate_password(&self.password)?;

        Ok(())
    }
}