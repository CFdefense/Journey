/*
 * src/agent/tools/research.rs
 *
 * File for Reseearch Agent Tools
 *
 * Purpose:
 *   Store Research Agent Tools
 */

use async_trait::async_trait;
use langchain_rust::tools::Tool;
use serde_json::{Value, json};
use std::error::Error;

use crate::global::GOOGLE_MAPS_API_KEY;

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
		let location = input["location"].as_str().ok_or("Location should be a string")?;
		dotenvy::dotenv().map_err(|_|"Failed to load environment variables")?;
		let gm_api_key = std::env::var(GOOGLE_MAPS_API_KEY).map_err(|_|"GOOGLE_MAPS_API_KEY is not set")?;

		// use google maps api to get the address from the provided location and use geocoding to get its coordinates
		let gm_client = google_maps::Client::try_new(gm_api_key).map_err(|_|"Failed to create client for Google Maps API")?;
		let geocode_res = gm_client
			.geocoding()
			.with_address(location)
			.execute()
			.await?;
		if let Some(err) = geocode_res.error_message {
			return Err(format!("Geocoding failed with status {} - {err}", geocode_res.status).into());
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
		}).to_string())
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
					"description": "A location in the world; ideally a city name or postal code, but could be anything that indicates a place like a street address."
				},
				"name": {
					"type": "string",
					"description": "The name of the person to greet"
				}
			},
			"required": ["name"]
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
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
				"name": {
					"type": "string",
					"description": "The name of the person to greet"
				}
			},
			"required": ["name"]
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let name = input["name"].as_str().ok_or("Name should be a string")?;

		Ok(format!("Hello, {}! Welcome to our AI assistant.", name))
	}
}