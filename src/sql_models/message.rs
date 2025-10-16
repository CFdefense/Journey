use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Serialize, Deserialize, FromRow)]
pub struct MessageRow {
	pub id: i32,
	pub chat_session_id: i32,
	pub itinerary_id: Option<i32>,
	pub is_user: bool,
	pub timestamp: NaiveDateTime,
	pub text: String
}