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

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::http_models::event::Event;

#[derive(Debug, Serialize, Deserialize)]
pub struct Itinerary {
	pub id: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub event_days: Vec<EventDay>,
    pub chat_session_id: Option<i32>,
    pub title: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventDay {
	pub morning_events: Vec<Event>,
    pub noon_events: Vec<Event>,
    pub afternoon_events: Vec<Event>,
    pub evening_events: Vec<Event>,
    pub date: NaiveDate
}

/// API route response for GET `/api/itinerary/saved`
/// - Fields:
///   - `itineraries`: List of saved itinerary summaries for the authenticated user.
#[derive(Debug, Serialize, Deserialize)]
pub struct SavedResponse {
    pub itineraries: Vec<Itinerary>
}

#[derive(Serialize)]
pub struct SaveResponse {
	pub id: i32
}