use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::sql_models::TimeOfDay;

/// Row model for an inner join of `event_list` and `events` tables on chat session id.
/// - Represents one event for an itinerary.
#[derive(Debug, Serialize, Deserialize)]
pub struct EventListJoinRow {
	/// Primary key
	pub id: i32,
	/// Morning/Noon/Afternoon/Evening
	pub time_of_day: TimeOfDay,
	/// UTC date within itinerary date range (%Y-%m-%d)
	pub date: NaiveDate,
	/// Event address
	pub street_address: Option<String>,
	/// Event post code
	pub postal_code: Option<i32>,
	/// Event City
	pub city: Option<String>,
	/// Event Country
	pub country: Option<String>,
	/// Event type
	pub event_type: Option<String>,
	/// Event description
	pub event_description: Option<String>,
	/// Event name
	pub event_name: String,
	/// User-Created
	pub user_created: bool,
	/// Hard Start Time
	pub hard_start: Option<NaiveDateTime>,
	/// Hard End Time
	pub hard_end: Option<NaiveDateTime>,
	/// Timezone of hard start and hard end
	pub timezone: Option<String>,
	/// Index the event is in within the time block.
	/// Must be some to guarantee ordering
	pub block_index: Option<i32>,
}
