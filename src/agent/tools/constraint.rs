/*
 * src/agent/tools/constraint.rs
 *
 * Constraint Agent tools.
 *
 * These tools are used by the Constraint Agent to filter research results
 * based on user constraints (e.g., wheelchair accessibility).
 */

use async_trait::async_trait;
use futures::future;
use langchain_rust::language_models::llm::LLM;
use langchain_rust::tools::Tool;
use serde_json::{Value, json};
use sqlx::PgPool;
use std::error::Error;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};

use crate::http_models::event::Event;

/// Uses an LLM to intelligently determine if an event should be included
/// based on trip context, user preferences, and constraints
async fn should_include_event(
	llm: &Arc<dyn LLM + Send + Sync>,
	event: &Event,
	preferences: &[String],
	constraints: &[String],
) -> Result<(bool, Option<String>), Box<dyn Error>> {
	// Serialize the event as JSON for the LLM to analyze
	let event_json = serde_json::to_string_pretty(event)
		.unwrap_or_else(|_| format!("Event: {}", event.event_name));

	// Create a prompt with all event details
	let prompt = format!(
		r#"You are evaluating whether a place/event is relevant for a vacation trip.

TRIP CONTEXT:
- User preferences: {}
- User constraints: {}

PLACE TO EVALUATE (all available data):
{}

RULES:
1. ALWAYS EXCLUDE: schools, universities, hospitals, clinics, retail stores (Home Depot, Target, CVS, Staples), DMV, government offices, gas stations, banks
2. INCLUDE places that match user preferences (e.g., if they want to "eat a lot", include restaurants, cafes, bars, wineries)
3. INCLUDE potentially interesting vacation spots: parks, museums, theaters, attractions, hotels, landmarks
4. For food preferences, only include actual dining establishments (restaurants, cafes, bars, bakeries, wineries) - NOT grocery stores
5. If user requires wheelchair accessibility, check the wheelchair_accessible fields and EXCLUDE places that are not accessible

Should this place be INCLUDED in the vacation itinerary?
Respond with ONLY a JSON object in this exact format:
{{"include": true/false, "reason": "brief reason"}}"#,
		if preferences.is_empty() {
			"none specified".to_string()
		} else {
			preferences.join(", ")
		},
		if constraints.is_empty() {
			"none".to_string()
		} else {
			constraints.join(", ")
		},
		event_json
	);

	let response = llm.invoke(&prompt).await?;

	// Parse the LLM response
	let cleaned = response
		.trim()
		.trim_start_matches("```json")
		.trim_end_matches("```")
		.trim();

	match serde_json::from_str::<Value>(cleaned) {
		Ok(parsed) => {
			let include = parsed
				.get("include")
				.and_then(|v| v.as_bool())
				.unwrap_or(false);
			let reason = parsed
				.get("reason")
				.and_then(|v| v.as_str())
				.map(|s| s.to_string());
			Ok((include, reason))
		}
		Err(_) => {
			// If parsing fails, default to including the event
			debug!(target: "constraint_tools", "Failed to parse LLM response for event: {}, response: {}", event.event_name, response);
			Ok((
				true,
				Some("LLM response parsing failed, including by default".to_string()),
			))
		}
	}
}

/// Tool that filters a list of event IDs based on user constraints.
///
/// Expected input (from the Orchestrator via `route_task` with `task_type = "constraint"`):
///
/// ```json
/// {
///   "event_ids": [1, 2, 3, ...], // array of event IDs from research agent
///   "constraints": ["No Tree Nuts", "No Peanuts", "Wheelchair accessible required: ..."],
///   "trip_context": { ... } // optional, for future use
/// }
/// ```
///
/// - If `constraints` is empty, this tool returns a natural-language question
///   asking the user for constraints/preferences.
/// - If constraints are present, it fetches the events from the database by ID,
///   evaluates each with an LLM, and returns:
///
/// ```json
/// {
///   "filtered_event_ids": [1, 3, 5, ...],
///   "removed_events": [ { "event_id": 2, "event_name": "...", "reasons": ["..."] } ]
/// }
/// ```
#[derive(Clone)]
pub struct FilterEventsByConstraintsTool {
	llm: Arc<dyn LLM + Send + Sync>,
	db: PgPool,
}

impl FilterEventsByConstraintsTool {
	pub fn new(llm: Arc<dyn LLM + Send + Sync>, db: PgPool) -> Self {
		Self { llm, db }
	}
}

#[async_trait]
impl Tool for FilterEventsByConstraintsTool {
	fn name(&self) -> String {
		"filter_events_by_constraints".to_string()
	}

	fn description(&self) -> String {
		"Filters a list of event IDs from the Research Agent using the user's constraints (e.g., wheelchair accessibility). IMPORTANT: Pass the entire JSON input you received (with event_ids, constraints, and trip_context) as the action_input. The tool will extract what it needs."
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
			agent: "constraint",
			tool: "filter_events_by_constraints",
			status: "start"
		);

