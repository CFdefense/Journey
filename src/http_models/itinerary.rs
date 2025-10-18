/*
 * src/models/itinerary.rs
 *
 * File for Itinerary table models and related responses
 *
 * Purpose:
 *   Strongly-typed models for the `itineraries` table and response DTOs
 *   used by itinerary routes.
 *
 * Include:
 *   Itinerary        - Row model for the itineraries table
 *   SavedResponse    - API route response for GET /api/itinerary/saved
 */

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::sql_models::event::EventRow;

#[derive(Debug, Serialize, Deserialize)]
pub struct Itinerary {
	pub id: i32,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub morning_events: Vec<EventRow>,
    pub noon_events: Vec<EventRow>,
    pub afternoon_events: Vec<EventRow>,
    pub evening_events: Vec<EventRow>,
    pub chat_session_id: Option<i32>
}

/// API route response for GET `/api/itinerary/saved`
/// - Fields:
///   - `itineraries`: List of saved itinerary summaries for the authenticated user.
#[derive(Debug, Serialize, Deserialize)]
pub struct SavedResponse {
    pub itineraries: Vec<Itinerary>,
}

#[derive(Serialize)]
pub struct SaveResponse {
	pub id: i32
}