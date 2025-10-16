/// Row model for the `accounts` table.
/// - Represents a persisted user.
/// - Fields:
///   - `id`: Primary key
///   - `email`: Unique email address
///   - `password`: Argon2 hashed password
pub struct AccountRow {
    pub id: i32,
    #[allow(dead_code)] // email required for login route
    pub email: String,
    pub password: String
}