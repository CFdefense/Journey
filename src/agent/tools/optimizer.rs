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
use sqlx::PgPool;
use std::{error::Error, sync::Arc, time::Instant};
use tracing::{debug, info, warn};

use crate::agent::models::event::Event;

pub fn optimizer_tools(llm: Arc<dyn LLM + Send + Sync>, db: PgPool) -> Vec<Arc<dyn Tool>> {
	vec![Arc::new(OptimizeItineraryTool::new(
		llm.clone(),
		db.clone(),
	))]
}

/// Main tool that orchestrates the full optimization workflow.
/// This tool:
/// 1. Accepts filtered event IDs from the constraint agent
/// 2. Fetches events from the database
/// 3. Retrieves user profile from context
/// 4. Ranks POIs by preference
/// 5. Drafts an itinerary
/// 6. Optimizes routes for each day
/// 7. Returns a complete structured itinerary
#[derive(Clone)]
struct OptimizeItineraryTool {
	llm: Arc<dyn LLM + Send + Sync>,
	db: PgPool,
}

impl OptimizeItineraryTool {
	pub fn new(llm: Arc<dyn LLM + Send + Sync>, db: PgPool) -> Self {
		Self { llm, db }
	}
}

#[async_trait]
impl Tool for OptimizeItineraryTool {
	fn name(&self) -> String {
		"optimize_itinerary".to_string()
	}

