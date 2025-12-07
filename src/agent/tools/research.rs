/*
 * src/agent/tools/research.rs
 *
 * File for Reseearch Agent Tools
 *
 * Purpose:
 *   Store Research Agent Tools
 */

use async_trait::async_trait;
use google_maps::places_new::{Field, FieldMask, PlaceType};
use langchain_rust::tools::Tool;
use serde::{Deserialize, de::IntoDeserializer};
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use std::{error::Error, sync::Arc};

use crate::{global::GOOGLE_MAPS_API_KEY, http_models::event::Event};

pub fn research_tools(db: PgPool) -> [Arc<dyn Tool>; 2] {
	[
		Arc::new(GeocodeTool),
		// Arc::new(QueryDbEventsTool { db: db.clone() }),
		Arc::new(NearbySearchTool { db }),
	]
}

/// This tool takes an address and converts it into coordinates using Google Maps Geocoding API.
#[derive(Clone)]
struct GeocodeTool;

/// This tool queries the DB for events that may be relevant to the itinerary being generated.
#[derive(Clone)]
struct QueryDbEventsTool {
	pub db: PgPool,
}

/// This tool uses Google Maps Nearby Search to fetch a list of places in a given area with certain input criteria.
/// The resulting events are inserted or updated in the database.
#[derive(Clone)]
struct NearbySearchTool {
	pub db: PgPool,
}

#[async_trait]
impl Tool for GeocodeTool {
	fn name(&self) -> String {
		"geocode_tool".to_string()
	}

	fn description(&self) -> String {
		"A tool that takes an address or location and converts it into coordinates using Google Maps Geocoding API."
            .to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {
				"location": {
					"type": "string",
					"description": "The street address that you want to geocode, in the format used by the national postal service of the country concerned. Additional address elements such as business names and unit, suite or floor numbers hould be avoided. The street will likely not always be provided, but city should almost always be expected."
				}
			},
			"required": ["location"]
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let location = input["location"]
			.as_str()
			.ok_or("Location should be a string")?;
		dotenvy::dotenv().map_err(|_| "Failed to load environment variables")?;
		let gm_api_key =
			std::env::var(GOOGLE_MAPS_API_KEY).map_err(|_| "GOOGLE_MAPS_API_KEY is not set")?;

		// use google maps api to get the address from the provided location and use geocoding to get its coordinates
		let gm_client = google_maps::Client::try_new(gm_api_key)
			.map_err(|_| "Failed to create client for Google Maps API")?;
		let geocode_res = gm_client
			.geocoding()
			.with_address(location)
			.execute()
			.await?;
		if let Some(err) = geocode_res.error_message {
			return Err(format!(
				"Geocoding failed with status {} - {err}",
				geocode_res.status
			)
			.into());
		}
		if !matches!(geocode_res.status, google_maps::geocoding::Status::Ok) {
			return Err(format!("Geocoding failed with status {}", geocode_res.status).into());
		}
		if geocode_res.results.is_empty() {
			return Err(format!("Geocoding could not get coordinates for {location}").into());
		}

		Ok(json!({
			"lat": geocode_res.results[0].geometry.location.lat,
			"lng": geocode_res.results[0].geometry.location.lng
		})
		.to_string())
	}
}

#[async_trait]
impl Tool for QueryDbEventsTool {
	fn name(&self) -> String {
		"query_db_events_tool".to_string()
	}

	fn description(&self) -> String {
		"A tool that queries the DB for events that may be relevant to the itinerary being generated."
            .to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {
				"location": {
					"type": "string",
					"description": "A location in the world from the user's prompt; ideally a city name or postal code, but could be anything that indicates a place like a street address or country."
				},
				"keywords": {
					"type": "array",
					"description": "An array of keywords from the user's prompt that can be used to search for relevant events.",
					"items": {"type": "string"}
				}
			},
			"required": ["location"]
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		// TODO create filters and query the db for possibly relevant events

		let name = input["name"].as_str().ok_or("Name should be a string")?;
		Ok(format!("Hello, {}! Welcome to our AI assistant.", name))
	}
}

