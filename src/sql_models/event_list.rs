use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::sql_models::{Period, TimeOfDay};

/// Row model for an inner join of `event_list` and `events` tables on chat session id.
/// - Represents one event for an itinerary.
#[derive(Debug, Serialize, Deserialize)]
pub struct EventListJoinRow {
	/// Primary key
	pub id: i32,
	/// Event name
	pub event_name: String,
	/// Event description
	pub event_description: Option<String>,
	/// Event address
	pub street_address: Option<String>,
	/// Event City
	pub city: Option<String>,
	/// Event Country
	pub country: Option<String>,
	/// Event post code
	pub postal_code: Option<i32>,
	/// Location coordinates
	pub lat: Option<f64>,
	pub lng: Option<f64>,
	/// Event type
	pub event_type: Option<String>,
	/// User-Created
	pub user_created: bool,
	/// Hard Start Time
	pub hard_start: Option<NaiveDateTime>,
	/// Hard End Time
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
	/// Morning/Noon/Afternoon/Evening
	pub time_of_day: TimeOfDay,
	/// UTC date within itinerary date range (%Y-%m-%d)
	pub date: NaiveDate,
	/// Index the event is in within the time block.
	/// Must be some to guarantee ordering
	pub block_index: Option<i32>,
}