	fn description(&self) -> String {
		"Main optimization workflow tool. IMPORTANT: Pass the entire JSON input you received (with filtered_event_ids, trip_context, and user_profile) as the action_input. Fetches events from database, ranks them by user preference, drafts an itinerary, and optimizes routes. Returns a complete structured itinerary ready for storage."
			.to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {
				"input_data": {
					"type": "string",
					"description": "Pass the entire JSON input you received as a string. Do not pass empty string."
				}
			},
			"required": []
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let start_time = Instant::now();

		crate::tool_trace!(
			agent: "optimize",
			tool: "optimize_itinerary",
			status: "start"
		);

		info!(
			target: "optimize_tools",
			tool = "optimize_itinerary",
			"Starting itinerary optimization workflow"
		);
		debug!(
			target: "optimize_tools",
			input = %serde_json::to_string(&input).unwrap_or_else(|_| "invalid".to_string()),
			"Tool input"
		);

		// Parse input (handle both string and object formats from langchain_rust)
		let parsed_input: Value = if input.is_string() {
			serde_json::from_str(input.as_str().unwrap_or("{}")).unwrap_or_else(|_| json!({}))
		} else {
			input
		};

		// Extract and parse filtered_event_ids
		let mut event_ids_val = parsed_input
			.get("filtered_event_ids")
			.cloned()
			.or_else(|| {
				// Try alternative paths from orchestrator payload
				parsed_input
					.get("events")
					.and_then(|v| v.get("filtered_event_ids"))
					.cloned()
			})
			.unwrap_or(Value::Null);

		// If event_ids is a JSON string, parse it
		if event_ids_val.is_string() {
			let event_ids_str = event_ids_val.as_str().unwrap_or("[]");
			event_ids_val = serde_json::from_str(event_ids_str).unwrap_or(Value::Null);
		}

		// Check if we already have a result (agent calling tool on previous output)
		if event_ids_val.is_null() && parsed_input.get("event_days").is_some() {
			// Agent is trying to call the tool on its own previous output (an itinerary)
			// Just return that output directly
			info!(
				target: "optimize_tools",
				"Tool called with previous output, returning it directly"
			);
			return Ok(serde_json::to_string(&parsed_input)?);
		}

		let event_ids: Vec<i32> = event_ids_val
			.as_array()
			.ok_or("filtered_event_ids should be an array of integers")?
			.iter()
			.filter_map(|v| v.as_i64().map(|i| i as i32))
			.collect();

		if event_ids.is_empty() {
			crate::tool_trace!(
				agent: "optimize",
				tool: "optimize_itinerary",
				status: "error",
				details: "no event IDs provided"
			);
			return Err("No event IDs provided to optimize agent".into());
		}

		info!(
			target: "optimize_tools",
			event_count = event_ids.len(),
			"Fetching events from database"
		);

		// Fetch all events from database
		let rows = sqlx::query!(
			r#"
			SELECT
				id,
				event_name,
				event_description,
				street_address,
				city,
				country,
				postal_code,
				lat,
				lng,
				event_type,
				hard_start,
				hard_end,
				timezone,
				wheelchair_accessible_parking,
				wheelchair_accessible_entrance,
				wheelchair_accessible_restroom,
				wheelchair_accessible_seating,
				serves_vegetarian_food,
				price_level,
				utc_offset_minutes,
				types,
				weekday_descriptions,
				secondary_hours_type,
				next_open_time,
				next_close_time,
				open_now,
				periods as "periods!: Vec<crate::sql_models::Period>",
				special_days
			FROM events
			WHERE id = ANY($1)
			"#,
			&event_ids
		)
		.fetch_all(&self.db)
		.await?;

		let events: Vec<Event> = rows
			.into_iter()
			.map(|row| Event {
				id: row.id,
				event_name: row.event_name,
				event_description: row.event_description,
				street_address: row.street_address,
				city: row.city,
				country: row.country,
				postal_code: row.postal_code,
				lat: row.lat,
				lng: row.lng,
				event_type: row.event_type,
				hard_start: row.hard_start,
				hard_end: row.hard_end,
				timezone: row.timezone,
				wheelchair_accessible_parking: row.wheelchair_accessible_parking,
				wheelchair_accessible_entrance: row.wheelchair_accessible_entrance,
				wheelchair_accessible_restroom: row.wheelchair_accessible_restroom,
				wheelchair_accessible_seating: row.wheelchair_accessible_seating,
				serves_vegetarian_food: row.serves_vegetarian_food,
				price_level: row.price_level,
				utc_offset_minutes: row.utc_offset_minutes,
				types: row.types,
				weekday_descriptions: row.weekday_descriptions,
				secondary_hours_type: row.secondary_hours_type,
				next_open_time: row.next_open_time,
				next_close_time: row.next_close_time,
				open_now: row.open_now,
				periods: row.periods,
				special_days: row.special_days,
				block_index: None,
			})
			.collect();

		if events.is_empty() {
			crate::tool_trace!(
				agent: "optimize",
				tool: "optimize_itinerary",
				status: "error",
				details: "no events found in database"
			);
			return Err("No events found in database for provided IDs".into());
		}

		info!(
			target: "optimize_tools",
			events_fetched = events.len(),
			"Events fetched successfully"
		);

		// Extract trip_context and user_profile
		let mut trip_context_val = parsed_input
			.get("trip_context")
			.cloned()
			.unwrap_or(Value::Null);
		if trip_context_val.is_string() {
			trip_context_val = serde_json::from_str(trip_context_val.as_str().unwrap_or("{}"))
				.unwrap_or(json!({}));
		}

		let mut user_profile_val = parsed_input
			.get("user_profile")
			.cloned()
			.unwrap_or(Value::Null);
		if user_profile_val.is_string() {
			user_profile_val = serde_json::from_str(user_profile_val.as_str().unwrap_or("{}"))
				.unwrap_or(json!({}));
		}

		// STEP 1: Rank POIs by preference
		info!(
			target: "optimize_tools",
			"Step 1: Ranking POIs by user preference"
		);

		let events_json = serde_json::to_value(&events)?;
		let rank_input = json!({
			"pois": events_json,
			"user_profile": user_profile_val
		});

		let rank_tool = RankPOIsByPreferenceTool {
			llm: self.llm.clone(),
		};
		let ranked_result = rank_tool.run(rank_input).await?;

		// Parse the ranked POIs (should be JSON array with rank fields added)
		// Try to extract JSON from markdown code blocks if present
		let cleaned_result = ranked_result
			.trim()
			.strip_prefix("```json")
			.and_then(|s| s.strip_suffix("```"))
			.or_else(|| {
				ranked_result
					.trim()
					.strip_prefix("```")
					.and_then(|s| s.strip_suffix("```"))
			})
			.unwrap_or(ranked_result.trim());

		let mut ranked_pois: Vec<Value> = serde_json::from_str(cleaned_result)
			.map_err(|e| {
				info!(
					target: "optimize_tools",
					error = %e,
					response = %ranked_result,
					"Failed to parse ranked POIs, adding default ranks"
				);
				e
			})
			.unwrap_or_else(|_| {
				// Fallback: use original events and add default ranks
				events_json.as_array().cloned().unwrap_or_default()
			});

		// Ensure all POIs have a rank field (add default if missing)
		for poi in ranked_pois.iter_mut() {
			if poi.get("rank").is_none() {
				// Add a high default rank for POIs missing the rank field
				if let Some(obj) = poi.as_object_mut() {
					obj.insert("rank".to_string(), json!(999));
				}
			}
		}

		// Sort by rank to ensure best POIs come first
		ranked_pois.sort_by(|a, b| {
			let rank_a = a.get("rank").and_then(|r| r.as_i64()).unwrap_or(999);
			let rank_b = b.get("rank").and_then(|r| r.as_i64()).unwrap_or(999);
			rank_a.cmp(&rank_b)
		});

		// Extract ranking summary for logging
		let mut rankings: Vec<String> = ranked_pois
			.iter()
			.map(|poi| {
				let name = poi
					.get("event_name")
					.and_then(|n| n.as_str())
					.unwrap_or("Unknown");
				let rank = poi.get("rank").and_then(|r| r.as_i64()).unwrap_or(999);
				format!("{}(rank:{})", name, rank)
			})
			.collect();
		rankings.sort_by_key(|s| {
			// Extract rank number for sorting
			s.split("rank:")
				.nth(1)
				.and_then(|r| r.trim_end_matches(')').parse::<i64>().ok())
				.unwrap_or(999)
		});

		info!(
			target: "optimize_tools",
			ranked_count = ranked_pois.len(),
			"POIs ranked successfully"
		);

		crate::tool_trace!(
			agent: "optimize",
			tool: "rank_pois_by_preference",
			status: "success",
			details: format!("rankings=[{}]", rankings.join(", "))
		);

		// STEP 2: Draft the itinerary
		info!(
			target: "optimize_tools",
			"Step 2: Drafting itinerary structure"
		);

		let draft_input = json!({
			"pois": ranked_pois,
			"diversity_factor": 0.7,
			"trip_context": trip_context_val
		});

		let draft_tool = DraftItineraryTool {
			llm: self.llm.clone(),
		};
		let draft_result = draft_tool.run(draft_input).await?;

		// Parse the draft itinerary
		// Try to extract JSON from markdown code blocks if present
		let cleaned_draft = draft_result
			.trim()
			.strip_prefix("```json")
			.and_then(|s| s.strip_suffix("```"))
			.or_else(|| {
				draft_result
					.trim()
					.strip_prefix("```")
					.and_then(|s| s.strip_suffix("```"))
			})
			.unwrap_or(draft_result.trim());

		// Try standard JSON parsing first
		let mut itinerary: Value = match serde_json::from_str(cleaned_draft) {
			Ok(value) => value,
			Err(e) => {
				warn!(
					target: "optimize_tools",
					error = %e,
					response_len = draft_result.len(),
					"Failed to parse draft itinerary with standard JSON parser, trying JSON5"
				);

				// Try JSON5 parser which is more lenient (handles trailing commas, comments, etc.)
				match json5::from_str(cleaned_draft) {
					Ok(value) => {
						info!(
							target: "optimize_tools",
							"Successfully parsed draft itinerary using JSON5 (lenient parser)"
						);
						value
					}
					Err(json5_err) => {
						// Both parsers failed - log detailed error and return error
						let preview = draft_result.chars().take(500).collect::<String>();

						crate::tool_trace!(
							agent: "optimize",
							tool: "draft_itinerary",
							status: "error",
							details: format!("JSON parse failed: {}", e)
						);

						return Err(format!(
						"Failed to parse draft itinerary. Standard JSON error: {}. JSON5 error: {}. Response preview: {}",
						e, json5_err, preview
					).into());
					}
				}
			}
		};

		// Build schedule summary
		use std::collections::HashMap;
		let name_by_id: HashMap<i32, String> =
			events.iter().map(|e| (e.id, e.event_name.clone())).collect();

		let mut schedule_summary: Vec<String> = Vec::new();
		if let Some(event_days) = itinerary.get("event_days").and_then(|v| v.as_array()) {
			for (day_idx, day) in event_days.iter().enumerate() {
				let date = day
					.get("date")
					.and_then(|d| d.as_str())
					.unwrap_or("unknown");

				let morning = day
					.get("morning_events")
					.and_then(|e| e.as_array())
					.map(|arr| {
						arr.iter()
							.filter_map(|e| {
								if let Some(name) =
									e.get("event_name").and_then(|n| n.as_str())
								{
									Some(name.to_string())
								} else if let Some(id) = e.get("id").and_then(|v| v.as_i64())
								{
									name_by_id.get(&(id as i32)).cloned()
								} else {
									None
								}
							})
							.collect::<Vec<_>>()
							.join(", ")
					})
					.unwrap_or_default();

				let afternoon = day
					.get("afternoon_events")
					.and_then(|e| e.as_array())
					.map(|arr| {
						arr.iter()
							.filter_map(|e| {
								if let Some(name) =
									e.get("event_name").and_then(|n| n.as_str())
								{
									Some(name.to_string())
								} else if let Some(id) = e.get("id").and_then(|v| v.as_i64())
								{
									name_by_id.get(&(id as i32)).cloned()
								} else {
									None
								}
							})
							.collect::<Vec<_>>()
							.join(", ")
					})
					.unwrap_or_default();

				let evening = day
					.get("evening_events")
					.and_then(|e| e.as_array())
					.map(|arr| {
						arr.iter()
							.filter_map(|e| {
								if let Some(name) =
									e.get("event_name").and_then(|n| n.as_str())
								{
									Some(name.to_string())
								} else if let Some(id) = e.get("id").and_then(|v| v.as_i64())
								{
									name_by_id.get(&(id as i32)).cloned()
								} else {
									None
								}
							})
							.collect::<Vec<_>>()
							.join(", ")
					})
					.unwrap_or_default();

				let mut day_parts = Vec::new();
				if !morning.is_empty() {
					day_parts.push(format!("AM:[{}]", morning));
				}
				if !afternoon.is_empty() {
					day_parts.push(format!("PM:[{}]", afternoon));
				}
				if !evening.is_empty() {
					day_parts.push(format!("EVE:[{}]", evening));
				}

				schedule_summary.push(format!(
					"Day{}({}):{}",
					day_idx + 1,
					date,
					if day_parts.is_empty() {
						"empty".to_string()
					} else {
						day_parts.join(" ")
					}
				));
			}
		}

		let days_count = schedule_summary.len();

		info!(
			target: "optimize_tools",
			days_count = days_count,
			"Draft itinerary created"
		);

		crate::tool_trace!(
			agent: "optimize",
			tool: "draft_itinerary",
			status: "success",
			details: format!("schedule=[{}]", schedule_summary.join(" | "))
		);

		// STEP 3: Optimize routes for each day
		info!(
			target: "optimize_tools",
			"Step 3: Optimizing routes for each day"
		);

		let optimize_route_tool = OptimizeRouteTool;
		let mut optimized_days = 0;

		// Get event_days array
		if let Some(event_days) = itinerary
			.get_mut("event_days")
			.and_then(|v| v.as_array_mut())
		{
			for day in event_days.iter_mut() {
				// Optimize morning events
				if let Some(morning) = day.get("morning_events").cloned() {
					if let Some(morning_arr) = morning.as_array() {
						if !morning_arr.is_empty() && morning_arr.len() > 1 {
							// Get first event location as start
							if let Some(first) = morning_arr.first() {
								let start_location = json!({
									"latitude": first.get("lat").and_then(|v| v.as_f64()).unwrap_or(0.0),
									"longitude": first.get("lng").and_then(|v| v.as_f64()).unwrap_or(0.0)
								});

								let route_input = json!({
									"day_pois": morning,
									"start_location": start_location
								});

								if let Ok(optimized) = optimize_route_tool.run(route_input).await {
									if let Ok(optimized_arr) =
										serde_json::from_str::<Value>(&optimized)
									{
										day["morning_events"] = optimized_arr;
										optimized_days += 1;
									}
								}
							}
						}
					}
				}

				// Optimize afternoon events
				if let Some(afternoon) = day.get("afternoon_events").cloned() {
					if let Some(afternoon_arr) = afternoon.as_array() {
						if !afternoon_arr.is_empty() && afternoon_arr.len() > 1 {
							if let Some(first) = afternoon_arr.first() {
								let start_location = json!({
									"latitude": first.get("lat").and_then(|v| v.as_f64()).unwrap_or(0.0),
									"longitude": first.get("lng").and_then(|v| v.as_f64()).unwrap_or(0.0)
								});

								let route_input = json!({
									"day_pois": afternoon,
									"start_location": start_location
								});

								if let Ok(optimized) = optimize_route_tool.run(route_input).await {
									if let Ok(optimized_arr) =
										serde_json::from_str::<Value>(&optimized)
									{
										day["afternoon_events"] = optimized_arr;
										optimized_days += 1;
									}
								}
							}
						}
					}
				}

				// Optimize evening events
				if let Some(evening) = day.get("evening_events").cloned() {
					if let Some(evening_arr) = evening.as_array() {
						if !evening_arr.is_empty() && evening_arr.len() > 1 {
							if let Some(first) = evening_arr.first() {
								let start_location = json!({
									"latitude": first.get("lat").and_then(|v| v.as_f64()).unwrap_or(0.0),
									"longitude": first.get("lng").and_then(|v| v.as_f64()).unwrap_or(0.0)
								});

								let route_input = json!({
									"day_pois": evening,
									"start_location": start_location
								});

								if let Ok(optimized) = optimize_route_tool.run(route_input).await {
									if let Ok(optimized_arr) =
										serde_json::from_str::<Value>(&optimized)
									{
										day["evening_events"] = optimized_arr;
										optimized_days += 1;
									}
								}
							}
						}
					}
				}
			}
		}

		info!(
			target: "optimize_tools",
			optimized_blocks = optimized_days,
			"Routes optimized for all days"
		);

		crate::tool_trace!(
			agent: "optimize",
			tool: "optimize_route",
			status: "success",
			details: format!("{} time blocks optimized", optimized_days)
		);

		// Add metadata to itinerary
		itinerary["start_date"] = trip_context_val
			.get("start_date")
			.cloned()
			.unwrap_or(Value::Null);
		itinerary["end_date"] = trip_context_val
			.get("end_date")
			.cloned()
			.unwrap_or(Value::Null);
		itinerary["title"] = trip_context_val
			.get("destination")
			.cloned()
			.unwrap_or(json!("Trip Itinerary"));

		// Normalize itinerary events to reference-only shape for downstream tools.
		//
		// Design goal:
		// - The optimizer works with full Event objects internally.
		// - The final itinerary we hand back to the orchestrator should use
		//   *event ids* as the durable reference, not full embedded records.
		// - `respond_to_user` (and the HTTP layer) will hydrate those ids back
		//   into full Event structs from the database.
		//
		// Implementation:
		// - For every event in `event_days[*].{morning,afternoon,evening}_events`
		//   and in `unassigned_events`:
		//     * If it has a valid `id` that came from our input set, collapse it
		//       to `{ "id": <int> }`.
		//     * If it does not have an `id`, leave it as-is (these are rare
		//       LLM-synthesized events; `respond_to_user` has defensive logic
		//       to persist and hydrate them).
		let allowed_ids: std::collections::HashSet<i32> = event_ids.iter().cloned().collect();

		if let Some(days) = itinerary.get_mut("event_days").and_then(|v| v.as_array_mut()) {
			for day in days.iter_mut() {
				for block in &["morning_events", "afternoon_events", "evening_events"] {
					if let Some(events_arr) = day.get_mut(*block).and_then(|v| v.as_array_mut()) {
						for ev in events_arr.iter_mut() {
							if let Some(id_val) = ev.get("id").and_then(|v| v.as_i64()) {
								let id = id_val as i32;
								if allowed_ids.contains(&id) {
									*ev = json!({ "id": id });
								}
							}
						}
					}
				}
			}
		}

		if let Some(unassigned) = itinerary
			.get_mut("unassigned_events")
			.and_then(|v| v.as_array_mut())
		{
			for ev in unassigned.iter_mut() {
				if let Some(id_val) = ev.get("id").and_then(|v| v.as_i64()) {
					let id = id_val as i32;
					if allowed_ids.contains(&id) {
						*ev = json!({ "id": id });
					}
				}
			}
		}

		let elapsed = start_time.elapsed();
		let result = serde_json::to_string(&itinerary)?;

		crate::tool_trace!(
			agent: "optimize",
			tool: "optimize_itinerary",
			status: "success",
			details: format!("elapsed_ms={}, events_processed={}", elapsed.as_millis(), events.len())
		);

		info!(
			target: "optimize_tools",
			elapsed_ms = elapsed.as_millis() as u64,
			events_processed = events.len(),
			"Optimization workflow completed successfully"
		);

		Ok(result)
	}
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
		let start_time = Instant::now();

