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

use crate::agent::models::context::{ContextData, ToolExecution};
use crate::agent::tools::task::RespondToUserTool;
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
use tracing::{debug, info};

/// Helper function to automatically track tool executions in context.
/// This is called by every tool to record its execution in the tool_history.
/// 
/// Marked pub(crate) so it can be reused by Task tools without exposing it outside
/// the agent tools module.
pub(crate) async fn track_tool_execution(
	pool: &PgPool,
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

	// Get existing context
	let context_row = sqlx::query!(
		r#"SELECT context as "context: serde_json::Value" FROM chat_sessions WHERE id = $1"#,
		chat_id
	)
	.fetch_optional(pool)
	.await
	.map_err(|e| format!("Database error: {}", e))?;

	let mut context_data = if let Some(row) = context_row {
		if let Some(ctx) = row.context {
			serde_json::from_value::<ContextData>(ctx.clone())
				.unwrap_or_else(|_| ContextData {
					user_profile: None,
					chat_history: vec![],
					active_itinerary: None,
					events: vec![],
					tool_history: vec![],
					pipeline_stage: None,
					researched_events: vec![],
					constrained_events: vec![],
					optimized_events: vec![],
					constraints: vec![],
				})
		} else {
			ContextData {
				user_profile: None,
				chat_history: vec![],
				active_itinerary: None,
				events: vec![],
				tool_history: vec![],
				pipeline_stage: None,
				researched_events: vec![],
				constrained_events: vec![],
				optimized_events: vec![],
				constraints: vec![],
			}
		}
	} else {
		ContextData {
			user_profile: None,
			chat_history: vec![],
			active_itinerary: None,
			events: vec![],
			tool_history: vec![],
			pipeline_stage: None,
			researched_events: vec![],
			constrained_events: vec![],
			optimized_events: vec![],
			constraints: vec![],
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

	// Save updated context to database
	let context_json = serde_json::to_value(&context_data)?;
	sqlx::query!(
		r#"UPDATE chat_sessions SET context = $1::jsonb WHERE id = $2"#,
		context_json as serde_json::Value,
		chat_id
	)
	.execute(pool)
	.await
	.map_err(|e| format!("Database error: {}", e))?;

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
}

impl RouteTaskTool {
	pub fn new(
		task_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
		research_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
		constraint_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
		optimize_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
		pool: PgPool,
		chat_session_id: Arc<AtomicI32>,
	) -> Self {
		Self {
			task_agent,
			research_agent,
			constraint_agent,
			optimize_agent,
			pool,
			chat_session_id,
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
		
		// Handle task_type - can be string, object, or JSON string
		let task_type = if let Some(s) = input["task_type"].as_str() {
			// Try to parse as JSON object first, then extract value
			if let Ok(obj) = serde_json::from_str::<Value>(s) {
				if obj.is_object() {
					obj.get("value")
						.or_else(|| obj.get("task_type"))
						.and_then(|v| v.as_str())
						.map(|s| s.to_string())
						.unwrap_or_else(|| s.to_string())
				} else {
					s.to_string()
				}
			} else {
				s.to_string()
			}
		} else if input["task_type"].is_object() {
			// Try to extract value from object
			input["task_type"].get("value")
				.or_else(|| input["task_type"].get("task_type"))
				.and_then(|v| v.as_str())
				.map(|s| s.to_string())
				.unwrap_or_else(|| serde_json::to_string(&input["task_type"]).unwrap_or_else(|_| "unknown".to_string()))
		} else {
			input["task_type"].to_string()
		};
		
		// Handle payload - can be string (JSON), object, or already a string
		let payload_str = if let Some(s) = input["payload"].as_str() {
			// If it's a string, check if it's valid JSON, otherwise use as-is
			if serde_json::from_str::<Value>(s).is_ok() {
				s.to_string()
			} else {
				s.to_string()
			}
		} else if input["payload"].is_object() {
			// If it's an object, serialize it to string
			serde_json::to_string(&input["payload"])?
		} else {
			// Fallback: convert to string representation
			input["payload"].to_string()
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
			input = %serde_json::to_string(&input)?,
			"Tool input"
		);

		// Normalize task_type to lowercase string
		let task_type_normalized = task_type.to_lowercase().trim().to_string();

		// SPECIAL HANDLING: High-level Task Agent
		//
		// When task_type == "task", we want to delegate the entire planning pipeline
		// to the Task Agent and propagate its raw string output back to the controller.
		// This preserves markers like "MESSAGE_INSERTED:" and "FINAL_ANSWER:" so that
		// `send_message_to_llm` can handle them as before.
		if task_type_normalized == "task" {
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
					info!(target: "orchestrator_pipeline", agent = "task", status = "completed", "Task agent completed");
					debug!(target: "orchestrator_pipeline", agent = "task", response = %response, "Task agent raw output");
					response
				}
				Err(e) => {
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
				&self.pool,
				&self.chat_session_id,
				"route_task",
				&input_clone,
				&tracking_str,
			)
			.await?;

			return Ok(response);
		}

		let result = match task_type_normalized.as_str() {
			"research" => {
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
						
						info!(target: "orchestrator_pipeline", agent = "research", status = "completed", "Research agent completed");
						debug!(target: "orchestrator_pipeline", agent = "research", response = %serde_json::to_string(&data)?, "Agent output");
						
						json!({
							"agent": "research",
							"status": "completed",
							"data": data
						})
					}
					Err(e) => {
						info!(target: "orchestrator_pipeline", agent = "research", status = "error", error = %e, "Research agent error");
						json!({
							"agent": "research",
							"status": "error",
							"error": format!("{}", e)
						})
					},
				}
			}
			"constraint" => {
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
						let data: Value = serde_json::from_str(&response)
							.unwrap_or_else(|_| json!({ "raw": response }));
						
						info!(target: "orchestrator_pipeline", agent = "constraint", status = "completed", "Constraint agent completed");
						debug!(target: "orchestrator_pipeline", agent = "constraint", response = %serde_json::to_string(&data)?, "Agent output");
						
						json!({
							"agent": "constraint",
							"status": "completed",
							"data": data
						})
					}
					Err(e) => {
						info!(target: "orchestrator_pipeline", agent = "constraint", status = "error", error = %e, "Constraint agent error");
						json!({
							"agent": "constraint",
							"status": "error",
							"error": format!("{}", e)
						})
					},
				}
			}
			"optimize" => {
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
						let data: Value = serde_json::from_str(&response)
							.unwrap_or_else(|_| json!({ "raw": response }));
						
						info!(target: "orchestrator_pipeline", agent = "optimize", status = "completed", "Optimize agent completed");
						debug!(target: "orchestrator_pipeline", agent = "optimize", response = %serde_json::to_string(&data)?, "Agent output");
						
						json!({
							"agent": "optimize",
							"status": "completed",
							"data": data
						})
					}
					Err(e) => {
						info!(target: "orchestrator_pipeline", agent = "optimize", status = "error", error = %e, "Optimize agent error");
						json!({
							"agent": "optimize",
							"status": "error",
							"error": format!("{}", e)
						})
					},
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
		track_tool_execution(&self.pool, &self.chat_session_id, "route_task", &input_clone, &result_str).await?;
		
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
) -> Vec<Arc<dyn Tool>> {
	vec![
		Arc::new(RouteTaskTool::new(
			task_agent,
			research_agent,
			constraint_agent,
			optimize_agent,
			pool.clone(),
			Arc::clone(&chat_session_id),
		)),
		Arc::new(RespondToUserTool::new(pool, chat_session_id)),
		// Note: context-building tools (profile, chat history, intent, clarification)
		// are exposed via the Task Agent through `get_task_tools` and should not be
		// called directly by the Orchestrator.
	]
}