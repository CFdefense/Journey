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
use sqlx::{FromRow, Type};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: i32,
    pub street_address: String,
    pub postal_code: i32,
    pub city: String,
    pub event_type: EventType,
    pub event_description: String,
    pub event_name: String,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum EventType {
    Concert,
    Museum,
    Restaurant,
    Hike,
    Festival,
    Sports,
    Other,
    // TODO: Add more event types...
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateEventPayload {
    pub street_address: Option<String>,
    pub postal_code: Option<i32>,
    pub city: Option<String>,
    pub event_type: Option<EventType>,
    pub event_description: Option<String>,
    pub event_name: Option<String>,
}

// TODO: More Payloads...
