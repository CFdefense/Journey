use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{ToResponse, ToSchema};

use crate::sql_models::{Period, event_list::EventListJoinRow};

/// A single event without context from an itinerary
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Default)]
pub struct Event {
	/// Primary key
	pub id: i32,
	pub event_name: String,
	pub event_description: Option<String>,
	pub street_address: Option<String>,
	pub city: Option<String>,
	pub country: Option<String>,
	pub postal_code: Option<i32>,
	pub coords: Option<String>,
	pub event_type: Option<String>,
	pub user_created: bool,
	pub hard_start: Option<NaiveDateTime>,
	pub hard_end: Option<NaiveDateTime>,
	/// Timezone of hard start and hard end
	pub timezone: Option<String>,
	pub place_id: Option<String>,
	pub wheelchair_accessible_parking: Option<bool>,
	pub wheelchair_accessible_entrance: Option<bool>,
	pub wheelchair_accessible_restroom: Option<bool>,
	pub wheelchair_accessible_seating: Option<bool>,
	pub serves_vegetarian_food: Option<bool>,
	pub price_level: Option<i32>,
	pub utc_offset_minutes: Option<i32>,
	pub website_uri: Option<String>,
	pub types: Option<String>,
	pub photo_name: Option<String>,
	pub photo_width: Option<i32>,
	pub photo_height: Option<i32>,
	pub photo_author: Option<String>,
	pub photo_author_uri: Option<String>,
	pub photo_author_photo_uri: Option<String>,
	pub weekday_descriptions: Option<String>,
	pub secondary_hours_type: Option<i32>,
	pub next_open_time: Option<NaiveDateTime>,
	pub next_close_time: Option<NaiveDateTime>,
	pub open_now: Option<bool>,
	pub periods: Vec<Period>,
	pub special_days: Vec<NaiveDate>,
	/// Must be some to guarantee ordering
	pub block_index: Option<i32>,
}

impl From<&EventListJoinRow> for Event {
	#[cfg(not(tarpaulin_include))]
	fn from(value: &EventListJoinRow) -> Self {
		Self {
			id: value.id,
			event_name: value.event_name.clone(),
			event_description: value.event_description.clone(),
			street_address: value.street_address.clone(),
			city: value.city.clone(),
			country: value.country.clone(),
			postal_code: value.postal_code,
			coords: value.coords.clone(),
			event_type: value.event_type.clone(),
			user_created: value.user_created.clone(),
			hard_start: value.hard_start.clone(),
			hard_end: value.hard_end.clone(),
			timezone: value.timezone.clone(),
			place_id: value.place_id.clone(),
			wheelchair_accessible_parking: value.wheelchair_accessible_parking.clone(),
			wheelchair_accessible_entrance: value.wheelchair_accessible_entrance.clone(),
			wheelchair_accessible_restroom: value.wheelchair_accessible_restroom.clone(),
			wheelchair_accessible_seating: value.wheelchair_accessible_seating.clone(),
			serves_vegetarian_food: value.serves_vegetarian_food.clone(),
			price_level: value.price_level.clone(),
			utc_offset_minutes: value.utc_offset_minutes.clone(),
			website_uri: value.website_uri.clone(),
			types: value.types.clone(),
			photo_name: value.photo_name.clone(),
			photo_width: value.photo_width.clone(),
			photo_height: value.photo_height.clone(),
			photo_author: value.photo_author.clone(),
			photo_author_uri: value.photo_author_uri.clone(),
			photo_author_photo_uri: value.photo_author_photo_uri.clone(),
			weekday_descriptions: value.weekday_descriptions.clone(),
			secondary_hours_type: value.secondary_hours_type.clone(),
			next_open_time: value.next_open_time.clone(),
			next_close_time: value.next_close_time.clone(),
			open_now: value.open_now.clone(),
			periods: value.periods.clone(),
			special_days: value.special_days.clone(),
			block_index: value.block_index,
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
	pub hard_start: Option<NaiveDateTime>,
	pub hard_end: Option<NaiveDateTime>,
	/// Timezone of hard start and hard end
	pub timezone: Option<String>,
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
#[derive(Debug, Deserialize, ToSchema, Default, Clone)]
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
	/// Search where timezone like ...
	pub timezone: Option<String>,
}

#[derive(Debug, Serialize, ToSchema, ToResponse)]
pub struct SearchEventResponse {
	pub events: Vec<Event>,
}
