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

use crate::models::event_list::EventList;

/// Row model for the `itineraries` table.
/// - Fields:
///   - `id`: Primary key
///   - `account_id`: Owner account id (FK)
///   - `is_public`: Visibility flag
///   - `date`: Event date/time (UTC naive)
///   - `event_list`: List of events in this itinerary
#[derive(Debug, Serialize, Deserialize)]
pub struct Itinerary {
    pub id: i32,
    pub account_id: i32,
    pub is_public: bool,
    pub date: NaiveDateTime,
}

/// API route response for GET `/api/itinerary/saved`
/// - Fields:
///   - `itineraries`: List of saved itinerary summaries for the authenticated user.
#[derive(Debug, Serialize, Deserialize)]
pub struct SavedResponse {
    pub itineraries: Vec<Itinerary>,
}

/// API route response for GET `/api/itinerary/{id}`.
/// - Fields:
///   - `itinerary`: Single itinerary metadata.
#[derive(Debug, Serialize, Deserialize)]
pub struct ItineraryResponse {
    pub itinerary: Itinerary,
}

/// API route response for GET `/api/itinerary/{id}/events`.
/// - Fields:
///   - `events`: List of events in the itinerary.
#[derive(Debug, Serialize, Deserialize)]
pub struct EventListResponse {
    pub events: Vec<EventList>,
}