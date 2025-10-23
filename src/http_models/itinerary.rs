/*
 * src/models/itinerary.rs
 *
 * File for Itinerary table models and related responses
 *
 * Purpose:
 *   Strongly-typed models for the `itineraries` response DTOs
 *   used by itinerary routes.
 */

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

use crate::http_models::event::Event;

/// A complete itinerary with event details
#[derive(Debug, Serialize, Deserialize, ToSchema, ToResponse)]
pub struct Itinerary {
	/// Primary key
	pub id: i32,
	/// UTC date that the first event may take place (%Y-%m-%d)
    pub start_date: NaiveDate,
	/// UTC date that the last event may take place (%Y-%m-%d)
    pub end_date: NaiveDate,
    /// List of days containing events for that day
    /// * Days are guaranteed to be sorted in chronological order
    pub event_days: Vec<EventDay>,
    /// Possible associated chat session for easy editing on frontend
    pub chat_session_id: Option<i32>,
    /// Title of itinerary, defaults to include location and date range
    pub title: String
}

/// A single day of events in an itinerary
#[derive(Debug, Serialize, Deserialize, ToSchema, ToResponse)]
pub struct EventDay {
	/// All the events taking place in the morning
	pub morning_events: Vec<Event>,
	/// All the events taking place around noon
    pub noon_events: Vec<Event>,
	/// All the events taking place in the afternoon
    pub afternoon_events: Vec<Event>,
	/// All the events taking place in the evening
    pub evening_events: Vec<Event>,
    /// The UTC date of this day within the range of itinerary start and end dates (%Y-%m-%d)
    pub date: NaiveDate
}

/// API route response for GET `/api/itinerary/saved`
#[derive(Debug, Serialize, Deserialize, ToSchema, ToResponse)]
pub struct SavedResponse {
	/// List of saved itineraries for the user.
    pub itineraries: Vec<Itinerary>
}

/// Response model from `/api/itinerary/save` endpoint
#[derive(Serialize, ToSchema, ToResponse)]
pub struct SaveResponse {
	/// id of the itinerary that was just saved
	/// * May be the same as the itinerary id passed in the request
	pub id: i32
}