use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Row model for the `itineraries` table.
/// - Fields:
///   - `id`: Primary key
///   - `account_id`: Owner account id (FK)
///   - `is_public`: Visibility flag
///   - `date`: Event date/time (UTC naive)
///   - `event_list`: List of events in this itinerary
#[derive(Debug, Serialize, Deserialize)]
pub struct ItineraryJoinedRow {
    pub id: i32,
    pub account_id: Option<i32>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub chat_session_id: Option<i32>,
    pub title: String
}