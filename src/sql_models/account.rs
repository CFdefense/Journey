use crate::sql_models::{BudgetBucket, RiskTolerence};

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
pub struct AccountRow {
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