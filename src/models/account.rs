/*
 * src/models/account.rs
 *
 * File for Account table models and related payload/response types
 *
 * Purpose:
 *   Strongly-typed models for the `accounts` table, enums for preferences,
 *   request payloads, and response DTOs used by account routes.
 *
 * Include:
 *   Account              - Row model for the accounts table
 *   BudgetBucket         - Enum mapped to DB type budget_bucket
 *   RiskTolerence        - Enum mapped to DB type risk_tolerence
 *   LoginPayload         - Request payload for POST /api/account/login
 *   SignupPayload        - Request payload for POST /api/account/signup
 *   UpdatePayload        - Request payload for POST /api/account/update
 *   LoginResponse        - API route response for POST /api/account/login
 *   SignupResponse       - API route response for POST /api/account/signup
 *   ValidateResponse     - API route response for GET /api/account/validate
 *   UpdateResponse       - API route response for POST /api/account/update
 *   CurrentResponse      - API route response for GET  /api/account/current
 */

use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::Type;

/// Row model for the `accounts` table.
/// - Represents a persisted user.
/// - Fields:
///   - `id`: Primary key
///   - `email`: Unique email address
///   - `password`: Argon2 hashed password
///   - `first_name`: User first name
///   - `last_name`: User last name
///   - `budget_preference`: Optional budget preference enum
///   - `risk_preference`: Optional risk tolerance enum
///   - `food_allergies`: Optional text notes
///   - `disabilities`: Optional text notes
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub budget_preference: Option<BudgetBucket>,
    pub risk_preference: Option<RiskTolerence>,
    pub food_allergies: Option<String>,
    pub disabilities: Option<String>,
    // TODO: More Preferences...
}

/// Budget preference enum mapped to Postgres `budget_bucket`.
/// Used in account preferences and returned by account APIs.
/// - Fields:
///   - Enum variants representing budget bands
#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[sqlx(type_name = "budget_bucket")]
pub enum BudgetBucket {
    VeryLowBudget,
    LowBudget,
    MediumBudget,
    HighBudget,
    LuxuryBudget,
}

/// Risk tolerance enum mapped to Postgres `risk_tolerence`.
/// Used in account preferences and returned by account APIs.
/// - Fields:
///   - Enum variants representing risk appetite
#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[sqlx(type_name = "risk_tolerence")]
pub enum RiskTolerence {
    ChillVibes,
    LightFun,
    Adventurer,
    RiskTaker,
}

/// Request payload for POST `/api/account/login`.
/// - Fields:
///   - `email`: Account email
///   - `password`: Plaintext password submitted by the user
#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

/// Request payload for POST `/api/account/signup`.
/// Validated server-side before insert.
/// - Fields:
///   - `email`, `first_name`, `last_name`, `password`
#[derive(Debug, Deserialize)]
pub struct SignupPayload {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
}

/// Request payload for POST `/api/account/update`.
/// - Only non-None fields are updated.
/// - `password`, when provided, is re-hashed before store.
/// - Fields:
///   - `email`: Optional new email
///   - `first_name`: Optional new first name
///   - `last_name`: Optional new last name
///   - `password`: Optional new password
///   - `budget_preference`: Optional new budget enum
///   - `risk_preference`: Optional new risk enum
///   - `food_allergies`: Optional new notes
///   - `disabilities`: Optional new notes
#[derive(Debug, Deserialize)]
pub struct UpdatePayload {
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub password: Option<String>,
    pub budget_preference: Option<BudgetBucket>,
    pub risk_preference: Option<RiskTolerence>,
    pub food_allergies: Option<String>,
    pub disabilities: Option<String>,
}

// TODO: More Payloads...





/// API route response for POST `/api/account/update`.
/// - Contains full updated account profile for convenience.
/// - Preference fields may be `None` if unset.
/// - Fields:
///   - `id`: User id
///   - `email`: Current email
///   - `first_name`: Current first name
///   - `last_name`: Current last name
///   - `budget_preference`: Optional budget enum
///   - `risk_preference`: Optional risk enum
///   - `food_allergies`: Optional notes
///   - `disabilities`: Optional notes
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateResponse {
    pub id: i32,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub budget_preference: Option<BudgetBucket>,
    pub risk_preference: Option<RiskTolerence>,
    pub food_allergies: Option<String>,
    pub disabilities: Option<String>,
}

/// API route response for GET `/api/account/current`.
/// - Full, safe-to-return account profile for current user
/// - Fields:
///   - `email`: Email
///   - `first_name`: First name
///   - `last_name`: Last name
///   - `budget_preference`: Optional budget enum
///   - `risk_preference`: Optional risk enum
///   - `food_allergies`: Optional notes
///   - `disabilities`: Optional notes
#[derive(Serialize, FromRow)]
pub struct CurrentResponse {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub budget_preference: Option<BudgetBucket>,
    pub risk_preference: Option<RiskTolerence>,
    pub food_allergies: Option<String>,
    pub disabilities: Option<String>
}

// TODO: More Responses...


impl SignupPayload {
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