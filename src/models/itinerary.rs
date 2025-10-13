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

#[derive(Debug, Serialize, Deserialize)]
pub struct Itinerary {
    pub id: i32,
    pub account_id: i32,
    pub is_public: bool,
    pub date: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedResponse {
    pub itineraries: Vec<Itinerary>,
}