use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{ToResponse, ToSchema};

use crate::sql_models::event_list::EventListJoinRow;

/// A single event without context from an itinerary
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Event {
	/// Primary key
	pub id: i32,
	pub street_address: Option<String>,
	pub postal_code: Option<i32>,
	pub city: Option<String>,
	pub country: Option<String>,
	pub event_type: Option<String>,
	pub event_description: Option<String>,
	pub event_name: String,
	pub user_created: bool,
	/// UTC
	pub hard_start: Option<NaiveDateTime>,
	/// UTC
	pub hard_end: Option<NaiveDateTime>,
}

impl From<&EventListJoinRow> for Event {
	#[cfg(not(tarpaulin_include))]
	fn from(value: &EventListJoinRow) -> Self {
		Self {
			id: value.id,
			street_address: value.street_address.clone(),
			postal_code: value.postal_code,
			city: value.city.clone(),
			country: value.country.clone(),
			event_type: value.event_type.clone(),
			event_description: value.event_description.clone(),
			event_name: value.event_name.clone(),
			user_created: value.user_created.clone(),
			hard_start: value.hard_start.clone(),
			hard_end: value.hard_end.clone(),
		}
	}
}

/// A user-created event. It must have a name, and all other fields are optional.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UserEventRequest {
	/// If id is provided, it updates the user-event with that id. Otherwise it creates the event.
	pub id: Option<i32>,
	pub street_address: Option<String>,
	pub postal_code: Option<i32>,
	pub city: Option<String>,
	pub country: Option<String>,
	pub event_type: Option<String>,
	pub event_description: Option<String>,
	pub event_name: String,
	/// UTC
	pub hard_start: Option<NaiveDateTime>,
	/// UTC
	pub hard_end: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, ToSchema, ToResponse)]
pub struct UserEventResponse {
	pub id: i32,
}

/// A set of query filters to search for an event in the DB.
///
/// ## Example
/// If event_name is provided, it will query the DB with something like this:
/// ```sql
/// SELECT * FROM events
/// WHERE name LIKE $1
/// LIMIT 10;
/// ```
#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchEventRequest {
	/// Search where id=...
	pub id: Option<i32>,
	/// Search where street_address like ...
	pub street_address: Option<String>,
	/// Search where postal_code=...
	pub postal_code: Option<i32>,
	/// Search where city like ...
	pub city: Option<String>,
	/// Search where countr like ...
	pub country: Option<String>,
	/// Search where event_type like ...
	pub event_type: Option<String>,
	/// Search where event_description like ...
	pub event_description: Option<String>,
	/// Search where event_name like ...
	pub event_name: Option<String>,
	/// Search where hard_start < ...
	pub hard_start_before: Option<NaiveDateTime>,
	/// Search where hard_start > ...
	pub hard_start_after: Option<NaiveDateTime>,
	/// Search where hard_end < ...
	pub hard_end_before: Option<NaiveDateTime>,
	/// Search where hard_end > ...
	pub hard_end_after: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, ToSchema, ToResponse)]
pub struct SearchEventResponse {
	pub events: Vec<Event>,
}
