use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;

use crate::sql_models::Period;

/// A subset of [crate::http_models::event::Event] which only contains fields that the LLM might need for context.
#[derive(Deserialize)]
pub struct Event {
	/// Primary key
	pub id: i32,
	pub event_name: String,
	pub event_description: Option<String>,
	pub street_address: Option<String>,
	pub city: Option<String>,
	pub country: Option<String>,
	pub postal_code: Option<i32>,
	pub lat: Option<f64>,
	pub lng: Option<f64>,
	pub event_type: Option<String>,
	pub hard_start: Option<NaiveDateTime>,
	pub hard_end: Option<NaiveDateTime>,
	/// Timezone of hard start and hard end
	pub timezone: Option<String>,
	pub wheelchair_accessible_parking: Option<bool>,
	pub wheelchair_accessible_entrance: Option<bool>,
	pub wheelchair_accessible_restroom: Option<bool>,
	pub wheelchair_accessible_seating: Option<bool>,
	pub serves_vegetarian_food: Option<bool>,
	pub price_level: Option<i32>,
	pub utc_offset_minutes: Option<i32>,
	pub types: Option<String>,
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
