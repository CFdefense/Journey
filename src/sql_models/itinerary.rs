use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Row model for the `itineraries` table.
#[derive(Debug, Serialize, Deserialize)]
pub struct ItineraryRow {
	/// Primary key
	pub id: i32,
	/// Owner account id (FK)
	pub account_id: Option<i32>,
	/// Start date for itinerary (UTC naive %Y-%m-%d)
	pub start_date: NaiveDate,
	/// End date for itinerary (UTC naive %Y-%m-%d)
	pub end_date: NaiveDate,
	/// Possible chat session to link to if this itinerary is edited
	pub chat_session_id: Option<i32>,
	/// Title of itinerary, defaults to include location and date range
	pub title: String,
}