		info!(
			target: "constraint_tools",
			tool = "filter_events_by_constraints",
			"Starting constraint filtering"
		);
		debug!(
			target: "constraint_tools",
			tool = "filter_events_by_constraints",
			input = %serde_json::to_string(&input).unwrap_or_else(|_| "invalid".to_string()),
			"Tool input"
		);

		// Normalize input: langchain_rust passes `action_input` as a STRING.
		let parsed_input: Value = if input.is_string() {
			serde_json::from_str(input.as_str().unwrap_or("{}")).unwrap_or_else(|_| json!({}))
		} else {
			input
		};

		// Extract event_ids array - handle both array and JSON string formats
		let mut event_ids_val = parsed_input
			.get("event_ids")
			.cloned()
			.or_else(|| {
				parsed_input
					.get("events")
					.and_then(|v| v.get("event_ids"))
					.cloned()
			})
			.or_else(|| {
				parsed_input
					.get("data")
					.and_then(|v| v.get("event_ids"))
					.cloned()
			})
			.unwrap_or(Value::Null);

		// If event_ids is a JSON string, parse it into an array
		if event_ids_val.is_string() {
			let event_ids_str = event_ids_val.as_str().unwrap_or("[]");
			event_ids_val = serde_json::from_str(event_ids_str).unwrap_or(Value::Null);
		}

		// Check if we already have a result (agent calling tool on previous output)
		if event_ids_val.is_null() && parsed_input.get("filtered_event_ids").is_some() {
			// Agent is trying to call the tool on its own previous output
			// Just return that output directly
			info!(
				target: "constraint_tools",
				"Tool called with previous output, returning it directly"
			);
			return Ok(serde_json::to_string(&parsed_input)?);
		}

		let event_ids: Vec<i32> = event_ids_val
			.as_array()
			.ok_or("event_ids should be an array of integers")?
			.iter()
			.filter_map(|v| v.as_i64().map(|i| i as i32))
			.collect();

		if event_ids.is_empty() {
			crate::tool_trace!(
				agent: "constraint",
				tool: "filter_events_by_constraints",
				status: "error",
				details: "no event IDs provided"
			);
			return Err("No event IDs provided to constraint agent".into());
		}

		info!(
			target: "constraint_tools",
			tool = "filter_events_by_constraints",
			event_ids_count = event_ids.len(),
			"Fetching events from database"
		);

