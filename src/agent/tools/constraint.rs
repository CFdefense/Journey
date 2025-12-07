/*
 * src/agent/tools/constraint.rs
 *
 * Constraint Agent tools.
 *
 * These tools are used by the Constraint Agent to filter research results
 * based on user constraints (e.g., wheelchair accessibility).
 */

use async_trait::async_trait;
use langchain_rust::tools::Tool;
use langchain_rust::language_models::llm::LLM;
use serde_json::{json, Value};
use std::error::Error;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};

/// Uses an LLM to intelligently determine if an event should be included
/// based on trip context, user preferences, and constraints
async fn should_include_event(
	llm: &Arc<dyn LLM + Send + Sync>,
	event: &Value,
	preferences: &[String],
	constraints: &[String],
) -> Result<(bool, Option<String>), Box<dyn Error>> {
	let event_name = event.get("event_name").and_then(|v| v.as_str()).unwrap_or("Unknown");
	
	// Serialize the entire event as pretty JSON for the LLM to analyze
	let event_json = serde_json::to_string_pretty(event)
		.unwrap_or_else(|_| event.to_string());
	
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
		if preferences.is_empty() { "none specified".to_string() } else { preferences.join(", ") },
		if constraints.is_empty() { "none".to_string() } else { constraints.join(", ") },
		event_json
	);

	let response = llm.invoke(&prompt).await?;
	
	// Parse the LLM response
	let cleaned = response.trim().trim_start_matches("```json").trim_end_matches("```").trim();
	
	match serde_json::from_str::<Value>(cleaned) {
		Ok(parsed) => {
			let include = parsed.get("include").and_then(|v| v.as_bool()).unwrap_or(false);
			let reason = parsed.get("reason").and_then(|v| v.as_str()).map(|s| s.to_string());
			Ok((include, reason))
		},
		Err(_) => {
			// If parsing fails, default to including the event
			debug!(target: "constraint_tools", "Failed to parse LLM response for event: {}, response: {}", event_name, response);
			Ok((true, Some("LLM response parsing failed, including by default".to_string())))
		}
	}
}

/// Tool that filters a list of events based on simple constraints.
///
/// Expected input (from the Orchestrator via `route_task` with `task_type = "constraint"`):
///
/// ```json
/// {
///   "events": [ /* array of event-like objects */ ],
///   "constraints": ["No Tree Nuts", "No Peanuts", "Wheelchair accessible required: ..."],
///   "trip_context": { ... } // optional, for future use
/// }
/// ```
///
/// - If `constraints` is empty, this tool returns a natural-language question
///   asking the user for constraints/preferences.
/// - If constraints are present, it returns a JSON string:
///
/// ```json
/// {
///   "filtered_events": [ ... ],
///   "removed_events": [ { "event": { ... }, "reasons": ["..."] } ]
/// }
/// ```
#[derive(Clone)]
pub struct FilterEventsByConstraintsTool {
	llm: Arc<dyn LLM + Send + Sync>,
}

impl FilterEventsByConstraintsTool {
	pub fn new(llm: Arc<dyn LLM + Send + Sync>) -> Self {
		Self { llm }
	}
}

#[async_trait]
impl Tool for FilterEventsByConstraintsTool {
	fn name(&self) -> String {
		"filter_events_by_constraints".to_string()
	}