#[async_trait]
impl<'db> Tool for NearbySearchTool {
	fn name(&self) -> String {
		"nearby_search_tool".to_string()
	}

	fn description(&self) -> String {
		"A tool that uses Google Maps Nearby Search to fetch a list of places in a given area with certain input criteria. The resulting events are inserted or updated in the database."
            .to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {
				"lat": {
					"type": "number",
					"description": "The lattitude of the target location."
				},
				"lng": {
					"type": "number",
					"description": "The longitude of the target location."
				},
				"includedTypes": {
					"type": "array",
					"description": "An array of places types to include.",
					"items": {"type": "string"}
				},
				"excludedTypes": {
					"type": "array",
					"description": "An array of places types to exclude.",
					"items": {"type": "string"}
				}
			},
			"required": ["lat", "lng"]
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let lat = input["lat"]
			.as_f64()
			.ok_or("lat should be a 64-bit floating point number")?;
		let lng = input["lng"]
			.as_f64()
			.ok_or("lng should be a 64-bit floating point number")?;

		const INCLUDE_TYPES_ERR: &str = "includedTypes should be an array of strings";
		const EXCLUDE_TYPES_ERR: &str = "excludedTypes should be an array of strings";
		let included_types = if !input["includedTypes"].is_null() {
			input["includedTypes"]
				.as_array()
				.ok_or(INCLUDE_TYPES_ERR)?
				.iter()
				.map(|v| v.as_str().ok_or(INCLUDE_TYPES_ERR))
				.collect::<Result<_, _>>()
				.map_err(|_| INCLUDE_TYPES_ERR)?
		} else {
			Vec::new()
		};
		let excluded_types = if !input["excludedTypes"].is_null() {
			input["excludedTypes"]
				.as_array()
				.ok_or(EXCLUDE_TYPES_ERR)?
				.iter()
				.map(|v| v.as_str().ok_or(EXCLUDE_TYPES_ERR))
				.collect::<Result<_, _>>()
				.map_err(|_| EXCLUDE_TYPES_ERR)?
		} else {
			Vec::new()
		};

		dotenvy::dotenv().map_err(|_| "Failed to load environment variables")?;
		let gm_api_key =
			std::env::var(GOOGLE_MAPS_API_KEY).map_err(|_| "GOOGLE_MAPS_API_KEY is not set")?;

		// use google maps api to get nearby places
		let gm_client = google_maps::Client::try_new(gm_api_key)
			.map_err(|_| "Failed to create client for Google Maps API")?;

		let search_res = gm_client
			.nearby_search((lat, lng, 50_000.))?
			.field_mask(FieldMask::Specific(vec![
				Field::PlacesAccessibilityOptions,
				Field::PlacesAdrFormatAddress,
				Field::PlacesDisplayName,
				Field::PlacesId,
				Field::PlacesPhotos,
				Field::PlacesUtcOffsetMinutes,
				Field::PlacesPriceLevel,
				Field::PlacesRegularOpeningHours,
				Field::PlacesWebsiteUri,
				Field::PlacesServesVegetarianFood,
				Field::PlacesTypes,
				Field::PlacesPrimaryType,
				Field::PlacesEditorialSummary,
			]))
			// pray to our lord and savior Terry Davis that this works
			.included_types(
				included_types
					.iter()
					.map(|t| {
						PlaceType::deserialize(t.into_deserializer()).map_err(
							|_: serde::de::value::Error| "Could not deserialize place type within includedTypes array",
						)
					})
					.collect::<Result<Vec<_>, _>>()?,
			)
			.excluded_types(
				excluded_types
					.iter()
					.map(|t| {
						PlaceType::deserialize(t.into_deserializer()).map_err(
							|_: serde::de::value::Error| "Could not deserialize place type within excludedTypes array",
						)
					})
					.collect::<Result<Vec<_>, _>>()?,
			)
			.execute()
			.await?;

		if let Some(err) = search_res.error() {
			return Err(format!("Nearby Search failed - {err}").into());
		}
		let places = search_res.places();
		if places.is_empty() {
			return Err(format!("Nearby Search returned an empty array of places").into());
		}

		let mut events: Vec<Event> = places.into_iter().map(|p| Event::from(p)).collect();

		/// ## NOTICE
		/// If you change the fields in the events table or the [Event] struct,
		/// you must make sure the EVENT_FIELD_COUNT is set to the number of fields
		/// that need to be inserted.
		const EVENT_FIELD_COUNT: usize = 36;
		let mut placeholders = String::new();
		for r in 0..events.len() {
			if r > 0 {
				placeholders.push(',');
			}
			placeholders.push('(');
			for c in 0..EVENT_FIELD_COUNT {
				if c > 0 {
					placeholders.push(',');
				}
				placeholders.push_str(&format!("${}", r * EVENT_FIELD_COUNT + c + 1));
			}
			placeholders.push(')');
		}

		let sql = format!(
			r#"
			INSERT INTO events (
				event_name,
				event_description,
				street_address,
				city,
				country,
				postal_code,
				lat,
				lng,
				event_type,
				user_created,
				hard_start,
				hard_end,
				timezone,
				place_id,
				wheelchair_accessible_parking,
				wheelchair_accessible_entrance,
				wheelchair_accessible_restroom,
				wheelchair_accessible_seating,
				serves_vegetarian_food,
				price_level,
				utc_offset_minutes,
				website_uri,
				types,
				photo_name,
				photo_width,
				photo_height,
				photo_author,
				photo_author_uri,
				photo_author_photo_uri,
				weekday_descriptions,
				secondary_hours_type,
				next_open_time,
				next_close_time,
				open_now,
				periods,
				special_days
			)
			VALUES {}
			ON CONFLICT (place_id) DO UPDATE SET
				event_name = EXCLUDED.event_name,
				event_description = EXCLUDED.event_description,
				street_address = EXCLUDED.street_address,
				city = EXCLUDED.city,
				country = EXCLUDED.country,
				postal_code = EXCLUDED.postal_code,
				lat = EXCLUDED.lat,
				lng = EXCLUDED.lng,
				event_type = EXCLUDED.event_type,
				user_created = EXCLUDED.user_created,
				hard_start = EXCLUDED.hard_start,
				hard_end = EXCLUDED.hard_end,
				timezone = EXCLUDED.timezone,
				wheelchair_accessible_parking = EXCLUDED.wheelchair_accessible_parking,
				wheelchair_accessible_entrance = EXCLUDED.wheelchair_accessible_entrance,
				wheelchair_accessible_restroom = EXCLUDED.wheelchair_accessible_restroom,
				wheelchair_accessible_seating = EXCLUDED.wheelchair_accessible_seating,
				serves_vegetarian_food = EXCLUDED.serves_vegetarian_food,
				price_level = EXCLUDED.price_level,
				utc_offset_minutes = EXCLUDED.utc_offset_minutes,
				website_uri = EXCLUDED.website_uri,
				types = EXCLUDED.types,
				photo_name = EXCLUDED.photo_name,
				photo_width = EXCLUDED.photo_width,
				photo_height = EXCLUDED.photo_height,
				photo_author = EXCLUDED.photo_author,
				photo_author_uri = EXCLUDED.photo_author_uri,
				photo_author_photo_uri = EXCLUDED.photo_author_photo_uri,
				weekday_descriptions = EXCLUDED.weekday_descriptions,
				secondary_hours_type = EXCLUDED.secondary_hours_type,
				next_open_time = EXCLUDED.next_open_time,
				next_close_time = EXCLUDED.next_close_time,
				open_now = EXCLUDED.open_now,
				periods = EXCLUDED.periods,
				special_days = EXCLUDED.special_days
			RETURNING id, place_id, event_name;
			"#,
			placeholders
		);

		let mut query = sqlx::query(&sql);

		for ev in events.iter() {
			query = query
				.bind(&ev.event_name)
				.bind(&ev.event_description)
				.bind(&ev.street_address)
				.bind(&ev.city)
				.bind(&ev.country)
				.bind(ev.postal_code)
				.bind(ev.lat)
				.bind(ev.lng)
				.bind(&ev.event_type)
				.bind(ev.user_created)
				.bind(ev.hard_start)
				.bind(ev.hard_end)
				.bind(&ev.timezone)
				.bind(&ev.place_id) // conflict target
				.bind(ev.wheelchair_accessible_parking)
				.bind(ev.wheelchair_accessible_entrance)
				.bind(ev.wheelchair_accessible_restroom)
				.bind(ev.wheelchair_accessible_seating)
				.bind(ev.serves_vegetarian_food)
				.bind(ev.price_level)
				.bind(ev.utc_offset_minutes)
				.bind(&ev.website_uri)
				.bind(&ev.types)
				.bind(&ev.photo_name)
				.bind(ev.photo_width)
				.bind(ev.photo_height)
				.bind(&ev.photo_author)
				.bind(&ev.photo_author_uri)
				.bind(&ev.photo_author_photo_uri)
				.bind(&ev.weekday_descriptions)
				.bind(ev.secondary_hours_type)
				.bind(ev.next_open_time)
				.bind(ev.next_close_time)
				.bind(ev.open_now)
				.bind(&ev.periods)
				.bind(&ev.special_days);
		}

		// Event ID is set to -1 initially and assigned when it's inserted into the db.
		// We need to return the new event ID and update the events vector.
		// The best way I can think of is to just find the event with the same place ID
		// and fallback to the event with the same event name. If there are duplicate names
		// or something, then it might cause bugs, but they should be rare and all it does is tell
		// the LLM that some event IDs are -1 or the wrong ID.
		for row in query.fetch_all(&self.db).await?.into_iter() {
			let id: i32 = row.get("id");
			let mut ev = None;
			if let Some(place_id) = row.get::<Option<String>, _>("place_id") {
				ev = events.iter_mut().find(|ev| {
					ev.place_id
						.as_ref()
						.and_then(|i| if *i == place_id { Some(()) } else { None })
						.is_some()
				});
			}

			if ev.is_none() {
				let name: String = row.get("event_name");
				ev = events.iter_mut().find(|ev| ev.event_name == name);
			}

			if let Some(event) = ev {
				event.id = id;
			}
		}

		Ok(json!(events).to_string())
	}
}
