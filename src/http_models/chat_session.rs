use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

use crate::sql_models::message::ChatSessionRow;

/// Response model from the `/api/chat/chats` endpoint
#[derive(Serialize, ToSchema, ToResponse)]
pub struct ChatsResponse {
	/// chat session ids belonging to the user who made the request
	pub chat_sessions: Vec<ChatSessionRow>,
}

/// Response model from the `/api/chat/newChat` endpoint
#[derive(Serialize, ToSchema, ToResponse)]
pub struct NewChatResponse {
	/// this chat session is guaranteed to not have any messages in it
	pub chat_session_id: i32,
}

/// Request model for the `/api/chat/rename` endpoint
#[derive(Deserialize, ToSchema)]
pub struct RenameRequest {
	pub new_title: String,
	pub id: i32,
}
