use serde::Serialize;

/// Response model from the `/api/chat/chats` endpoint
#[derive(Serialize)]
pub struct ChatsResponse {
	/// chat session ids belonging to the user who made the request
    pub chat_sessions: Vec<i32>
}

/// Response model from the `/api/chat/newChat` endpoint
#[derive(Serialize)]
pub struct NewChatResponse {
	/// this chat session is guaranteed to not have any messages in it
    pub chat_session_id: i32
}