		crate::tool_trace!(
			agent: "optimize",
			tool: "rank_pois_by_preference",
			status: "start"
		);

		info!(
			target: "optimize_tools",
			tool = "rank_pois_by_preference",
			"Starting POI ranking"
		);

		let pois = input["pois"]
			.as_array()
			.ok_or("pois must be an array of objects")?;
		let profile = input["user_profile"]
			.as_object()
			.ok_or("user_profile must be an object")?;

		info!(
			target: "optimize_tools",
			pois_count = pois.len(),
			"Ranking POIs by preference"
		);

		let prompt = format!(
			include_str!("../prompts/rank_pois_preference.md"),
			serde_json::to_string_pretty(&pois)?,
			serde_json::to_string_pretty(&profile)?
		);

		let response = self.llm.invoke(&prompt).await?;

		let elapsed = start_time.elapsed();

		crate::tool_trace!(
			agent: "optimize",
			tool: "rank_pois_by_preference",
			status: "success",
			details: format!("elapsed_ms={}, pois_count={}", elapsed.as_millis(), pois.len())
		);

		info!(
			target: "optimize_tools",
			elapsed_ms = elapsed.as_millis() as u64,
			pois_count = pois.len(),
			"POI ranking completed"
		);

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
		let start_time = Instant::now();

