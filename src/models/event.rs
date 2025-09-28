/*
 * src/models/event.rs
 *
 * File for Event table models
 *
 * Purpose:
 *   Models for the Event table and payloads which interact with it.
 *
 * Include:
 *   Event            - Model representing an instance of the Event table
 *   EventType        - Enum for the event type associated with an event
 */

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: i32,
    pub street_address: String,
    pub postal_code: i32,
    pub city: String,
    pub event_type: EventType,
    pub event_description: String,
    pub event_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EventType {
    // TODO: Add event types...
}

// TODO: More Payloads...
