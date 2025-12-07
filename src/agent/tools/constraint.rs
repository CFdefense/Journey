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
use serde_json::{json, Value};
use std::error::Error;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};

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
pub struct FilterEventsByConstraintsTool;

pub fn constraint_tools() -> Vec<Arc<dyn Tool>> {
	vec![Arc::new(FilterEventsByConstraintsTool)]
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

		// Extract events array
		let events_val = parsed_input
			.get("events")
			.cloned()
			.or_else(|| parsed_input.get("research_result").cloned())
			.or_else(|| parsed_input.get("data").cloned())
			.unwrap_or(Value::Null);

		let events = events_val
			.as_array()
			.ok_or("events should be an array of event-like objects")?;

		if events.is_empty() {
			return Err("No events provided to constraint agent".into());
		}

		// Extract constraints (strings, lowercased for matching)
		let constraints_val = if parsed_input.get("constraints").is_some() {
			parsed_input.get("constraints").cloned().unwrap_or(Value::Null)
		} else {
			parsed_input
				.get("trip_context")
				.and_then(|tc| tc.get("constraints"))
				.cloned()
				.unwrap_or(Value::Null)
		};

		let constraints: Vec<String> = if let Some(arr) = constraints_val.as_array() {
			arr.iter()
				.filter_map(|v| v.as_str().map(|s| s.to_lowercase()))
				.collect()
		} else {
			Vec::new()
		};

		// If we truly have no constraints, ask the user for them.
		if constraints.is_empty() {
			let question = "I have a list of candidate places for your trip, but I don't yet know your constraints or preferences (for example: accessibility needs, food allergies, or strict budget limits). Could you share any constraints you want me to respect when filtering these places?";
			info!(
				target: "constraint_tools",
				tool = "filter_events_by_constraints",
				"Missing constraints - asking user"
			);
			return Ok(question.to_string());
		}

		let wants_wheelchair = constraints
			.iter()
			.any(|c| c.contains("wheelchair") || c.contains("mobility"));

		let mut filtered: Vec<Value> = Vec::new();
		let mut removed: Vec<Value> = Vec::new();

		for ev in events.iter() {
			let mut reasons: Vec<String> = Vec::new();

			if wants_wheelchair {
				let accessible = ev
					.get("wheelchair_accessible")
					.and_then(|v| v.as_bool())
					.or_else(|| ev.get("wheelchair_accessible_entrance").and_then(|v| v.as_bool()))
					.or_else(|| ev.get("wheelchair_accessible_seating").and_then(|v| v.as_bool()))
					.unwrap_or(false);

				if !accessible {
					reasons.push("not wheelchair accessible".to_string());
				}
			}

			if reasons.is_empty() {
				filtered.push(ev.clone());
			} else {
				removed.push(json!({
					"event": ev.clone(),
					"reasons": reasons,
				}));
			}
		}

		let result = json!({
			"filtered_events": filtered,
			"removed_events": removed
		});

		let elapsed = start_time.elapsed();
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


