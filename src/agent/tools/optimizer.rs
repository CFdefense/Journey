/*
 * src/agent/tools/optimizer.rs
 *
 * File for Optimizer Agent Tools
 *
 * Purpose:
 *   Store Optimizer Agent Tools for itinerary optimization
 */

use async_trait::async_trait;
use langchain_rust::{language_models::llm::LLM, tools::Tool};
use serde_json::{Value, json};
use std::{error::Error, sync::Arc};

pub fn optimizer_tools(llm: Arc<dyn LLM + Send + Sync>) -> [Arc<dyn Tool>; 3] {
	[
		Arc::new(RankPOIsByPreferenceTool {llm: llm.clone()}),
		Arc::new(DraftItineraryTool {llm}),
		Arc::new(OptimizeRouteTool),
	]
}

/// Tool that ranks Points of Interest based on user preferences and constraints
///
/// This tool evaluates and ranks POIs considering user profile factors such as:
/// - Budget constraints
/// - Risk tolerance
/// - Dietary restrictions/allergies
/// - Accessibility needs/disabilities
/// - Personal interests and preferences
#[derive(Clone)]
struct RankPOIsByPreferenceTool {
	llm: Arc<dyn LLM + Send + Sync>,
}

/// Tool that builds an itinerary from a list of events
#[derive(Clone)]
struct DraftItineraryTool {
	llm: Arc<dyn LLM + Send + Sync>,
}

/// Tool that optimizes the travel route for a day using TSP algorithms
///
/// This tool applies Traveling Salesman Problem optimization to minimize
/// travel time and distance between POIs in a single day. Considers:
/// - Geographic proximity
/// - Transportation methods available
/// - Traffic patterns
/// - Walking distances
#[derive(Clone)]
struct OptimizeRouteTool;

#[async_trait]
impl Tool for RankPOIsByPreferenceTool {
	fn name(&self) -> String {
		"rank_pois_by_preference".to_string()
	}

	fn description(&self) -> String {
		"Ranks a list of Points of Interest based on user preferences, budget, risk tolerance, allergies, and accessibility needs. Returns a prioritized list of POIs with scores."
            .to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {
				"pois": {
					"type": "array",
					"description": "Array of POI objects to be ranked",
					"items": {
						"type": "object"
					}
				},
				"user_profile": {
					"type": "object",
					"description": "User profile containing budget, risk tolerance, allergies, disabilities, and preferences",
					"properties": {
						"budget": {"type": "number"},
						"risk_tolerance": {"type": "string"},
						"allergies": {"type": "array"},
						"disabilities": {"type": "array"},
						"interests": {"type": "array"}
					}
				}
			},
			"required": ["pois", "user_profile"]
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let pois = input["pois"]
			.as_array()
			.ok_or("pois must be an array of objects")?;
		let profile = input["user_profile"]
			.as_object()
			.ok_or("user_profile must be an object")?;

		let prompt = format!(
			include_str!("../prompts/rank_pois_preference.md"),
			serde_json::to_string_pretty(&pois)?,
			serde_json::to_string_pretty(&profile)?
		);

		let response = self.llm.invoke(&prompt).await?;

		Ok(response.trim().to_string())
	}
}

#[async_trait]
impl Tool for DraftItineraryTool {
	fn name(&self) -> String {
		"draft_itinerary".to_string()
	}

	fn description(&self) -> String {
		"Assemble an itinerary from the list of POIs into the provided itinerary model structure."
            .to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {
				"pois": {
					"type": "array",
					"description": "Array of POI objects to be clustered",
					"items": {
						"type": "object"
					}
				},
				"diversity_factor": {
					"type": "number",
					"description": "Factor controlling how much diversity to enforce (0.0 to 1.0)",
					"default": 0.7
				}
			},
			"required": ["pois"]
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let pois = input["pois"]
			.as_array()
			.ok_or("pois must be an array of objects")?;
		let diversity_factor = input["diversity_factor"]
			.as_number()
			.map(|n| n.as_f64().ok_or("diversity_factor must be a 64-bit floating point number"));

		let prompt = format!(
			include_str!("../prompts/draft_itinerary.md"),
			serde_json::to_string_pretty(&pois)?,
			include_str!("../prompts/itinerary.ts"),
			diversity_factor.unwrap_or(Ok(0.7))?
		);

		let response = self.llm.invoke(&prompt).await?;

		Ok(response.trim().to_string())
	}
}

