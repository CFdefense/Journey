/*
 * src/models/intinerary.rs
 *
 * File for Itinerary table models
 *
 * Purpose:
 * Models for the Itinerary table and payloads which interact with it.
 *
 * Include:
 *   Itinerary            - Model representing an instance of the Itinerary table
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
