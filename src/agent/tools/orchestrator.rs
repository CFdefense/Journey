/*
 * src/agent/tools/orchestrator.rs
 *
 * Orchestrator Agent Tools Implementation - LLM-based extraction
 *
 * IMPORTANT: Tool Parameter Schema Pattern
 * ========================================
 * For parameters that can be objects/arrays, ALWAYS use "type": "string" in the schema.
 * Do NOT use "anyOf" or omit the type - langchain_rust 4.6.0 has a validation bug
 * that causes "invalid type: map, expected a string" errors during agent planning.
 *
 * Pattern to follow:
 * 1. Schema: Use "type": "string" for flexible parameters (objects/arrays)
 * 2. Description: Explicitly instruct LLM to pass JSON as strings
 * 3. Run method: Parse strings as JSON, but handle objects/arrays as fallback
 *
 * Example (see ask_for_clarification tool):
 *   "missing_info": {
 *     "type": "string",  // NOT "anyOf" or no type!
 *     "description": "...as a JSON string. Example: '[\"item\"]'"
 *   }
 *
 * This allows langchain_rust validation to pass while still accepting
 * whatever format the LLM generates (we handle both in run()).
 */

use crate::agent::models::context::{ContextData, SharedContextStore, ToolExecution};
use crate::agent::tools::task::RespondToUserTool;
use crate::sql_models::LlmProgress;
use async_trait::async_trait;
use langchain_rust::chain::Chain;
use langchain_rust::language_models::llm::LLM;
use langchain_rust::tools::Tool;
use serde_json::{Value, json};
use sqlx::PgPool;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

/// Helper function to automatically track tool executions in context.
/// This is called by every tool to record its execution in the tool_history.
///
/// Marked pub(crate) so it can be reused by Task tools without exposing it outside
/// the agent tools module.
pub(crate) async fn track_tool_execution(
	_context_store: &SharedContextStore,
	chat_session_id: &Arc<AtomicI32>,
	tool_name: &str,
	input: &Value,
	output: &str,
) -> Result<(), Box<dyn Error>> {
	let chat_id = chat_session_id.load(Ordering::Relaxed);
	if chat_id == 0 {
		// If chat_session_id is not set, we're probably in a test or the tool is being called outside the agent context
		return Ok(());
	}

	// Get existing in-memory context (should exist from controller initialization)
	let mut store_guard = _context_store.write().await;
	let context_data = match store_guard.get_mut(&chat_id) {
		Some(ctx) => ctx,
		None => {
			// Context doesn't exist - this shouldn't happen in normal flow
			// but create it to be safe
			store_guard.insert(
				chat_id,
				ContextData {
					chat_session_id: chat_id,
					user_id: 0, // Unknown
					user_profile: None,
					chat_history: vec![],
					trip_context: crate::agent::models::context::TripContext::default(),
					active_itinerary: None,
					events: vec![],
					tool_history: vec![],
					pipeline_stage: None,
					researched_events: vec![],
					constrained_events: vec![],
					optimized_events: vec![],
					constraints: vec![],
				},
			);
			store_guard.get_mut(&chat_id).unwrap()
		}
	};

	// Add tool execution to history
	let tool_exec = ToolExecution {
		tool_name: tool_name.to_string(),
		input: input.clone(),
		output: serde_json::from_str(output).unwrap_or_else(|_| json!(output)),
		timestamp: chrono::Utc::now().to_rfc3339(),
		success: true, // Assume success if the tool is calling this function
	};

	context_data.tool_history.push(tool_exec);

	// Keep only last 100 entries
	if context_data.tool_history.len() > 100 {
		context_data.tool_history.remove(0);
	}

	debug!(
		target: "orchestrator_tool",
		tool = tool_name,
		chat_id = chat_id,
		tool_history_count = context_data.tool_history.len(),
		"Tracked tool execution"
	);

	Ok(())
}

#[derive(Clone)]
pub struct RouteTaskTool {
	pub task_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
	pub research_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
	pub constraint_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
	pub optimize_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
	pool: PgPool,
	chat_session_id: Arc<AtomicI32>,
	context_store: SharedContextStore,
}

impl RouteTaskTool {
	pub fn new(
		task_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
		research_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
		constraint_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
		optimize_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
		pool: PgPool,
		chat_session_id: Arc<AtomicI32>,
		context_store: SharedContextStore,
	) -> Self {
		Self {
			task_agent,
			research_agent,
			constraint_agent,
			optimize_agent,
			pool,
			chat_session_id,
			context_store,
		}
	}
}

