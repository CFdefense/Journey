/*
	src/agent/models/itinerary.rs
	File for Agent Itinerary Models
	Purpose:
		Store Agent Itinerary Models

*/
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
use serde::Deserialize;

/// A complete itinerary with event details
#[allow(dead_code)]
#[derive(Deserialize)]
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
	pub title: String,
	/// Events that are not assigned to any specific time slot
	pub unassigned_events: Vec<Event>,
}

/// A single day of events in an itinerary
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct EventDay {
	/// All the events taking place in the morning
	pub morning_events: Vec<Event>,
	/// All the events taking place in the afternoon
	pub afternoon_events: Vec<Event>,
	/// All the events taking place in the evening
	pub evening_events: Vec<Event>,
	/// The date of this day within the range of itinerary start and end dates (Destination's local timezone - %Y-%m-%d)
	pub date: NaiveDate,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Event {
	id: i32,
}