#[async_trait]
impl Tool for OptimizeRouteTool {
	fn name(&self) -> String {
		"optimize_route".to_string()
	}

	fn description(&self) -> String {
		"Optimizes the order of POIs for a day to minimize travel time and distance using Traveling Salesman Problem algorithms. Returns the most efficient route. Input must be a single array of events, and the output will be that array sorted to optimize for shortest routes between events."
            .to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {
				"day_pois": {
					"type": "array",
					"description": "Array of POI objects for a single day with location data. Should not include start_location or end_location.",
					"items": {
						"type": "object",
						"properties": {
							"id": {"type": "string"},
							"latitude": {"type": "number"},
							"longitude": {"type": "number"}
						}
					}
				},
				"start_location": {
					"type": "object",
					"description": "Starting location (e.g., hotel)",
					"properties": {
						"latitude": {"type": "number"},
						"longitude": {"type": "number"}
					}
				},
				"end_location": {
					"type": "object",
					"description": "Ending location (optional, defaults to start_location)",
					"properties": {
						"latitude": {"type": "number"},
						"longitude": {"type": "number"}
					}
				}
			},
			"required": ["day_pois", "start_location"]
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		use super::tsp::{EndpointMode, Pt, compute_route};

		let mut pois = input["day_pois"]
			.as_array()
			.ok_or("day_pois must be an array of objects")?
			.iter()
			.map(|poi| {
				let poi = poi
					.as_object()
					.ok_or("day_pois must be an array of objects")?;
				Ok(Pt {
					id: Some(poi["id"].as_str().ok_or("id must be a string")?),
					lat: poi["latitude"]
						.as_number()
						.ok_or("latitude must be an number")?
						.as_f64()
						.ok_or("latitude must be a 64-bit floating point number")?,
					lng: poi["longitude"]
						.as_number()
						.ok_or("longitude must be an number")?
						.as_f64()
						.ok_or("longitude must be a 64-bit floating point number")?,
				})
			})
			.collect::<Result<Vec<Pt>, &str>>()?;

		let start = input["start_location"]
			.as_object()
			.ok_or("start_location must be an object")?;
		let start = Pt {
			id: None,
			lat: start["latitude"]
				.as_number()
				.ok_or("latitude must be an number")?
				.as_f64()
				.ok_or("latitude must be a 64-bit floating point number")?,
			lng: start["longitude"]
				.as_number()
				.ok_or("longitude must be an number")?
				.as_f64()
				.ok_or("longitude must be a 64-bit floating point number")?,
		};

		pois.insert(0, start);

		if input["end_location"].is_null() {
			pois = compute_route(pois.as_slice(), EndpointMode::Circle)
				.into_iter()
				.map(|i| pois[i])
				.collect();
			return Ok(json!(pois).to_string());
		}

		let end = input["end_location"]
			.as_object()
			.ok_or("end_location must be an object")?;
		let end = Pt {
			id: None,
			lat: end["latitude"]
				.as_number()
				.ok_or("latitude must be an number")?
				.as_f64()
				.ok_or("latitude must be a 64-bit floating point number")?,
			lng: end["longitude"]
				.as_number()
				.ok_or("longitude must be an number")?
				.as_f64()
				.ok_or("longitude must be a 64-bit floating point number")?,
		};

		if start.lat == end.lat && start.lng == end.lng {
			pois = compute_route(pois.as_slice(), EndpointMode::Circle)
				.into_iter()
				.map(|i| pois[i])
				.collect();
			return Ok(json!(pois).to_string());
		}

		pois.push(end);

		pois = compute_route(pois.as_slice(), EndpointMode::Path)
			.into_iter()
			.map(|i| pois[i])
			.collect();
		Ok(json!(pois).to_string())
	}
}