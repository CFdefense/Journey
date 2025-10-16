use serde::{Deserialize, Serialize};

use crate::sql_models::TimeOfDay;

#[derive(Debug, Serialize, Deserialize)]
pub struct EventListJoinRow {
    pub time_of_day: TimeOfDay,
    pub street_address: String,
    pub postal_code: i32,
    pub city: String,
    pub event_type: String,
    pub event_description: String,
    pub event_name: String,
}