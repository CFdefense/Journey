use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct EventRow {
    pub id: i32,
    pub street_address: String,
    pub postal_code: i32,
    pub city: String,
    pub event_type: String,
    pub event_description: String,
    pub event_name: String,
}