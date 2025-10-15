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

use crate::sql_models::{event::EventRow, event_list::EventListJoinRow};

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
	pub street_address: String,
    pub postal_code: i32,
    pub city: String,
    pub event_type: String,
    pub event_description: String,
    pub event_name: String
}

impl From<&EventListJoinRow> for Event {
	fn from(value: &EventListJoinRow) -> Self {
		Self {
			street_address: value.street_address.clone(),
			postal_code: value.postal_code,
			city: value.city.clone(),
			event_type: value.event_type.clone(),
			event_description: value.event_description.clone(),
			event_name: value.event_name.clone()
		}
	}
}

#[derive(Deserialize)]
pub struct UpdateEventRequest {
    pub street_address: Option<String>,
    pub postal_code: Option<i32>,
    pub city: Option<String>,
    pub event_type: Option<String>,
    pub event_description: Option<String>,
    pub event_name: Option<String>,
}

// TODO: More Payloads...
