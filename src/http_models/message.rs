use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Message {
	pub id: i32,
	pub is_user: bool,
	pub timestamp: NaiveDateTime,
	pub text: String,
	pub itinerary_id: Option<i32>
}

#[derive(Serialize)]
pub struct MessagePageResponse {
	pub message_page: Vec<Message>,
	pub prev_message_id: Option<i32>
}

#[derive(Deserialize)]
pub struct UpdateMessageRequest {
	pub message_id: i32,
	pub new_text: String
}

#[derive(Serialize)]
pub struct SendMessageResponse {
	pub user_message_id: i32,
	pub bot_message: Message
}