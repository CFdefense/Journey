use serde::Serialize;

#[derive(Serialize)]
pub struct ChatsResponse {
    pub chat_sessions: Vec<i32>
}