use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ChatSession {
    pub id: i32,
    pub account_id: i32
}

#[derive(Serialize)]
pub struct ChatsResponse {
    pub chat_sessions: Vec<ChatSession>
}