		// Fetch all events from database by their IDs
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
				periods as "periods!: Vec<crate::sql_models::Period>",
				special_days
			FROM events
			WHERE id = ANY($1)
			"#,
			&event_ids
		)
		.fetch_all(&self.db)
		.await?;

		// Map rows to Event structs
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
				user_created: row.user_created,
				hard_start: row.hard_start,
				hard_end: row.hard_end,
				timezone: row.timezone,
				place_id: row.place_id,
				wheelchair_accessible_parking: row.wheelchair_accessible_parking,
				wheelchair_accessible_entrance: row.wheelchair_accessible_entrance,
				wheelchair_accessible_restroom: row.wheelchair_accessible_restroom,
				wheelchair_accessible_seating: row.wheelchair_accessible_seating,
				serves_vegetarian_food: row.serves_vegetarian_food,
				price_level: row.price_level,
				utc_offset_minutes: row.utc_offset_minutes,
				website_uri: row.website_uri,
				types: row.types,
				photo_name: row.photo_name,
				photo_width: row.photo_width,
				photo_height: row.photo_height,
				photo_author: row.photo_author,
				photo_author_uri: row.photo_author_uri,
				photo_author_photo_uri: row.photo_author_photo_uri,
				weekday_descriptions: row.weekday_descriptions,
				secondary_hours_type: row.secondary_hours_type,
				next_open_time: row.next_open_time,
				next_close_time: row.next_close_time,
				open_now: row.open_now,
				periods: row.periods,
				special_days: row.special_days,
				block_index: None, // Not used in constraint filtering
			})
			.collect();

		if events.is_empty() {
			crate::tool_trace!(
				agent: "constraint",
				tool: "filter_events_by_constraints",
				status: "error",
				details: "no events found in database for provided IDs"
			);
			return Err("No events found in database for the provided IDs".into());
		}

		info!(
			target: "constraint_tools",
			tool = "filter_events_by_constraints",
			events_fetched = events.len(),
			"Events fetched successfully"
		);

		// Extract constraints (strings, lowercased for matching)
		let mut constraints_val = if parsed_input.get("constraints").is_some() {
			parsed_input
				.get("constraints")
				.cloned()
				.unwrap_or(Value::Null)
		} else {
			parsed_input
				.get("trip_context")
				.and_then(|tc| tc.get("constraints"))
				.cloned()
				.unwrap_or(Value::Null)
		};

		// If constraints is a JSON string, parse it into an array
		if constraints_val.is_string() {
			let constraints_str = constraints_val.as_str().unwrap_or("[]");
			constraints_val = serde_json::from_str(constraints_str).unwrap_or(Value::Null);
		}

		let constraints: Vec<String> = if let Some(arr) = constraints_val.as_array() {
			arr.iter()
				.filter_map(|v| v.as_str().map(|s| s.to_lowercase()))
				.collect()
		} else {
			Vec::new()
		};

		// Extract preferences from trip_context
		let preferences: Vec<String> = parsed_input
			.get("trip_context")
			.and_then(|tc| tc.get("preferences"))
			.and_then(|p| p.as_array())
			.map(|arr| {
				arr.iter()
					.filter_map(|v| v.as_str().map(|s| s.to_lowercase()))
					.collect()
			})
			.unwrap_or_else(Vec::new);

		// Constraints are optional - if none exist, we proceed with no filtering
		// The task agent should have already asked for clarification if critical info was missing
		if constraints.is_empty() {
			info!(
				target: "constraint_tools",
				tool = "filter_events_by_constraints",
				"No constraints provided - will include all events"
			);
		}

		debug!(
			target: "constraint_tools",
			tool = "filter_events_by_constraints",
			preferences = ?preferences,
			constraints = ?constraints,
			events_count = events.len(),
			"Processing events with LLM-based filtering"
		);

		// Process each event with LLM evaluation. We evaluate multiple events in parallel
		// with a bounded level of concurrency so the overall tool is faster while still
		// respecting upstream rate limits.
		// Evaluate all events concurrently. join_all does not require the inner
		// futures to be Send, but still allows the HTTP calls to overlap.
		let tasks = events.iter().cloned().map(|event| {
			let llm = Arc::clone(&self.llm);
			let preferences = preferences.clone();
			let constraints = constraints.clone();

			async move {
				match should_include_event(&llm, &event, &preferences, &constraints).await {
					Ok((should_include, reason)) => (event, should_include, reason),
					Err(e) => {
						debug!(
							target: "constraint_tools",
							tool = "filter_events_by_constraints",
							error = %e,
							event_name = %event.event_name,
							"LLM evaluation failed, including event by default"
						);
						// On error, include the event by default to avoid over-filtering
						(
							event,
							true,
							Some("LLM evaluation failed, including by default".to_string()),
						)
					}
				}
			}
		});

		let eval_results: Vec<(Event, bool, Option<String>)> = future::join_all(tasks).await;

		let mut filtered_ids: Vec<i32> = Vec::new();
		let mut removed: Vec<Value> = Vec::new();

		for (event, should_include, reason) in eval_results {
			if should_include {
				filtered_ids.push(event.id);
			} else {
				let reason_text = reason.unwrap_or_else(|| "not relevant for trip".to_string());
				removed.push(json!({
					"event_id": event.id,
					"event_name": &event.event_name,
					"reasons": [reason_text],
				}));
			}
		}

		let result = json!({
			"filtered_event_ids": filtered_ids,
			"removed_events": removed,
			"count": filtered_ids.len()
		});

		let elapsed = start_time.elapsed();

		// Extract event names for debugging
		let filtered_names: Vec<String> = events
			.iter()
			.filter(|e| filtered_ids.contains(&e.id))
			.map(|e| e.event_name.clone())
			.collect();

		let removed_names: Vec<String> = result["removed_events"]
			.as_array()
			.map(|arr| {
				arr.iter()
					.filter_map(|e| {
						e.get("event_name")
							.and_then(|n| n.as_str())
							.map(|s| s.to_string())
					})
					.collect()
			})
			.unwrap_or_default();

		crate::tool_trace!(
			agent: "constraint",
			tool: "filter_events_by_constraints",
			status: "success",
			details: format!(
				"elapsed_ms={}, filtered_count={}, removed_count={}, filtered=[{}], removed=[{}]",
				elapsed.as_millis(),
				filtered_ids.len(),
				removed.len(),
				filtered_names.join(", "),
				removed_names.join(", ")
			)
		);

		info!(
			target: "constraint_tools",
			tool = "filter_events_by_constraints",
			elapsed_ms = elapsed.as_millis() as u64,
			filtered_count = filtered_ids.len(),
			removed_count = removed.len(),
			"Constraint filtering completed"
		);

		Ok(result.to_string())
	}
}

pub fn constraint_tools(llm: Arc<dyn LLM + Send + Sync>, db: PgPool) -> Vec<Arc<dyn Tool>> {
	vec![Arc::new(FilterEventsByConstraintsTool::new(llm, db))]
}
