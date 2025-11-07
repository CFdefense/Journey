use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::sql_models::event_list::EventListJoinRow;

/// A single event without context from an itinerary
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Event {
	/// Primary key
	pub id: i32,
	pub street_address: String,
	pub postal_code: i32,
	pub city: String,
	pub event_type: String,
	pub event_description: String,
	pub event_name: String,
	pub user_created: bool,
	pub account_id: Option<i32>,
	pub hard_start: Option<NaiveDateTime>,
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
			event_type: value.event_type.clone(),
			event_description: value.event_description.clone(),
			event_name: value.event_name.clone(),
			user_created: value.user_created.clone(),
			account_id: value.account_id.clone(),
			hard_start: value.hard_start.clone(),
			hard_end: value.hard_end.clone(),
		}
	}
}