		crate::tool_trace!(
			agent: "optimize",
			tool: "draft_itinerary",
			status: "start"
		);

		info!(
			target: "optimize_tools",
			tool = "draft_itinerary",
			"Starting itinerary drafting"
		);

		let pois = input["pois"]
			.as_array()
			.ok_or("pois must be an array of objects")?;
		let diversity_factor = input["diversity_factor"].as_number().map(|n| {
			n.as_f64()
				.ok_or("diversity_factor must be a 64-bit floating point number")
		});
		let trip_context = input.get("trip_context").cloned().unwrap_or(json!({}));

		info!(
			target: "optimize_tools",
			pois_count = pois.len(),
			diversity_factor = diversity_factor.unwrap_or(Ok(0.7)).unwrap_or(0.7),
			"Drafting itinerary from POIs"
		);

		let prompt = format!(
			include_str!("../prompts/draft_itinerary.md"),
			serde_json::to_string_pretty(&pois)?,
			include_str!("../prompts/itinerary.ts"),
			diversity_factor.unwrap_or(Ok(0.7))?,
			serde_json::to_string_pretty(&trip_context)?
		);

		let response = self.llm.invoke(&prompt).await?;

		let elapsed = start_time.elapsed();

		crate::tool_trace!(
			agent: "optimize",
			tool: "draft_itinerary",
			status: "success",
			details: format!("elapsed_ms={}, pois_count={}", elapsed.as_millis(), pois.len())
		);

		info!(
			target: "optimize_tools",
			elapsed_ms = elapsed.as_millis() as u64,
			pois_count = pois.len(),
			"Itinerary draft completed"
		);

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

		let start_time = Instant::now();

		crate::tool_trace!(
			agent: "optimize",
			tool: "optimize_route",
			status: "start"
		);

		info!(
			target: "optimize_tools",
			tool = "optimize_route",
			"Starting route optimization"
		);

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

		let elapsed = start_time.elapsed();

		crate::tool_trace!(
			agent: "optimize",
			tool: "optimize_route",
			status: "success",
			details: format!("elapsed_ms={}", elapsed.as_millis())
		);

		info!(
			target: "optimize_tools",
			elapsed_ms = elapsed.as_millis() as u64,
			"Route optimization completed"
		);

		Ok(json!(pois).to_string())
	}
}