	fn description(&self) -> String {
		"Filters a list of events from the Research Agent using the user's constraints (e.g., wheelchair accessibility). If no constraints are available, asks the user once for their preferences and constraints."
			.to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {
				"events": {
					"type": "string",
					"description": "OPTIONAL. JSON stringified array of events to filter. If omitted, the tool will look for an 'events' field or 'research_result' object in the input payload."
				},
				"constraints": {
					"type": "string",
					"description": "OPTIONAL. JSON stringified array of constraint strings (e.g., '[\"No Tree Nuts\", \"Wheelchair accessible\"]'). If omitted, the tool will look for a 'constraints' or 'trip_context.constraints' field."
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

	// Extract events array - handle both array and JSON string formats
	let mut events_val = parsed_input
		.get("events")
		.cloned()
		.or_else(|| parsed_input.get("research_result").cloned())
		.or_else(|| parsed_input.get("data").cloned())
		.unwrap_or(Value::Null);

	// If events is a JSON string, parse it into an array
	if events_val.is_string() {
		let events_str = events_val.as_str().unwrap_or("[]");
		events_val = serde_json::from_str(events_str).unwrap_or(Value::Null);
	}

	let events = events_val
		.as_array()
		.ok_or("events should be an array of event-like objects")?;

		if events.is_empty() {
			crate::tool_trace!(
				agent: "constraint",
				tool: "filter_events_by_constraints",
				status: "error",
				details: "no events provided"
			);
			return Err("No events provided to constraint agent".into());
		}

	// Extract constraints (strings, lowercased for matching)
	let mut constraints_val = if parsed_input.get("constraints").is_some() {
		parsed_input.get("constraints").cloned().unwrap_or(Value::Null)
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

		// If we truly have no constraints, ask the user for them.
		if constraints.is_empty() {
			let question = "I have a list of candidate places for your trip, but I don't yet know your constraints or preferences (for example: accessibility needs, food allergies, or strict budget limits). Could you share any constraints you want me to respect when filtering these places?";
			info!(
				target: "constraint_tools",
				tool = "filter_events_by_constraints",
				"Missing constraints - asking user"
			);
			crate::tool_trace!(
				agent: "constraint",
				tool: "filter_events_by_constraints",
				status: "no_constraints",
				details: "asking user for constraints"
			);
			return Ok(question.to_string());
		}

	debug!(
		target: "constraint_tools",
		tool = "filter_events_by_constraints",
		preferences = ?preferences,
		constraints = ?constraints,
		events_count = events.len(),
		"Processing events with LLM-based filtering"
	);

		let mut filtered: Vec<Value> = Vec::new();
		let mut removed: Vec<Value> = Vec::new();

		// Process each event with LLM evaluation
		for ev in events.iter() {
			let event_name = ev.get("event_name").and_then(|v| v.as_str()).unwrap_or("Unknown");
			
			// Use LLM to evaluate if event should be included
			match should_include_event(&self.llm, ev, &preferences, &constraints).await {
				Ok((should_include, reason)) => {
					if should_include {
						filtered.push(ev.clone());
					} else {
						let reason_text = reason.unwrap_or_else(|| "not relevant for trip".to_string());
						removed.push(json!({
							"event": ev.clone(),
							"reasons": [reason_text],
						}));
					}
				},
				Err(e) => {
					debug!(
						target: "constraint_tools",
						tool = "filter_events_by_constraints",
						error = %e,
						event_name = %event_name,
						"LLM evaluation failed, including event by default"
					);
					// On error, include the event by default to avoid over-filtering
					filtered.push(ev.clone());
				}
			}
		}

	let result = json!({
		"filtered_events": filtered,
		"removed_events": removed
	});

	let elapsed = start_time.elapsed();

	// Extract event names for debugging
	let filtered_names: Vec<String> = result["filtered_events"]
		.as_array()
		.map(|arr| {
			arr.iter()
				.filter_map(|e| e.get("event_name").and_then(|n| n.as_str()).map(|s| s.to_string()))
				.collect()
		})
		.unwrap_or_default();
	
	let removed_names: Vec<String> = result["removed_events"]
		.as_array()
		.map(|arr| {
			arr.iter()
				.filter_map(|e| {
					e.get("event")
						.and_then(|ev| ev.get("event_name"))
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
			result["filtered_events"].as_array().map(|a| a.len()).unwrap_or(0),
			result["removed_events"].as_array().map(|a| a.len()).unwrap_or(0),
			filtered_names.join(", "),
			removed_names.join(", ")
		)
	);

		info!(
			target: "constraint_tools",
			tool = "filter_events_by_constraints",
			elapsed_ms = elapsed.as_millis() as u64,
			filtered_count = result["filtered_events"].as_array().map(|a| a.len()).unwrap_or(0),
			removed_count = result["removed_events"].as_array().map(|a| a.len()).unwrap_or(0),
			"Constraint filtering completed"
		);

		Ok(result.to_string())
	}
}

pub fn constraint_tools(llm: Arc<dyn LLM + Send + Sync>) -> Vec<Arc<dyn Tool>> {
	vec![Arc::new(FilterEventsByConstraintsTool::new(llm))]
}