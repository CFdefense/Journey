/*
 * src/models/event_list.rs
 *
 * File for Event List table models
 *
 * Purpose:
 *   Models for the Event List table and payloads which interact with it.
 *
 * Include:
 *   EventList          - Model representing an instance of the EventList table
 */

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EventList {
    pub id: i32,
    pub itinerary_id: i32,
    pub event_id: i32,
    pub time_of_day: String,
}

// TODO: More Payloads...
