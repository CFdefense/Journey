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

use crate::http_models::event::Event;

#[derive(Debug, Serialize, Deserialize)]
pub struct Itinerary {
    pub date: NaiveDateTime,
    pub morning_events: Vec<Event>,
    pub noon_events: Vec<Event>,
    pub afternoon_events: Vec<Event>,
    pub evening_events: Vec<Event>
}

/// API route response for GET `/api/itinerary/saved`
/// - Fields:
///   - `itineraries`: List of saved itinerary summaries for the authenticated user.
#[derive(Debug, Serialize, Deserialize)]
pub struct SavedResponse {
    pub itineraries: Vec<Itinerary>,
}