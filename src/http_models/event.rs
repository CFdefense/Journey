use chrono::{NaiveDate, NaiveDateTime};
use google_maps::places_new::Place;
use num_traits::ToPrimitive;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{ToResponse, ToSchema};

use crate::sql_models::{Period, event_list::EventListJoinRow};

/// A single event without context from an itinerary
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema, Default)]
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
			lat: value.lat.clone(),
			lng: value.lng.clone(),
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

#[cfg(not(tarpaulin_include))]
pub static REGEX_ST_ADDR: Lazy<Regex> =
	Lazy::new(|| Regex::new(r#"<span\s+class="street-address"\s*>([^<]*)</span>"#).unwrap());
#[cfg(not(tarpaulin_include))]
pub static REGEX_LOCALITY: Lazy<Regex> =
	Lazy::new(|| Regex::new(r#"<span\s+class="locality"\s*>([^<]*)</span>"#).unwrap());
#[cfg(not(tarpaulin_include))]
pub static REGEX_COUNTRY: Lazy<Regex> =
	Lazy::new(|| Regex::new(r#"<span\s+class="country-name"\s*>([^<]*)</span>"#).unwrap());
#[cfg(not(tarpaulin_include))]
pub static REGEX_POST_CODE: Lazy<Regex> =
	Lazy::new(|| Regex::new(r#"<span\s+class="postal-code"\s*>([^<]*)</span>"#).unwrap());

#[cfg(not(tarpaulin_include))]
impl From<&Place> for Event {
	fn from(value: &Place) -> Self {
		#[inline]
		fn extract(input: &str, re: &Regex) -> Option<String> {
			re.captures(input)
				.and_then(|caps| caps.get(1))
				.map(|m| m.as_str().to_string())
		}
		let empty = String::new();
		let input = value.adr_format_address.as_ref().unwrap_or(&empty).as_str();
		let street_address = extract(input, &REGEX_ST_ADDR);
		let city = extract(input, &REGEX_LOCALITY);
		let country = extract(input, &REGEX_COUNTRY);
		let postal_code = extract(input, &REGEX_POST_CODE)
			.map(|p| p.parse().ok())
			.unwrap_or(None);
		Self {
			id: -1,
			event_name: value
				.display_name
				.as_ref()
				.map(|n| n.to_string())
				.unwrap_or("Unnamed Event".to_string()),
			event_description: value.editorial_summary.as_ref().map(|n| n.to_string()),
			street_address,
			city,
			country,
			postal_code,
			lat: value.location.map(|l| l.latitude.to_f64()).unwrap_or(None),
			lng: value.location.map(|l| l.longitude.to_f64()).unwrap_or(None),
			event_type: value.primary_type.map(|t| t.to_string()),
			user_created: false,
			hard_start: None,
			hard_end: None,
			timezone: None,
			place_id: value.id.clone(),
			wheelchair_accessible_parking: value
				.accessibility_options
				.map(|a| a.wheelchair_accessible_parking)
				.unwrap_or(None),
			wheelchair_accessible_entrance: value
				.accessibility_options
				.map(|a| a.wheelchair_accessible_entrance)
				.unwrap_or(None),
			wheelchair_accessible_restroom: value
				.accessibility_options
				.map(|a| a.wheelchair_accessible_restroom)
				.unwrap_or(None),
			wheelchair_accessible_seating: value
				.accessibility_options
				.map(|a| a.wheelchair_accessible_seating)
				.unwrap_or(None),
			serves_vegetarian_food: value.serves_vegetarian_food,
			price_level: value.price_level.map(|p| p as i32),
			utc_offset_minutes: value.utc_offset_minutes,
			website_uri: value.website_uri.as_ref().map(|w| w.to_string()),
			types: Some(
				value
					.types
					.iter()
					.map(|t| t.to_string())
					.collect::<Vec<_>>()
					.join(","),
			),
			photo_name: if value.has_photos() {
				Some(value.photos[0].name.clone())
			} else {
				None
			},
			photo_width: if value.has_photos() {
				Some(value.photos[0].width_px as i32)
			} else {
				None
			},
			photo_height: if value.has_photos() {
				Some(value.photos[0].height_px as i32)
			} else {
				None
			},
			photo_author: if value.has_photos() && !value.photos[0].author_attributions.is_empty() {
				value.photos[0].author_attributions[0].display_name.clone()
			} else {
				None
			},
			photo_author_uri: if value.has_photos()
				&& !value.photos[0].author_attributions.is_empty()
			{
				value.photos[0].author_attributions[0]
					.uri
					.as_ref()
					.map(|u| u.to_string())
			} else {
				None
			},
			photo_author_photo_uri: if value.has_photos()
				&& !value.photos[0].author_attributions.is_empty()
			{
				value.photos[0].author_attributions[0]
					.photo_uri
					.as_ref()
					.map(|u| u.to_string())
			} else {
				None
			},
			weekday_descriptions: value
				.regular_opening_hours
				.as_ref()
				.map(|r| r.weekday_descriptions.join("\n")),
			secondary_hours_type: value
				.regular_opening_hours
				.as_ref()
				.map(|r| r.secondary_hours_type.map(|s| s as i32))
				.unwrap_or(None),
			next_open_time: value
				.regular_opening_hours
				.as_ref()
				.map(|r| {
					r.next_open_time.map(|t| {
						chrono::DateTime::parse_from_rfc3339(t.to_string().as_str())
							.map(|d| d.naive_utc())
							.ok()
					})
				})
				.unwrap_or(None)
				.unwrap_or(None),
			next_close_time: value
				.regular_opening_hours
				.as_ref()
				.map(|r| {
					r.next_close_time.map(|t| {
						chrono::DateTime::parse_from_rfc3339(t.to_string().as_str())
							.map(|d| d.naive_utc())
							.ok()
					})
				})
				.unwrap_or(None)
				.unwrap_or(None),
			open_now: value
				.regular_opening_hours
				.as_ref()
				.map(|r| r.open_now)
				.unwrap_or(None),
			periods: value
				.regular_opening_hours
				.as_ref()
				.map(|r| {
					r.periods
						.iter()
						.map(|p| Period {
							open_date: p
								.open
								.date
								.map(|d| {
									NaiveDate::from_ymd_opt(
										d.year() as i32,
										d.month() as u32,
										d.day() as u32,
									)
								})
								.unwrap_or(None),
							open_truncated: p.open.truncated,
							open_day: p.open.day as i32,
							open_hour: p.open.hour,
							open_minute: p.open.minute,
							close_date: p
								.close
								.as_ref()
								.map(|p| {
									p.date.map(|d| {
										NaiveDate::from_ymd_opt(
											d.year() as i32,
											d.month() as u32,
											d.day() as u32,
										)
									})
								})
								.unwrap_or(None)
								.unwrap_or(None),
							close_truncated: p.close.as_ref().map(|p| p.truncated).unwrap_or(None),
							close_day: p.close.as_ref().map(|p| p.day as i32),
							close_hour: p.close.as_ref().map(|p| p.hour),
							close_minute: p.close.as_ref().map(|p| p.minute),
						})
						.collect()
				})
				.unwrap_or(Vec::new()),
			special_days: value
				.regular_opening_hours
				.as_ref()
				.map(|r| {
					r.special_days
						.iter()
						.map(|d| {
							NaiveDate::from_ymd_opt(
								d.year()? as i32,
								d.month()? as u32,
								d.day()? as u32,
							)
						})
						.collect::<Option<Vec<_>>>()
				})
				.unwrap_or(None)
				.unwrap_or(Vec::new()),
			block_index: None,
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
	pub photo_name: Option<String>,
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