/// Tool 4: Route Task to Sub-Agent
/// Routes a task to the appropriate sub-agent (research, constraint, or optimize).
/// Returns a TaskRoute object.
#[async_trait]
impl Tool for RouteTaskTool {
	fn name(&self) -> String {
		"route_task".to_string()
	}

	fn description(&self) -> String {
		"Routes a task to the appropriate sub-agent (research, constraint, or optimize). IMPORTANT: All parameters must be passed as strings. For 'payload', pass the JSON object as a JSON string."
			.to_string()
	}

	fn parameters(&self) -> Value {
		let params = json!({
			"type": "object",
			"properties": {
				"task_type": {
					"type": "string",
					"enum": ["task", "research", "constraint", "optimize"],
					"description": "The type of task to route: 'task' (Task Agent for context building), 'research', 'constraint', or 'optimize'"
				},
				"payload": {
					"type": "string",
					"description": "The data to send to the sub-agent as a JSON string. If you have an object, serialize it to JSON first."
				}
			},
			"required": ["task_type", "payload"]
		});
		debug!(
			target: "orchestrator_tool",
			tool = "route_task",
			parameters = %serde_json::to_string(&params).unwrap_or_else(|_| "failed".to_string()),
			"Tool parameters schema"
		);
		params
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let input_clone = input.clone(); // Clone for tracking

		debug!(
			target: "orchestrator_tool",
			tool = "route_task",
			input_raw = %serde_json::to_string(&input).unwrap_or_else(|_| "failed to serialize".to_string()),
			"Received input in route_task"
		);

		// langchain_rust passes action_input as a STRING, so parse it first if needed
		let parsed_input: Value = if input.is_string() {
			// If input is a string (JSON string from action_input), parse it
			serde_json::from_str(input.as_str().unwrap_or("{}")).unwrap_or_else(|_| {
				debug!(
					target: "orchestrator_tool",
					tool = "route_task",
					"Failed to parse input as JSON, using as-is"
				);
				input.clone()
			})
		} else {
			// If it's already a Value object, use it directly
			input
		};

		// Handle task_type - prefer simple string, but be defensive about weird shapes
		//
		// In theory the LLM should always pass a plain string (\"task\", \"research\", etc.)
		// but in practice we have seen cases in logs where this ended up as \"null\"
		// and caused `Unknown task type: null` errors, preventing the Task Agent from running.
		//
		// Strategy:
		// - If we can read a string, use it directly.
		// - If it's an object, look for common fields (`value`, `task_type`).
		// - If it's missing / null / empty, *default to \"task\"* so first-turn
		//   orchestration still calls the Task Agent instead of hard failing.
		let raw_task_type_value = &parsed_input["task_type"];

		let mut task_type = if let Some(s) = raw_task_type_value.as_str() {
			s.to_string()
		} else if raw_task_type_value.is_object() {
			raw_task_type_value
				.get("value")
				.or_else(|| raw_task_type_value.get("task_type"))
				.and_then(|v| v.as_str())
				.map(|s| s.to_string())
				.unwrap_or_else(|| raw_task_type_value.to_string())
		} else if raw_task_type_value.is_null() {
			// This is the problematic case we've seen in logs; default to \"task\"
			"task".to_string()
		} else {
			raw_task_type_value.to_string()
		};

		debug!(
			target: "orchestrator_tool",
			tool = "route_task",
			raw_task_type = %serde_json::to_string(&raw_task_type_value)?,
			parsed_task_type = %task_type,
			"Parsed task_type from input"
		);

		// Handle payload - can be string (JSON), object, or already a string
		let payload_str = if let Some(s) = parsed_input["payload"].as_str() {
			// If it's a string, check if it's valid JSON, otherwise use as-is
			if serde_json::from_str::<Value>(s).is_ok() {
				s.to_string()
			} else {
				s.to_string()
			}
		} else if parsed_input["payload"].is_object() {
			// If it's an object, serialize it to string
			serde_json::to_string(&parsed_input["payload"])?
		} else {
			// Fallback: convert to string representation
			parsed_input["payload"].to_string()
		};

		info!(
			target: "orchestrator_tool",
			tool = "route_task",
			task_type = task_type,
			"Routing task to sub-agent"
		);
		debug!(
			target: "orchestrator_tool",
			tool = "route_task",
			input = %serde_json::to_string(&parsed_input)?,
			"Tool input"
		);

		// Normalize task_type to lowercase string and handle degenerate values
		let mut task_type_normalized = task_type.to_lowercase().trim().to_string();

		// If the normalized value is empty or literally \"null\", treat it as \"task\"
		// so that we still invoke the Task Agent instead of erroring.
		if task_type_normalized.is_empty() || task_type_normalized == "null" {
			debug!(
				target: "orchestrator_tool",
				tool = "route_task",
				raw_task_type = %raw_task_type_value,
				"task_type was empty or 'null'; defaulting to 'task'"
			);
			task_type_normalized = "task".to_string();
			task_type = "task".to_string();
		}

		// Tool trace logging
		crate::tool_trace!(
			agent: "orchestrator",
			tool: "route_task",
			status: "start",
			details: format!("task_type={}", task_type)
		);

		// Update LLM progress status in database BEFORE processing
		if let Some(progress) = match task_type_normalized.as_str() {
			"research" => Some(LlmProgress::Searching),
			"constraint" => Some(LlmProgress::Filtering),
			"optimize" => Some(LlmProgress::Optimizing),
			"task" => Some(LlmProgress::Scheduling),
			_ => None,
		} {
			let chat_session_id = self.chat_session_id.load(Ordering::Relaxed);
			info!(target: "orchestrator_pipeline", chat_session_id = chat_session_id, progress = ?progress, "Updating LLM progress");

			match sqlx::query!(
				r#"UPDATE chat_sessions
				SET llm_progress=$1
				WHERE id=$2;"#,
				progress as _,
				chat_session_id
			)
			.execute(&self.pool)
			.await
			{
				Ok(result) => {
					info!(target: "orchestrator_pipeline", chat_session_id = chat_session_id, rows_affected = result.rows_affected(), "LLM progress updated successfully");
				}
				Err(e) => {
					error!(target: "orchestrator_pipeline", chat_session_id = chat_session_id, error = %e, "Failed to update LLM progress");
				}
			}
		}

		// SPECIAL HANDLING: High-level Task Agent
		//
		// When task_type == "task", we want to delegate the entire planning pipeline
		// to the Task Agent and propagate its raw string output back to the controller.
		// This preserves markers like "MESSAGE_INSERTED:" and "FINAL_ANSWER:" so that
		// `send_message_to_llm` can handle them as before.
		if task_type_normalized == "task" {
			crate::tool_trace!(agent: "task", tool: "begin", status: "invoked");
			info!(target: "orchestrator_pipeline", agent = "task", "Invoking task agent");
			debug!(target: "orchestrator_pipeline", agent = "task", payload = %payload_str, "Agent input");

			let agent_outer = self.task_agent.lock().await;
			let agent_inner = agent_outer.lock().await;

			let response = match agent_inner
				.invoke(langchain_rust::prompt_args! {
					"input" => payload_str.as_str(),
				})
				.await
			{
				Ok(response) => {
					crate::tool_trace!(agent: "task", tool: "complete", status: "success");
					info!(target: "orchestrator_pipeline", agent = "task", status = "completed", "Task agent completed");
					debug!(target: "orchestrator_pipeline", agent = "task", response = %response, "Task agent raw output");
					response
				}
				Err(e) => {
					crate::tool_trace!(agent: "task", tool: "complete", status: "error", details: format!("{}", e));
					info!(target: "orchestrator_pipeline", agent = "task", status = "error", error = %e, "Task agent error");
					format!("TASK_AGENT_ERROR: {}", e)
				}
			};

			// Track this tool execution with a JSON wrapper for observability,
			// but return the raw response string so the controller can interpret it.
			let tracking_value = json!({
				"agent": "task",
				"status": if response.starts_with("TASK_AGENT_ERROR:") { "error" } else { "completed" },
				"raw": response,
			});
			let tracking_str = serde_json::to_string(&tracking_value)?;

			track_tool_execution(
				&self.context_store,
				&self.chat_session_id,
				"route_task",
				&input_clone,
				&tracking_str,
			)
			.await?;

			return Ok(response);
		}

		// For research/constraint/optimize agents, inject context from context_store
		let payload_str = if task_type_normalized == "research" {
			// Research gets the current trip_context snapshot
			let chat_id = self.chat_session_id.load(Ordering::Relaxed);
			if chat_id > 0 {
				let store_guard = self.context_store.read().await;
				if let Some(context_data) = store_guard.get(&chat_id) {
					let trip_context_json = serde_json::to_string(&context_data.trip_context)
						.unwrap_or_else(|_| "{}".to_string());

					info!(
						target: "orchestrator_pipeline",
						agent = "research",
						"Injecting trip context into payload"
					);
					debug!(
						target: "orchestrator_pipeline",
						trip_context = %trip_context_json,
						"Trip context being passed to research agent"
					);

					drop(store_guard);
					trip_context_json
				} else {
					payload_str
				}
			} else {
				payload_str
			}
		} else if task_type_normalized == "constraint" {
			// Constraint gets both trip context and the latest research results
			let chat_id = self.chat_session_id.load(Ordering::Relaxed);
			if chat_id > 0 {
				let store_guard = self.context_store.read().await;
				if let Some(context_data) = store_guard.get(&chat_id) {
					// Find latest successful research result from tool_history
					let mut research_data: Value = json!(null);
					for exec in context_data.tool_history.iter().rev() {
						if exec.tool_name == "route_task" {
							if let Some(agent) = exec.output.get("agent").and_then(|v| v.as_str()) {
								if agent == "research" {
									if exec.output.get("status").and_then(|v| v.as_str())
										== Some("completed")
									{
										research_data =
											exec.output.get("data").cloned().unwrap_or(json!(null));
										break;
									}
								}
							}
						}
					}

					// Extract event_ids from research data
					let event_ids = if let Some(ids) = research_data.get("event_ids") {
						ids.clone()
					} else {
						// Research data might be wrapped differently
						json!([])
					};

					let constraint_payload = json!({
						"trip_context": &context_data.trip_context,
						"constraints": &context_data.constraints,
						"event_ids": event_ids
					});

					let payload_json = serde_json::to_string(&constraint_payload)
						.unwrap_or_else(|_| "{}".to_string());

					info!(
						target: "orchestrator_pipeline",
						agent = "constraint",
						"Injecting trip context and research results into constraint payload"
					);
					debug!(
						target: "orchestrator_pipeline",
						payload = %payload_json,
						"Constraint payload being passed to agent"
					);

					drop(store_guard);
					payload_json
				} else {
					payload_str
				}
			} else {
				payload_str
			}
		} else if task_type_normalized == "optimize" {
			// Optimize gets trip context, user profile, and constraint results
			let chat_id = self.chat_session_id.load(Ordering::Relaxed);
			debug!(
				target: "orchestrator_pipeline",
				agent = "optimize",
				chat_id = chat_id,
				"Building optimize payload from context"
			);
			if chat_id > 0 {
				let store_guard = self.context_store.read().await;
				if let Some(context_data) = store_guard.get(&chat_id) {
					debug!(
						target: "orchestrator_pipeline",
						agent = "optimize",
						tool_history_count = context_data.tool_history.len(),
						"Found context data with tool history"
					);
					// Find latest successful constraint result from tool_history
					let mut constraint_data: Value = json!(null);
					for exec in context_data.tool_history.iter().rev() {
						if exec.tool_name == "route_task" {
							if let Some(agent) = exec.output.get("agent").and_then(|v| v.as_str()) {
								if agent == "constraint" {
									if exec.output.get("status").and_then(|v| v.as_str())
										== Some("completed")
									{
										constraint_data =
											exec.output.get("data").cloned().unwrap_or(json!(null));
										debug!(
											target: "orchestrator_pipeline",
											agent = "optimize",
											constraint_data = %serde_json::to_string(&constraint_data).unwrap_or_else(|_| "error".to_string()),
											"Found constraint result in tool_history"
										);
										break;
									}
								}
							}
						}
					}

					// Helper: best-effort extraction of filtered_event_ids from arbitrary text.
					//
					// In some failure cases the constraint agent returns *truncated* JSON inside
					// markdown fences, so `serde_json::from_str` cannot parse it. However, the
					// `filtered_event_ids` segment is usually intact near the top:
					//
					//   ..."filtered_event_ids":[26,9,12,...],\"removed_events\":[{...  (truncated)
					//
					// Rather than giving up, we scan the raw text for that slice and pull out the
					// comma-separated integers between the first '[' and the following ']'.
					fn extract_ids_from_text(text: &str) -> Value {
						if let Some(start) = text.find("filtered_event_ids") {
							let tail = &text[start..];
							if let Some(bracket_start) = tail.find('[') {
								let after_bracket = &tail[bracket_start + 1..];
								if let Some(bracket_end) = after_bracket.find(']') {
									let ids_segment = &after_bracket[..bracket_end];
									let ids: Vec<Value> = ids_segment
										.split(|c: char| c == ',' || c.is_whitespace())
										.filter_map(|s| s.trim().parse::<i64>().ok())
										.map(|n| json!(n as i32))
										.collect();

									return json!(ids);
								}
							}
						}

						// Fallback: nothing usable found
						json!([])
					}

					// Extract filtered_event_ids from constraint result
					//
					// We have seen a few different shapes in the wild:
					// 1. Direct JSON from the tool:
					//    {"filtered_event_ids":[...], "removed_events":[...], "count": N}
					// 2. Wrapped in a "raw" string by the constraint agent when it
					//    returns markdown code blocks:
					//    {"raw":"```json\n{ \"filtered_event_ids\":[...] ... }\n```"}
					// 3. Double-wrapped via the agent's "Final Answer" schema:
					//    {
					//      "raw":"```json\n{
					//        \"action\":\"Final Answer\",
					//        \"action_input\":\"{\\\"filtered_event_ids\\\":[...],...}\"
					//      }\n```"
					// 4. Or the whole thing as a plain string.
					//
					// The goal here is to be very defensive and recover the inner
					// JSON object that actually contains `filtered_event_ids`, no
					// matter how many layers of wrapping the LLM produced.
					let filtered_ids = if let Some(ids) = constraint_data.get("filtered_event_ids")
					{
						// Fast-path: already a proper object with filtered_event_ids
						ids.clone()
					} else if let Some(raw) = constraint_data.get("raw") {
						// Parse the raw string, handling markdown fences and optional
						// {"action": "...", "action_input": "..."} wrappers.
						if let Some(raw_str) = raw.as_str() {
							// Strip common markdown code fences the agents like to add
							let cleaned = raw_str
								.trim()
								.trim_start_matches("```json")
								.trim_start_matches("```")
								.trim_end_matches("```")
								.trim();

							if let Ok(parsed_outer) = serde_json::from_str::<Value>(cleaned) {
								// Case 2: direct object with filtered_event_ids
								if let Some(ids) = parsed_outer.get("filtered_event_ids") {
									ids.clone()
								} else if let Some(action_input) =
									parsed_outer.get("action_input").and_then(|v| v.as_str())
								{
									// Case 3: inner JSON string living in action_input
									if let Ok(inner) = serde_json::from_str::<Value>(action_input) {
										inner
											.get("filtered_event_ids")
											.cloned()
											.unwrap_or(json!([]))
									} else {
										// JSON inside action_input is malformed (often truncated) –
										// fall back to a best-effort text scrape.
										extract_ids_from_text(action_input)
									}
								} else {
									// No obvious JSON structure – last resort: scan the whole blob.
									extract_ids_from_text(cleaned)
								}
							} else {
								// Outer JSON is malformed – try text-based extraction directly.
								extract_ids_from_text(cleaned)
							}
						} else {
							json!([])
						}
					} else if constraint_data.is_string() {
						// Parse constraint result if it's a string itself. This can be
						// either the direct JSON or the agent "Final Answer" wrapper.
						let constraint_str = constraint_data.as_str().unwrap_or("{}");
						// Strip markdown fences first, just like above.
						let cleaned = constraint_str
							.trim()
							.trim_start_matches("```json")
							.trim_start_matches("```")
							.trim_end_matches("```")
							.trim();

						if let Ok(parsed_outer) = serde_json::from_str::<Value>(cleaned) {
							if let Some(ids) = parsed_outer.get("filtered_event_ids") {
								ids.clone()
							} else if let Some(action_input) =
								parsed_outer.get("action_input").and_then(|v| v.as_str())
							{
								if let Ok(inner) = serde_json::from_str::<Value>(action_input) {
									inner
										.get("filtered_event_ids")
										.cloned()
										.unwrap_or(json!([]))
								} else {
									extract_ids_from_text(action_input)
								}
							} else {
								extract_ids_from_text(cleaned)
							}
						} else {
							extract_ids_from_text(cleaned)
						}
					} else {
						json!([])
					};

					// Log what we extracted for debugging
					debug!(
						target: "orchestrator_pipeline",
						agent = "optimize",
						filtered_ids = %serde_json::to_string(&filtered_ids).unwrap_or_else(|_| "error".to_string()),
						"Extracted filtered_event_ids from constraint result"
					);

					let optimize_payload = json!({
						"trip_context": &context_data.trip_context,
						"user_profile": &context_data.user_profile,
						"filtered_event_ids": filtered_ids
					});

					let payload_json = serde_json::to_string(&optimize_payload)
						.unwrap_or_else(|_| "{}".to_string());

					info!(
						target: "orchestrator_pipeline",
						agent = "optimize",
						"Injecting trip context, user profile, and constraint results into optimize payload"
					);
					debug!(
						target: "orchestrator_pipeline",
						payload = %payload_json,
						"Optimize payload being passed to agent"
					);

					drop(store_guard);
					payload_json
				} else {
					debug!(
						target: "orchestrator_pipeline",
						agent = "optimize",
						chat_id = chat_id,
						"No context data found for chat_id, using original payload"
					);
					payload_str
				}
			} else {
				debug!(
					target: "orchestrator_pipeline",
					agent = "optimize",
					"chat_id is 0, using original payload"
				);
				payload_str
			}
		} else {
			payload_str
		};

		let result = match task_type_normalized.as_str() {
			"research" => {
				crate::tool_trace!(agent: "research", tool: "begin", status: "invoked");
				info!(target: "orchestrator_pipeline", agent = "research", "Invoking research agent");
				debug!(target: "orchestrator_pipeline", agent = "research", payload = %payload_str, "Agent input");

				let agent_outer = self.research_agent.lock().await;
				let agent_inner = agent_outer.lock().await;
				match agent_inner
					.invoke(langchain_rust::prompt_args! {
						"input" => payload_str.as_str(),
					})
					.await
				{
					Ok(response) => {
						// Parse response as JSON Value if possible
						let data: Value = serde_json::from_str(&response)
							.unwrap_or_else(|_| json!({ "raw": response }));

						crate::tool_trace!(agent: "research", tool: "complete", status: "success");
						info!(target: "orchestrator_pipeline", agent = "research", status = "completed", "Research agent completed");
						debug!(target: "orchestrator_pipeline", agent = "research", response = %serde_json::to_string(&data)?, "Agent output");

						json!({
							"agent": "research",
							"status": "completed",
							"data": data
						})
					}
					Err(e) => {
						crate::tool_trace!(agent: "research", tool: "complete", status: "error", details: format!("{}", e));
						info!(target: "orchestrator_pipeline", agent = "research", status = "error", error = %e, "Research agent error");
						json!({
							"agent": "research",
							"status": "error",
							"error": format!("{}", e)
						})
					}
				}
			}
			"constraint" => {
				crate::tool_trace!(agent: "constraint", tool: "begin", status: "invoked");
				info!(target: "orchestrator_pipeline", agent = "constraint", "Invoking constraint agent");
				debug!(target: "orchestrator_pipeline", agent = "constraint", payload = %payload_str, "Agent input");

				let agent_outer = self.constraint_agent.lock().await;
				let agent_inner = agent_outer.lock().await;
				match agent_inner
					.invoke(langchain_rust::prompt_args! {
						"input" => payload_str.as_str(),
					})
					.await
				{
					Ok(response) => {
						debug!(
							target: "orchestrator_pipeline",
							agent = "constraint",
							raw_response = %response,
							"Constraint agent raw response before parsing"
						);

						let data: Value = serde_json::from_str(&response)
							.unwrap_or_else(|_| json!({ "raw": response }));

						crate::tool_trace!(agent: "constraint", tool: "complete", status: "success");
						info!(target: "orchestrator_pipeline", agent = "constraint", status = "completed", "Constraint agent completed");
						debug!(target: "orchestrator_pipeline", agent = "constraint", response = %serde_json::to_string(&data)?, "Agent output");

						json!({
							"agent": "constraint",
							"status": "completed",
							"data": data
						})
					}
					Err(e) => {
						crate::tool_trace!(agent: "constraint", tool: "complete", status: "error", details: format!("{}", e));
						info!(target: "orchestrator_pipeline", agent = "constraint", status = "error", error = %e, "Constraint agent error");
						json!({
							"agent": "constraint",
							"status": "error",
							"error": format!("{}", e)
						})
					}
				}
			}
			"optimize" => {
				crate::tool_trace!(agent: "optimize", tool: "begin", status: "invoked");
				info!(target: "orchestrator_pipeline", agent = "optimize", "Invoking optimize agent");
				debug!(target: "orchestrator_pipeline", agent = "optimize", payload = %payload_str, "Agent input");

				let agent_outer = self.optimize_agent.lock().await;
				let agent_inner = agent_outer.lock().await;
				match agent_inner
					.invoke(langchain_rust::prompt_args! {
						"input" => payload_str.as_str(),
					})
					.await
				{
					Ok(response) => {
						debug!(
							target: "orchestrator_pipeline",
							agent = "optimize",
							raw_response = %response,
							"Optimize agent raw response before parsing"
						);

						let data: Value = serde_json::from_str(&response)
							.unwrap_or_else(|_| json!({ "raw": response }));

						// Store the complete itinerary in active_itinerary context
						let chat_id = self.chat_session_id.load(Ordering::Relaxed);
						if chat_id > 0 {
							let mut store_guard = self.context_store.write().await;
							if let Some(context_data) = store_guard.get_mut(&chat_id) {
								context_data.active_itinerary = Some(data.clone());
								info!(
									target: "orchestrator_pipeline",
									agent = "optimize",
									"Stored itinerary in active_itinerary context"
								);
								debug!(
									target: "orchestrator_pipeline",
									itinerary = %serde_json::to_string(&data).unwrap_or_else(|_| "error".to_string()),
									"Itinerary stored in context"
								);
							}
							drop(store_guard);
						}

						crate::tool_trace!(agent: "optimize", tool: "complete", status: "success");
						info!(target: "orchestrator_pipeline", agent = "optimize", status = "completed", "Optimize agent completed");
						debug!(target: "orchestrator_pipeline", agent = "optimize", response = %serde_json::to_string(&data)?, "Agent output");

						json!({
							"agent": "optimize",
							"status": "completed",
							"data": data
						})
					}
					Err(e) => {
						crate::tool_trace!(agent: "optimize", tool: "complete", status: "error", details: format!("{}", e));
						info!(target: "orchestrator_pipeline", agent = "optimize", status = "error", error = %e, "Optimize agent error");
						json!({
							"agent": "optimize",
							"status": "error",
							"error": format!("{}", e)
						})
					}
				}
			}
			_ => {
				return Err(format!("Unknown task type: {}", task_type).into());
			}
		};

		let result_str = serde_json::to_string(&result)?;

		info!(
			target: "orchestrator_tool",
			tool = "route_task",
			task_type = task_type,
			status = result.get("status").and_then(|s| s.as_str()).unwrap_or("unknown"),
			"Task routing completed"
		);
		debug!(
			target: "orchestrator_tool",
			tool = "route_task",
			result = %result_str,
			"Tool output"
		);

		// Track this tool execution
		track_tool_execution(
			&self.context_store,
			&self.chat_session_id,
			"route_task",
			&input_clone,
			&result_str,
		)
		.await?;

		Ok(result_str)
	}
}

/// Gets all the orchestrator tools.
/// Returns a vector of Arc<dyn Tool> objects.
/// chat_session_id and user_id are shared across tools that need them and can be updated per request.
pub fn get_orchestrator_tools(
	_llm: Arc<dyn LLM + Send + Sync>,
	pool: PgPool,
	task_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
	research_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
	constraint_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
	optimize_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
	chat_session_id: Arc<AtomicI32>,
	_user_id: Arc<AtomicI32>,
	context_store: SharedContextStore,
) -> Vec<Arc<dyn Tool>> {
	vec![
		Arc::new(RouteTaskTool::new(
			task_agent,
			research_agent,
			constraint_agent,
			optimize_agent,
			pool.clone(),
			Arc::clone(&chat_session_id),
			context_store.clone(),
		)),
		Arc::new(RespondToUserTool::new(pool, chat_session_id, context_store)),
		// Note: context-building tools (profile, chat history, intent, clarification)
		// are exposed via the Task Agent through `get_task_tools` and should not be
		// called directly by the Orchestrator.
	]
}
