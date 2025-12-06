/*
 * src/agent/tools/research.rs
 *
 * File for Reseearch Agent Tools
 *
 * Purpose:
 *   Store Research Agent Tools
 */

use async_trait::async_trait;
use google_maps::places_new::{Circle, Field, FieldMask, LatLng, LocationRestriction, PlaceType};
use langchain_rust::tools::Tool;
use serde::{Deserialize, Serialize, de::IntoDeserializer};
use serde_json::{Value, json};
use std::error::Error;

use crate::{global::GOOGLE_MAPS_API_KEY, http_models::event::Event};

/// This tool takes an address and converts it into coordinates using Google Maps Geocoding API.
#[derive(Clone)]
pub struct GeocodeTool;

/// This tool queries the DB for events that may be relevant to the itinerary being generated.
#[derive(Clone)]
pub struct QueryDbEventsTool;

/// This tool uses Google Maps Nearby Search to fetch a list of places in a given area with certain input criteria.
#[derive(Clone)]
pub struct NearbySearchTool;

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
		let gm_api_key = std::env::var(GOOGLE_MAPS_API_KEY)
			.map_err(|_| "GOOGLE_MAPS_API_KEY is not set")?;

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
					"description": "An array of keywords from the user's prompt that can be used to search for relevant events."
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
impl Tool for NearbySearchTool {
	fn name(&self) -> String {
		"nearby_search_tool".to_string()
	}

	fn description(&self) -> String {
		"A tool that uses Google Maps Nearby Search to fetch a list of places in a given area with certain input criteria."
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
					"description": "An array of places types to include."
				},
				"excludedTypes": {
					"type": "array",
					"description": "An array of places types to exclude."
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
				.collect::<Result<_,_>>()
				.map_err(|_|INCLUDE_TYPES_ERR)?
		} else {
			Vec::new()
		};
		let excluded_types = if !input["excludedTypes"].is_null() {
			input["excludedTypes"]
				.as_array()
				.ok_or(EXCLUDE_TYPES_ERR)?
				.iter()
				.map(|v| v.as_str().ok_or(EXCLUDE_TYPES_ERR))
				.collect::<Result<_,_>>()
				.map_err(|_|EXCLUDE_TYPES_ERR)?
		} else {
			Vec::new()
		};

		dotenvy::dotenv().map_err(|_| "Failed to load environment variables")?;
		let gm_api_key = std::env::var(GOOGLE_MAPS_API_KEY)
				.map_err(|_| "GOOGLE_MAPS_API_KEY is not set")?;

		// use google maps api to get nearby places
		let gm_client = google_maps::Client::try_new(gm_api_key)
			.map_err(|_| "Failed to create client for Google Maps API")?;

		let search_res = gm_client.nearby_search((lat, lng, 50_000.))?
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
					.map(|t| PlaceType::deserialize(t.into_deserializer())
						.map_err(|_: serde::de::value::Error|"Could not deserialize place type within includedTypes array")
					)
					.collect::<Result<Vec<_>,_>>()?
			)
			.excluded_types(
				excluded_types
					.iter()
					.map(|t| PlaceType::deserialize(t.into_deserializer())
						.map_err(|_: serde::de::value::Error|"Could not deserialize place type within excludedTypes array")
					)
					.collect::<Result<Vec<_>,_>>()?
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

		let events: Vec<Event> = places
			.into_iter()
			.map(|p| Event::from(p))
			.collect();
		Ok(json!(events).to_string())
	}
}
