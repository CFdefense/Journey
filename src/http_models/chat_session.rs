use serde::Serialize;

#[derive(Serialize)]
pub struct ChatsResponse {
    pub chat_sessions: Vec<i32>
}

#[derive(Serialize)]
pub struct NewChatResponse {
    pub chat_session_id: i32
}