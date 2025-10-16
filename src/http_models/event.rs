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

impl From<EventRow> for Event {
	#[cfg(not(tarpaulin_include))]
	fn from(value: EventRow) -> Self {
	    Self {
	        street_address: value.street_address,
	        postal_code: value.postal_code,
	        city: value.city,
	        event_type: value.event_type,
	        event_description: value.event_description,
	        event_name: value.event_name
	    }
	}
}

impl From<&EventListJoinRow> for Event {
	#[cfg(not(tarpaulin_include))]
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