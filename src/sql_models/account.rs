/// Row model for the `accounts` table.
/// - Represents a persisted user.
pub struct AccountRow {
	/// Primary key
	pub id: i32,
	/// Unique email address
	#[allow(dead_code)] // email required for login route
	pub email: String,
	/// Argon2 hashed password
	pub password: String,
}
