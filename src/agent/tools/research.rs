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
use std::time::Instant;
use tracing::{debug, info};

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
#[allow(dead_code)]
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
		let start_time = Instant::now();
		
		crate::tool_trace!(agent: "research", tool: "geocode_tool", status: "start");
		
		info!(
			target: "research_tools",
			tool = "geocode_tool",
			"Starting geocoding"
		);
		debug!(
			target: "research_tools",
			tool = "geocode_tool",
			input = %serde_json::to_string(&input).unwrap_or_else(|_| "invalid".to_string()),
			"Tool input"
		);
		
		// langchain-rust passes `action_input` as a STRING, but the LLM may:
		// - pass a plain text location like "Connecticut"
		// - pass a JSON string like "{\"location\":\"Connecticut\"}"
		// - or pass an object with `location` or `destination` fields.
		//
		// Normalize all of these into a single `location` string.
		let location: String = if input.is_string() {
			let raw = input.as_str().unwrap_or("").trim();

			if raw.is_empty() {
				return Err("Location is required".into());
			}

			if raw.starts_with('{') || raw.starts_with('[') {
				let v: Value = serde_json::from_str(raw)
					.unwrap_or_else(|_| json!({ "location": raw }));

				v.get("location")
					.or_else(|| v.get("destination"))
					.and_then(|v| v.as_str())
					.ok_or("Location or destination field should be a string")?
					.to_string()
			} else {
				raw.to_string()
			}
		} else {
			input
				.get("location")
				.or_else(|| input.get("destination"))
				.and_then(|v| v.as_str())
				.ok_or("Location or destination field should be a string")?
				.to_string()
		};
		
		debug!(
			target: "research_tools",
			tool = "geocode_tool",
			location = %location,
			"Geocoding location"
		);
		
		dotenvy::dotenv().map_err(|_| "Failed to load environment variables")?;
		let gm_api_key =
			std::env::var(GOOGLE_MAPS_API_KEY).map_err(|_| "GOOGLE_MAPS_API_KEY is not set")?;

		// use google maps api to get the address from the provided location and use geocoding to get its coordinates
		let gm_client = google_maps::Client::try_new(gm_api_key)
			.map_err(|_| "Failed to create client for Google Maps API")?;
		let geocode_res = gm_client
			.geocoding()
			.with_address(&location)
			.execute()
			.await?;
		if let Some(err) = geocode_res.error_message {
			let elapsed = start_time.elapsed();
			crate::tool_trace!(
				agent: "research", 
				tool: "geocode_tool", 
				status: "error",
				details: format!("{}ms - Geocoding API error: {}", elapsed.as_millis(), err)
			);
			return Err(format!(
				"Geocoding failed with status {} - {err}",
				geocode_res.status
			)
			.into());
		}
		if !matches!(geocode_res.status, google_maps::geocoding::Status::Ok) {
			let elapsed = start_time.elapsed();
			crate::tool_trace!(
				agent: "research", 
				tool: "geocode_tool", 
				status: "error",
				details: format!("{}ms - Bad status: {}", elapsed.as_millis(), geocode_res.status)
			);
			return Err(format!("Geocoding failed with status {}", geocode_res.status).into());
		}
		if geocode_res.results.is_empty() {
			let elapsed = start_time.elapsed();
			crate::tool_trace!(
				agent: "research", 
				tool: "geocode_tool", 
				status: "error",
				details: format!("{}ms - No results", elapsed.as_millis())
			);
			return Err(format!("Geocoding could not get coordinates for {location}").into());
		}

	let result = json!({
		"lat": geocode_res.results[0].geometry.location.lat,
		"lng": geocode_res.results[0].geometry.location.lng
	});
	
	let elapsed = start_time.elapsed();
	
	info!(
		target: "research_tools",
		tool = "geocode_tool",
		elapsed_ms = elapsed.as_millis() as u64,
		lat = %geocode_res.results[0].geometry.location.lat,
		lng = %geocode_res.results[0].geometry.location.lng,
		"Geocoding completed successfully"
	);
	debug!(
		target: "research_tools",
		tool = "geocode_tool",
		result = %result.to_string(),
		"Tool output"
	);
	
	crate::tool_trace!(
		agent: "research", 
		tool: "geocode_tool", 
		status: "success",
		details: format!("{}ms", elapsed.as_millis())
	);
	
	Ok(result.to_string())
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
		"A tool that uses Google Maps Nearby Search to fetch a list of places in a given area with certain input criteria. The resulting events are inserted or updated in the database and their IDs are returned. Returns a JSON object with 'event_ids' (array of integer IDs) and 'count' (number of events found)."
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
		let start_time = Instant::now();
		
		crate::tool_trace!(agent: "research", tool: "nearby_search_tool", status: "start");
		
		info!(
			target: "research_tools",
			tool = "nearby_search_tool",
			"Starting nearby search"
		);
		debug!(
			target: "research_tools",
			tool = "nearby_search_tool",
			input = %serde_json::to_string(&input).unwrap_or_else(|_| "invalid".to_string()),
			"Tool input"
		);

		// langchain-rust usually passes `action_input` as a STRING. Normalize it:
		// - If it's a JSON string, parse it into an object.
		// - If it's a plain string, treat it as a "lat,lng" location string.
		let parsed_input: Value = if input.is_string() {
			let raw = input.as_str().unwrap_or("").trim();
			if raw.starts_with('{') || raw.starts_with('[') {
				serde_json::from_str(raw).unwrap_or_else(|_| json!({ "location": raw }))
			} else {
				json!({ "location": raw })
			}
		} else {
			input
		};
		
		// Handle multiple input formats:
		// 1. Combined "location" field as "lat,lng" string
		// 2. "location" field as object with lat/lng properties
		// 3. Separate lat/lng fields (as numbers or strings)
		let (lat, lng) = if let Some(location_val) = parsed_input.get("location") {
			debug!(
				target: "research_tools",
				tool = "nearby_search_tool",
				location_val = %location_val,
				"Processing location field"
			);

			// Check if location is an object with lat/lng properties
			if location_val.is_object() {
				let lat = if let Some(f) = location_val.get("lat").and_then(|v| v.as_f64()) {
					f
				} else if let Some(s) = location_val.get("lat").and_then(|v| v.as_str()) {
					s.parse::<f64>()
						.map_err(|e| format!("lat in location object should be a valid number: {}", e))?
				} else {
					return Err("location object must have a 'lat' field as a number or string".into());
				};

				let lng = if let Some(f) = location_val.get("lng").and_then(|v| v.as_f64()) {
					f
				} else if let Some(s) = location_val.get("lng").and_then(|v| v.as_str()) {
					s.parse::<f64>()
						.map_err(|e| format!("lng in location object should be a valid number: {}", e))?
				} else {
					return Err("location object must have a 'lng' field as a number or string".into());
				};

				(lat, lng)
			} else if let Some(location_str) = location_val.as_str() {
				// Parse from "location" field as "lat,lng" string
				debug!(
					target: "research_tools",
					tool = "nearby_search_tool",
					location = %location_str,
					"Parsing location string"
				);

				let parts: Vec<&str> = location_str.split(',').collect();
				if parts.len() != 2 {
					return Err("location string should be in format 'lat,lng'".into());
				}
				let lat = parts[0].trim().parse::<f64>()
					.map_err(|e| format!("Invalid latitude in location string: {}", e))?;
				let lng = parts[1].trim().parse::<f64>()
					.map_err(|e| format!("Invalid longitude in location string: {}", e))?;
				(lat, lng)
			} else {
				return Err("location field should be either a string in format 'lat,lng' or an object with lat/lng properties".into());
			}
		} else {
			// Parse from separate lat/lng fields
			debug!(
				target: "research_tools",
				tool = "nearby_search_tool",
				"No location field, looking for separate lat/lng fields"
			);

			let lat = if let Some(f) = parsed_input.get("lat").and_then(|v| v.as_f64()) {
				f
			} else if let Some(s) = parsed_input.get("lat").and_then(|v| v.as_str()) {
				s.parse::<f64>()
					.map_err(|e| format!("lat should be a valid number: {}", e))?
			} else {
				return Err("lat should be a 64-bit floating point number".into());
			};

			let lng = if let Some(f) = parsed_input.get("lng").and_then(|v| v.as_f64()) {
				f
			} else if let Some(s) = parsed_input.get("lng").and_then(|v| v.as_str()) {
				s.parse::<f64>()
					.map_err(|e| format!("lng should be a valid number: {}", e))?
			} else {
				return Err("lng should be a 64-bit floating point number".into());
			};
			(lat, lng)
		};

		debug!(
			target: "research_tools",
			tool = "nearby_search_tool",
			lat = lat,
			lng = lng,
			"Search coordinates"
		);

		const INCLUDE_TYPES_ERR: &str = "includedTypes should be an array of strings";
		const EXCLUDE_TYPES_ERR: &str = "excludedTypes should be an array of strings";
		let included_types = if !parsed_input["includedTypes"].is_null() {
			parsed_input["includedTypes"]
				.as_array()
				.ok_or(INCLUDE_TYPES_ERR)?
				.iter()
				.map(|v| v.as_str().ok_or(INCLUDE_TYPES_ERR))
				.collect::<Result<_, _>>()
				.map_err(|_| INCLUDE_TYPES_ERR)?
		} else {
			Vec::new()
		};
		let excluded_types = if !parsed_input["excludedTypes"].is_null() {
			parsed_input["excludedTypes"]
				.as_array()
				.ok_or(EXCLUDE_TYPES_ERR)?
				.iter()
				.map(|v| v.as_str().ok_or(EXCLUDE_TYPES_ERR))
				.collect::<Result<_, _>>()
				.map_err(|_| EXCLUDE_TYPES_ERR)?
		} else {
			Vec::new()
		};

		debug!(
			target: "research_tools",
			tool = "nearby_search_tool",
			included_types_count = included_types.len(),
			excluded_types_count = excluded_types.len(),
			"Place type filters"
		);

		dotenvy::dotenv().map_err(|_| "Failed to load environment variables")?;
		let gm_api_key =
			std::env::var(GOOGLE_MAPS_API_KEY).map_err(|_| "GOOGLE_MAPS_API_KEY is not set")?;

		// use google maps api to get nearby places
		let gm_client = google_maps::Client::try_new(gm_api_key)
			.map_err(|_| "Failed to create client for Google Maps API")?;

		info!(
			target: "research_tools",
			tool = "nearby_search_tool",
			"Calling Google Maps API"
		);

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
			let elapsed = start_time.elapsed();
			crate::tool_trace!(
				agent: "research", 
				tool: "nearby_search_tool", 
				status: "error",
				details: format!("{}ms - API error: {}", elapsed.as_millis(), err)
			);
			info!(
				target: "research_tools",
				tool = "nearby_search_tool",
				elapsed_ms = elapsed.as_millis() as u64,
				error = %err,
				"Nearby search API error"
			);
			return Err(format!("Nearby Search failed - {err}").into());
		}
		let places = search_res.places();
		if places.is_empty() {
			let elapsed = start_time.elapsed();
			crate::tool_trace!(
				agent: "research", 
				tool: "nearby_search_tool", 
				status: "error",
				details: format!("{}ms - No places found", elapsed.as_millis())
			);
			info!(
				target: "research_tools",
				tool = "nearby_search_tool",
				elapsed_ms = elapsed.as_millis() as u64,
				"No places found in nearby search"
			);
			return Err(format!("Nearby Search returned an empty array of places").into());
		}

		info!(
			target: "research_tools",
			tool = "nearby_search_tool",
			places_count = places.len(),
			"Found places from Google Maps"
		);

	let events: Vec<Event> = places.into_iter().map(|p| Event::from(p)).collect();

	info!(
		target: "research_tools",
		tool = "nearby_search_tool",
		events_to_insert = events.len(),
		"Inserting/updating events in database"
	);

	// Define a struct to capture the RETURNING clause results
	struct EventInsertResult {
		id: i32,
		event_name: String,
	}

	// Use query! macro for compile-time type checking
	// Insert events one by one to use the type-safe macro
	let mut results: Vec<EventInsertResult> = Vec::with_capacity(events.len());
	
	for ev in events.iter() {
		let result = sqlx::query!(
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
			VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36)
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
			RETURNING id, event_name
			"#,
			&ev.event_name,
			ev.event_description.as_ref(),
			ev.street_address.as_ref(),
			ev.city.as_ref(),
			ev.country.as_ref(),
			ev.postal_code,
			ev.lat,
			ev.lng,
			ev.event_type.as_ref(),
			ev.user_created,
			ev.hard_start,
			ev.hard_end,
			ev.timezone.as_ref(),
			ev.place_id.as_ref(),
			ev.wheelchair_accessible_parking,
			ev.wheelchair_accessible_entrance,
			ev.wheelchair_accessible_restroom,
			ev.wheelchair_accessible_seating,
			ev.serves_vegetarian_food,
			ev.price_level,
			ev.utc_offset_minutes,
			ev.website_uri.as_ref(),
			ev.types.as_ref(),
			ev.photo_name.as_ref(),
			ev.photo_width,
			ev.photo_height,
			ev.photo_author.as_ref(),
			ev.photo_author_uri.as_ref(),
			ev.photo_author_photo_uri.as_ref(),
			ev.weekday_descriptions.as_ref(),
			ev.secondary_hours_type,
			ev.next_open_time,
			ev.next_close_time,
			ev.open_now,
			&ev.periods as _,
			&ev.special_days as _,
		)
		.fetch_one(&self.db)
		.await?;
		
		results.push(EventInsertResult {
			id: result.id,
			event_name: result.event_name,
		});
	}

	let elapsed = start_time.elapsed();
	
	// Extract event IDs and names for the response and debugging
	let event_ids: Vec<i32> = results.iter().map(|r| r.id).collect();
	let event_names: Vec<String> = results.iter().map(|r| r.event_name.clone()).collect();
	
	// Return only the IDs to keep the context window clean
	let result = json!({
		"event_ids": event_ids,
		"count": event_ids.len()
	});
	
	info!(
		target: "research_tools",
		tool = "nearby_search_tool",
		elapsed_ms = elapsed.as_millis() as u64,
		events_count = event_ids.len(),
		event_ids = ?event_ids,
		"Nearby search completed successfully"
	);
	debug!(
		target: "research_tools",
		tool = "nearby_search_tool",
		events_sample = %serde_json::to_string(&results.iter().take(3).map(|r| json!({"id": r.id, "name": &r.event_name})).collect::<Vec<_>>()).unwrap_or_else(|_| "error".to_string()),
		"Sample of events (first 3)"
	);
	
	crate::tool_trace!(
		agent: "research", 
		tool: "nearby_search_tool", 
		status: "success",
		details: format!("{}ms - {} events - [{}]", elapsed.as_millis(), event_ids.len(), event_names.join(", "))
	);
		
		Ok(result.to_string())
	}
}
