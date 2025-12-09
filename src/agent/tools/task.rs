/*
 * src/agent/tools/task.rs
 *
 * Task Agent Tools Implementation - context and intent helpers
 *
 * These tools are focused on:
 * - retrieving user profile
 * - retrieving chat history/context
 * - parsing user intent
 * - asking for clarification when information is missing
 * - responding to the user
 *
 * They are used by the Task Agent and are intentionally kept separate
 * from the Orchestrator-specific tools.
 */

use crate::agent::models::context::{ContextData, SharedContextStore};
use crate::agent::models::user::UserIntent;
use crate::agent::tools::orchestrator::track_tool_execution;
use crate::controllers::itinerary::insert_event_list;
use crate::http_models::itinerary::Itinerary as HttpItinerary;
use crate::sql_models::LlmProgress;
use async_trait::async_trait;
use chrono::Datelike;
use langchain_rust::language_models::llm::LLM;
use langchain_rust::tools::Tool;
use serde_json::{Value, json};
use sqlx::PgPool;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Instant;
use tracing::{debug, info};

/// Tool 1: Parse User Intent
/// Parses user input to extract intent, destination, dates, budget, and constraints.
/// Returns a UserIntent object.
#[derive(Clone)]
pub struct ParseUserIntentTool {
	llm: Arc<dyn LLM + Send + Sync>,
	chat_session_id: Arc<AtomicI32>,
	context_store: SharedContextStore,
}

impl ParseUserIntentTool {
	pub fn new(
		llm: Arc<dyn LLM + Send + Sync>,
		chat_session_id: Arc<AtomicI32>,
		context_store: SharedContextStore,
	) -> Self {
		Self {
			llm,
			chat_session_id,
			context_store,
		}
	}
}

#[async_trait]
impl Tool for ParseUserIntentTool {
	fn name(&self) -> String {
		"parse_user_intent".to_string()
	}

	fn description(&self) -> String {
		"Parses user input using an LLM to extract intent, destination, dates, budget, preferences, and constraints. Returns a UserIntent object with constraints array that should be stored in context for other agents to access. IMPORTANT: If you have retrieved chat context, include the recent conversation history in your analysis to extract information from previous messages."
             .to_string()
	}

	fn parameters(&self) -> Value {
		let params = json!({
			"type": "object",
			"properties": {
				"user_message": {
					"type": "string",
					"description": "The raw user message to parse. If you have conversation history from retrieve_chat_context, include the recent messages (last 3-5 exchanges) as context in this field to help extract information from previous messages."
				}
			},
			"required": ["user_message"]
		});
		debug!(
			target: "orchestrator_tool",
			tool = "parse_user_intent",
			parameters = %serde_json::to_string(&params).unwrap_or_else(|_| "failed".to_string()),
			"Tool parameters schema"
		);
		params
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let start_time = Instant::now();
		let input_clone = input.clone(); // Clone for tracking

		crate::tool_trace!(agent: "task", tool: "parse_user_intent", status: "start");

		debug!(
			target: "orchestrator_tool",
			tool = "parse_user_intent",
			input_raw = %serde_json::to_string(&input).unwrap_or_else(|_| "failed to serialize".to_string()),
			"Received input in parse_user_intent"
		);

		// Handle user_message as string or object (convert object to string)
		let user_message = if let Some(s) = input["user_message"].as_str() {
			s.to_string()
		} else if input["user_message"].is_object() {
			serde_json::to_string(&input["user_message"])?
		} else {
			input["user_message"].to_string()
		};

		let prompt = format!(
			r#"Extract travel planning information from the user's conversation history.

CRITICAL: You will receive either:
1. A JSON string containing a chat_history array with role/content messages
2. A JSON object with destination/dates/budget fields that you should parse
3. Plain text describing the travel request

Your job is to extract ALL information mentioned in ANY format.

User input: {}

Extract the following information and return ONLY a valid JSON object with these fields:
{{
  "action": "create_itinerary" | "modify_itinerary" | "query" | "other",
  "destination": string or null (extract from ANY field - look for country/city names like "brazil", "paris", "destination", etc.),
  "start_date": string or null (ISO format YYYY-MM-DD if mentioned - look in "dates", "start_date", or message content),
  "end_date": string or null (ISO format YYYY-MM-DD if mentioned - look in "dates", "end_date", or message content),
  "budget": number or null (total budget in USD - look in "budget" field or dollar amounts in messages. Use midpoint for ranges like "20-30"),
  "preferences": [array of strings - look in "preferences" field or message content for activities, interests],
  "constraints": [array of strings - dietary restrictions, accessibility needs found anywhere],
  "missing_info": [array of strings - list ONLY what is truly missing. If destination/dates/budget appear ANYWHERE, they are NOT missing]
}}

Rules:
- If input has a "chat_history" array, read ALL messages in it
- If input has direct fields like "destination", "dates", "budget", extract those
- If input is plain text, parse it directly
- For "july 20-30th" or "june 10-20", extract as start_date "2026-07-20" and end_date "2026-07-30" (year 2026 since we're in Dec 2025)
- For budget ranges like "20-30 dollars", use the midpoint: 25
- If preferences say "no preferences" or similar, use empty array but don't list it as missing
- missing_info should ONLY contain items that are completely absent from the input

Return ONLY the JSON object, no other text."#,
			user_message
		);

		let response = self.llm.invoke(&prompt).await?;

		// Clean up the response - remove markdown code blocks if present
		let cleaned = response
			.trim()
			.trim_start_matches("```json")
			.trim_start_matches("```")
			.trim_end_matches("```")
			.trim();

		// Validate it's proper JSON and return as UserIntent
		let intent: UserIntent = serde_json::from_str(cleaned).map_err(|e| {
			format!(
				"Failed to parse LLM response as JSON: {}. Response was: {}",
				e, cleaned
			)
		})?;

		info!(
			target: "orchestrator_tool",
			tool = "parse_user_intent",
			action = %intent.action,
			destination = ?intent.destination,
			start_date = ?intent.start_date,
			end_date = ?intent.end_date,
			budget = ?intent.budget,
			preferences_count = intent.preferences.len(),
			constraints_count = intent.constraints.len(),
			missing_info = ?intent.missing_info,
			"Parsed user intent successfully"
		);
		debug!(
			target: "orchestrator_tool",
			tool = "parse_user_intent",
			intent = %serde_json::to_string(&intent)?,
			"Full parsed intent"
		);

		// Return serialized UserIntent
		let result = serde_json::to_string(&intent)?;

		let elapsed = start_time.elapsed();
		info!(
			target: "orchestrator_tool",
			tool = "parse_user_intent",
			elapsed_ms = elapsed.as_millis() as u64,
			"Tool completed"
		);

		// Track this tool execution
		track_tool_execution(
			&self.context_store,
			&self.chat_session_id,
			"parse_user_intent",
			&input_clone,
			&result,
		)
		.await?;

		Ok(result)
	}
}

/// Tool 2: Retrieve Chat Context
/// Retrieves chat history and context for the current conversation.
/// Returns a vector of Message objects.
#[derive(Clone)]
pub struct RetrieveChatContextTool {
	pool: PgPool,
	chat_session_id: Arc<AtomicI32>,
	context_store: SharedContextStore,
}

impl RetrieveChatContextTool {
	pub fn new(
		pool: PgPool,
		chat_session_id: Arc<AtomicI32>,
		context_store: SharedContextStore,
	) -> Self {
		Self {
			pool,
			chat_session_id,
			context_store,
		}
	}
}

#[async_trait]
impl Tool for RetrieveChatContextTool {
	fn name(&self) -> String {
		"retrieve_chat_context".to_string()
	}

	fn description(&self) -> String {
		"Retrieves chat history and context for the current conversation.".to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {},
			"required": []
		})
	}

	async fn run(&self, _input: Value) -> Result<String, Box<dyn Error>> {
		let start_time = Instant::now();
		let input_clone = _input.clone(); // Clone for tracking

		crate::tool_trace!(agent: "task", tool: "retrieve_chat_context", status: "start");

		// Get chat_session_id from shared atomic (set by controller before agent invocation)
		let chat_id = self.chat_session_id.load(Ordering::Relaxed);
		if chat_id == 0 {
			return Err("chat_session_id not set. This should be set by the controller before invoking the agent.".into());
		}

		debug!(
			target: "orchestrator_tool",
			tool = "retrieve_chat_context",
			chat_id = chat_id,
			"Retrieving chat context"
		);

		// Query database for chat history
		let messages = sqlx::query!(
			r#"
			SELECT
				m.id,
				m.is_user,
				m.timestamp,
				m.text,
				m.itinerary_id
			FROM messages m
			WHERE m.chat_session_id = $1
			ORDER BY m.timestamp ASC
			LIMIT 50
			"#,
			chat_id
		)
		.fetch_all(&self.pool)
		.await
		.map_err(|e| format!("Database error: {}", e))?;

		let chat_history: Vec<Value> = messages
			.into_iter()
			.map(|msg| {
				json!({
					"id": msg.id,
					"role": if msg.is_user { "user" } else { "assistant" },
					"content": msg.text,
					"timestamp": msg.timestamp.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
					"itinerary_id": msg.itinerary_id
				})
			})
			.collect();

		// Retrieve or initialize in-memory context (includes pipeline state and events)
		let mut store_guard = self.context_store.write().await;
		let context_data = match store_guard.get_mut(&chat_id) {
			Some(ctx) => ctx,
			None => {
				// Context doesn't exist - create it
				store_guard.insert(
					chat_id,
					ContextData {
						chat_session_id: chat_id,
						user_id: 0,
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

		// Update chat_history with the messages we just retrieved
		context_data.chat_history = chat_history.clone();

		info!(
			target: "orchestrator_tool",
			tool = "retrieve_chat_context",
			chat_id = chat_id,
			chat_history_count = context_data.chat_history.len(),
			pipeline_stage = ?context_data.pipeline_stage,
			events_count = context_data.events.len(),
			constraints_count = context_data.constraints.len(),
			"Retrieved chat context"
		);

		let result = serde_json::to_string(&context_data.clone())?;

		debug!(
			target: "orchestrator_tool",
			tool = "retrieve_chat_context",
			context = %result,
			"Full context data"
		);

		// Log trip_context specifically for debugging
		if let Ok(context_obj) = serde_json::from_str::<Value>(&result) {
			if let Some(trip_ctx) = context_obj.get("trip_context") {
				info!(
					target: "trip_context",
					tool = "retrieve_chat_context",
					chat_id = chat_id,
					"Retrieved trip_context from database",
				);
				debug!(
					target: "trip_context",
					trip_context = %serde_json::to_string_pretty(&trip_ctx).unwrap_or_else(|_| "error".to_string()),
					"Trip context at retrieve_chat_context"
				);
			}
		}

		// Return full context including pipeline state
		drop(store_guard);

		let elapsed = start_time.elapsed();
		info!(
			target: "orchestrator_tool",
			tool = "retrieve_chat_context",
			elapsed_ms = elapsed.as_millis() as u64,
			chat_history_count = chat_history.len(),
			"Tool completed"
		);

		// Track this tool execution
		track_tool_execution(
			&self.context_store,
			&self.chat_session_id,
			"retrieve_chat_context",
			&input_clone,
			&result,
		)
		.await?;

		Ok(result)
	}
}

/// Tool 3: Retrieve User Profile
/// Retrieves user profile information including preferences and past trips.
/// Returns a UserProfile object.
#[derive(Clone)]
pub struct RetrieveUserProfileTool {
	pool: PgPool,
	chat_session_id: Arc<AtomicI32>,
	#[allow(dead_code)]
	user_id: Arc<AtomicI32>,
	context_store: SharedContextStore,
}

impl RetrieveUserProfileTool {
	pub fn new(
		pool: PgPool,
		chat_session_id: Arc<AtomicI32>,
		user_id: Arc<AtomicI32>,
		context_store: SharedContextStore,
	) -> Self {
		Self {
			pool,
			chat_session_id,
			user_id,
			context_store,
		}
	}
}

#[async_trait]
impl Tool for RetrieveUserProfileTool {
	fn name(&self) -> String {
		"retrieve_user_profile".to_string()
	}

	fn description(&self) -> String {
		"Retrieves user profile information including preferences and past trips. Automatically uses the logged-in user's ID."
			.to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {},
			"required": []
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let start_time = Instant::now();
		let input_clone = input.clone(); // Clone for tracking

		crate::tool_trace!(agent: "task", tool: "retrieve_user_profile", status: "start");

		debug!(
			target: "orchestrator_tool",
			tool = "retrieve_user_profile",
			input_raw = %serde_json::to_string(&input).unwrap_or_else(|_| "failed to serialize".to_string()),
			"Received input in retrieve_user_profile"
		);

		// Get chat_session_id from atomic
		let chat_id = self.chat_session_id.load(Ordering::Relaxed);
		if chat_id == 0 {
			return Err("chat_session_id not set".into());
		}

		// Get user_id from context (safer than atomics - no race conditions)
		let user_id = {
			let store_guard = self.context_store.read().await;
			store_guard
				.get(&chat_id)
				.map(|ctx| ctx.user_id)
				.unwrap_or(0)
		};

		if user_id == 0 {
			// In some flows (e.g., tests or unauthenticated calls) we may not have
			// a user_id. Treat this as "no profile available" instead of a hard
			// error so the Task Agent can still proceed and rely on chat history.
			info!(
				target: "orchestrator_tool",
				tool = "retrieve_user_profile",
				chat_id = chat_id,
				"User ID not set in context; proceeding with empty profile"
			);

			let empty_profile = json!({
				"user_id": null,
				"email": null,
				"first_name": null,
				"last_name": null,
				"budget_preference": null,
				"risk_preference": null,
				"food_allergies": "",
				"disabilities": ""
			});

			// Save empty profile into in-memory context for this chat (if any)
			let mut store_guard = self.context_store.write().await;
			if let Some(context_data) = store_guard.get_mut(&chat_id) {
				context_data.user_profile = Some(empty_profile.clone());
			}

			let result = serde_json::to_string(&empty_profile)?;

			let elapsed = start_time.elapsed();
			info!(
				target: "orchestrator_tool",
				tool = "retrieve_user_profile",
				elapsed_ms = elapsed.as_millis() as u64,
				"Tool completed (no user logged in)"
			);

			track_tool_execution(
				&self.context_store,
				&self.chat_session_id,
				"retrieve_user_profile",
				&input_clone,
				&result,
			)
			.await?;

			return Ok(result);
		}

		info!(target: "orchestrator_tool", tool = "retrieve_user_profile", user_id = user_id, "Retrieving user profile");
		debug!(target: "orchestrator_tool", tool = "retrieve_user_profile", input = %serde_json::to_string(&input)?, "Tool input");

		// Query database for user profile
		use crate::sql_models::{BudgetBucket, RiskTolerence};
		let account = sqlx::query_as!(
			crate::http_models::account::CurrentResponse,
			r#"
			SELECT
				email,
				first_name,
				last_name,
				budget_preference as "budget_preference: BudgetBucket",
				risk_preference as "risk_preference: RiskTolerence",
				COALESCE(food_allergies, '') as "food_allergies!: String",
				COALESCE(disabilities, '') as "disabilities!: String",
				COALESCE(profile_picture, '') as "profile_picture!: String"
			FROM accounts
			WHERE id = $1
			"#,
			user_id
		)
		.fetch_optional(&self.pool)
		.await
		.map_err(|e| format!("Database error: {}", e))?;

		let profile = if let Some(acc) = account {
			json!({
				"user_id": user_id,
				"email": acc.email,
				"first_name": acc.first_name,
				"last_name": acc.last_name,
				"budget_preference": acc.budget_preference,
				"risk_preference": acc.risk_preference,
				"food_allergies": acc.food_allergies,
				"disabilities": acc.disabilities
			})
		} else {
			return Err(format!("User with id {} not found", user_id).into());
		};

		// Automatically save user profile to in-memory context AND pre-fill trip context
		let chat_id = self.chat_session_id.load(Ordering::Relaxed);
		if chat_id != 0 {
			// Get existing in-memory context
			let mut store_guard = self.context_store.write().await;
			if let Some(context_data) = store_guard.get_mut(&chat_id) {
				context_data.user_profile = Some(profile.clone());

				// Pre-fill trip_context constraints from user profile
				let mut constraints = Vec::new();

				// Add food allergies as constraints
				if let Some(allergies) = profile.get("food_allergies").and_then(|v| v.as_str()) {
					if !allergies.is_empty() {
						for allergy in allergies.split(',') {
							let allergy_trimmed = allergy.trim();
							if !allergy_trimmed.is_empty() {
								constraints.push(format!("No {}", allergy_trimmed));
							}
						}
					}
				}

				// Add disabilities as constraints
				if let Some(disabilities) = profile.get("disabilities").and_then(|v| v.as_str()) {
					if !disabilities.is_empty() {
						constraints
							.push(format!("Wheelchair accessible required: {}", disabilities));
					}
				}

				// Store constraints in trip_context
				context_data.trip_context.constraints = constraints.clone();

				// Also store in the legacy constraints field for backward compatibility
				context_data.constraints = constraints;

				info!(
					target: "orchestrator_tool",
					tool = "retrieve_user_profile",
					chat_id = chat_id,
					user_id = user_id,
					constraints_count = context_data.trip_context.constraints.len(),
					"Saved user profile to context and pre-filled trip constraints"
				);
				debug!(
					target: "trip_context",
					tool = "retrieve_user_profile",
					constraints = ?context_data.trip_context.constraints,
					"Pre-filled constraints from user profile"
				);
			}
		}

		let result = serde_json::to_string(&profile)?;

		let elapsed = start_time.elapsed();
		info!(
			target: "orchestrator_tool",
			tool = "retrieve_user_profile",
			elapsed_ms = elapsed.as_millis() as u64,
			user_id = user_id,
			"Tool completed"
		);

		// Track this tool execution
		track_tool_execution(
			&self.context_store,
			&self.chat_session_id,
			"retrieve_user_profile",
			&input_clone,
			&result,
		)
		.await?;

		Ok(result)
	}
}

/// Tool: Ask for Clarification
/// Generates a natural clarification question using an LLM when user input is ambiguous.
/// STOPS THE PIPELINE by inserting the clarification message into the chat and returning success.
#[derive(Clone)]
pub struct AskForClarificationTool {
	llm: Arc<dyn LLM + Send + Sync>,
	pool: PgPool,
	chat_session_id: Arc<AtomicI32>,
	context_store: SharedContextStore,
}

impl AskForClarificationTool {
	pub fn new(
		llm: Arc<dyn LLM + Send + Sync>,
		pool: PgPool,
		chat_session_id: Arc<AtomicI32>,
		context_store: SharedContextStore,
	) -> Self {
		Self {
			llm,
			pool,
			chat_session_id,
			context_store,
		}
	}
}

#[async_trait]
impl Tool for AskForClarificationTool {
	fn name(&self) -> String {
		"ask_for_clarification".to_string()
	}

	fn description(&self) -> String {
		"STOPS THE PIPELINE by generating a natural, human-readable clarification question and sending it to the user. This tool inserts a message into the chat and returns the readable question text. Use this when critical information is missing. CRITICAL STOPPING RULE: After calling this tool, you MUST immediately return 'Final Answer' with the EXACT text returned by this tool. DO NOT call this tool again. DO NOT call any other tools. DO NOT call retrieve_chat_context or parse_user_intent after this. The tool returns ONLY the readable question text - use that EXACT text as your Final Answer. This is your FINAL response to the user - stop immediately after receiving the tool response. Always provide the missing_info parameter as a JSON string array (e.g., '[\"destination\", \"dates\", \"budget\"]'). If missing_info is not provided, the tool will use default common missing information."
			.to_string()
	}

	fn parameters(&self) -> Value {
		let params = json!({
			"type": "object",
			"properties": {
				"missing_info": {
					"type": "string",
					"description": "Array of strings describing what information is missing, as a JSON string. Example: '[\"destination\", \"dates\", \"budget\"]'. If you have an array, serialize it to JSON first."
				},
				"context": {
					"type": "string",
					"description": "Additional context about the conversation as a JSON string. Optional."
				}
			},
			"required": []
		});
		debug!(
			target: "orchestrator_tool",
			tool = "ask_for_clarification",
			parameters = %serde_json::to_string(&params).unwrap_or_else(|_| "failed".to_string()),
			"Tool parameters schema"
		);
		params
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let start_time = Instant::now();
		let input_clone = input.clone(); // Clone for tracking

		crate::tool_trace!(agent: "task", tool: "ask_for_clarification", status: "start");

		debug!(
			target: "orchestrator_tool",
			tool = "ask_for_clarification",
			input_raw = %serde_json::to_string(&input).unwrap_or_else(|_| "failed to serialize".to_string()),
			"Received input in ask_for_clarification"
		);

		// langchain_rust passes action_input as a STRING, so we need to parse it first
		let parsed_input: Value = if input.is_string() {
			// If input is a string (JSON string from action_input), parse it
			serde_json::from_str(input.as_str().unwrap_or("{}")).unwrap_or_else(|_| json!({}))
		} else {
			// If it's already a Value object, use it directly
			input
		};

		// Handle missing_info - can be array, string, object, or missing
		debug!(
			target: "orchestrator_tool",
			tool = "ask_for_clarification",
			missing_info_type = ?parsed_input.get("missing_info").map(|v| {
				if v.is_array() { "array" }
				else if v.is_string() { "string" }
				else if v.is_object() { "object" }
				else { "other" }
			}),
			missing_info_value = ?parsed_input.get("missing_info"),
			"Processing missing_info"
		);

		// missing_info should be a JSON string, but handle all cases for robustness
		let missing_info: Vec<String> = if let Some(s) = parsed_input["missing_info"].as_str() {
			// Try to parse as JSON array first
			if let Ok(parsed) = serde_json::from_str::<Vec<String>>(s) {
				parsed
			} else {
				// If not valid JSON, treat as single string
				vec![s.to_string()]
			}
		} else if let Some(arr) = parsed_input["missing_info"].as_array() {
			// Fallback: if somehow we get an array directly
			arr.iter()
				.filter_map(|v| v.as_str().map(|s| s.to_string()))
				.collect()
		} else if parsed_input["missing_info"].is_object() {
			// If it's an object, try to find an array field in it
			parsed_input["missing_info"]
				.get("missing_info")
				.and_then(|v| v.as_array())
				.map(|arr| {
					arr.iter()
						.filter_map(|v| v.as_str().map(|s| s.to_string()))
						.collect()
				})
				.or_else(|| {
					// Try other common field names
					parsed_input["missing_info"]
						.get("items")
						.and_then(|v| v.as_array())
						.map(|arr| {
							arr.iter()
								.filter_map(|v| v.as_str().map(|s| s.to_string()))
								.collect()
						})
				})
				.unwrap_or_else(|| {
					vec![
						"destination".to_string(),
						"dates".to_string(),
						"budget".to_string(),
					]
				})
		} else if parsed_input.get("missing_info").is_some() {
			// Some other type - use defaults
			vec![
				"destination".to_string(),
				"dates".to_string(),
				"budget".to_string(),
			]
		} else {
			// If missing_info is not provided at all, use default common missing information
			// This prevents the tool from failing and allows the agent to continue
			info!(target: "orchestrator_tool", tool = "ask_for_clarification", "missing_info not provided, using defaults");
			vec![
				"destination".to_string(),
				"travel dates".to_string(),
				"budget".to_string(),
				"preferences".to_string(),
			]
		};

		// Handle context - can be string (JSON), object, or missing
		let context = parsed_input.get("context").unwrap_or(&Value::Null);
		let context_str = if let Some(s) = context.as_str() {
			// If it's a string, check if it's JSON, otherwise use as-is
			if serde_json::from_str::<Value>(s).is_ok() {
				s.to_string()
			} else {
				s.to_string()
			}
		} else if context.is_object() {
			serde_json::to_string(context).unwrap_or_else(|_| "{}".to_string())
		} else {
			"".to_string()
		};

		info!(target: "orchestrator_tool", tool = "ask_for_clarification", missing_info_count = missing_info.len(), "Asking for clarification");
		debug!(target: "orchestrator_tool", tool = "ask_for_clarification", input = %serde_json::to_string(&parsed_input)?, "Tool input");

		// Retrieve chat context to extract known information
		let chat_id = self.chat_session_id.load(Ordering::Relaxed);
		if chat_id == 0 {
			return Err("chat_session_id not set. This should be set by the controller before invoking the agent.".into());
		}

		// ANTI-LOOP PROTECTION: Check if we've already asked for clarification
		// If asked_clarification flag is already true in trip context, we should NOT ask again
		{
			let store_guard = self.context_store.read().await;
			if let Some(context_data) = store_guard.get(&chat_id) {
				if context_data.trip_context.asked_clarification {
					info!(
						target: "orchestrator_tool",
						tool = "ask_for_clarification",
						chat_id = chat_id,
						"Already asked for clarification before - returning ready signal to prevent loop"
					);
					// Return a signal that tells the agent we're ready to proceed
					return Ok("Ready for research pipeline.".to_string());
				}
			}
		}

		// Get chat history to extract known information
		let messages = sqlx::query!(
			r#"
			SELECT
				m.id,
				m.is_user,
				m.timestamp,
				m.text,
				m.itinerary_id
			FROM messages m
			WHERE m.chat_session_id = $1
			ORDER BY m.timestamp ASC
			LIMIT 50
			"#,
			chat_id
		)
		.fetch_all(&self.pool)
		.await
		.map_err(|e| format!("Database error: {}", e))?;

		// Extract known information from chat history and in-memory context
		let mut known_info: Vec<String> = Vec::new();
		let chat_text = messages
			.iter()
			.filter(|m| m.is_user)
			.map(|m| m.text.as_str())
			.collect::<Vec<&str>>()
			.join(" ");

		// Try to extract from chat history using simple patterns
		if known_info.is_empty() {
			let chat_lower = chat_text.to_lowercase();

			// Check for destination (common country/city names)
			let destinations = vec![
				"brazil",
				"paris",
				"tokyo",
				"london",
				"new york",
				"rome",
				"barcelona",
				"amsterdam",
				"berlin",
				"dubai",
				"singapore",
				"sydney",
				"mumbai",
				"bangkok",
			];
			for dest in destinations {
				if chat_lower.contains(dest) {
					known_info.push(format!(
						"Destination: {}",
						dest.split_whitespace()
							.next()
							.unwrap_or(dest)
							.to_uppercase()
					));
					break;
				}
			}

			// Check for budget (numbers with $ or dollar/buck keywords)
			if chat_lower.contains("$")
				|| chat_lower.contains("dollar")
				|| chat_lower.contains("budget")
				|| chat_lower.contains("buck")
			{
				let words: Vec<&str> = chat_text.split_whitespace().collect();
				for (i, word) in words.iter().enumerate() {
					let word_lower = word.to_lowercase();
					if word_lower.contains("$")
						|| word_lower.contains("dollar")
						|| word_lower.contains("budget")
						|| word_lower.contains("buck")
					{
						// Look for numbers nearby
						for j in i.saturating_sub(2)..(i + 3).min(words.len()) {
							if words[j].chars().any(|c| c.is_numeric()) {
								known_info.push(format!("Budget: {}", words[j]));
								break;
							}
						}
					}
				}
			}

			// Check for dates (months or date patterns)
			let months = vec![
				"january",
				"february",
				"march",
				"april",
				"may",
				"june",
				"july",
				"august",
				"september",
				"october",
				"november",
				"december",
			];
			for month in months {
				if chat_lower.contains(month) {
					known_info.push(format!("Dates: mentioned in conversation"));
					break;
				}
			}
		}

		let known_info_str = if known_info.is_empty() {
			"None yet".to_string()
		} else {
			known_info.join(", ")
		};
		let missing_info_str = missing_info.join(", ");

		let prompt = format!(
			r#"Generate a friendly, natural clarification message for a travel planning conversation.

IMPORTANT: You must show the user what information you already have and what you still need.

Information I Already Have:
{}

Information I Still Need:
{}

Conversation Context: {}

Create a friendly message that:
1. Acknowledges what information you already have (if any)
2. Clearly states what information is still needed to create the itinerary
3. Asks for the missing information in a natural, conversational way

Format your response as a complete message that shows both what you know and what you need.
Example: "Great! I see you're planning a trip to [destination]. To create your itinerary, I still need to know [missing info]. Could you share [specific questions]?"

Return ONLY the message text, nothing else."#,
			known_info_str, missing_info_str, context_str
		);

		let response = self.llm.invoke(&prompt).await?;
		let clarification = response.trim().to_string();

		// Insert the clarification message into the database to stop the pipeline
		let record = sqlx::query!(
			r#"
			INSERT INTO messages (chat_session_id, itinerary_id, is_user, timestamp, text)
			VALUES ($1, NULL, FALSE, NOW(), $2)
			RETURNING id;
			"#,
			chat_id,
			clarification
		)
		.fetch_one(&self.pool)
		.await
		.map_err(|e| format!("Database error: {}", e))?;

		info!(
			target: "orchestrator_tool",
			tool = "ask_for_clarification",
			chat_id = chat_id,
			message_id = record.id,
			"Clarification message sent - pipeline stopped"
		);
		debug!(
			target: "orchestrator_tool",
			tool = "ask_for_clarification",
			clarification = %clarification,
			"Tool output"
		);

		// Mark that we've asked for clarification in the trip context
		{
			let mut store_guard = self.context_store.write().await;
			if let Some(context_data) = store_guard.get_mut(&chat_id) {
				context_data.trip_context.asked_clarification = true;
				info!(
					target: "trip_context",
					tool = "ask_for_clarification",
					chat_id = chat_id,
					"Marked asked_clarification flag in trip context"
				);
			}
		}

		// Return the clarification text directly.
		// The message is already inserted in the database with the ID in record.id
		// The agent prompt instructs to use this as Final Answer immediately.
		let result = clarification.clone();

		let elapsed = start_time.elapsed();
		info!(
			target: "orchestrator_tool",
			tool = "ask_for_clarification",
			elapsed_ms = elapsed.as_millis() as u64,
			"Tool completed - pipeline stopped"
		);

		// Track this tool execution
		track_tool_execution(
			&self.context_store,
			&self.chat_session_id,
			"ask_for_clarification",
			&input_clone,
			&result,
		)
		.await?;

		Ok(result)
	}
}

/// Tool: Respond to User
/// Sends a response to the user with the current itinerary (if available) or asks for more information.
/// This tool STOPS the pipeline and sends the final message to the user.
#[derive(Clone)]
pub struct RespondToUserTool {
	pool: PgPool,
	chat_session_id: Arc<AtomicI32>,
	context_store: SharedContextStore,
}

impl RespondToUserTool {
	pub fn new(
		pool: PgPool,
		chat_session_id: Arc<AtomicI32>,
		context_store: SharedContextStore,
	) -> Self {
		Self {
			pool,
			chat_session_id,
			context_store,
		}
	}
}

#[async_trait]
impl Tool for RespondToUserTool {
	fn name(&self) -> String {
		"respond_to_user".to_string()
	}

	fn description(&self) -> String {
		"STOPS THE PIPELINE and sends a response to the user. If active_itinerary exists in context, creates/updates the itinerary in the database and sends it to the user. If active_itinerary is empty or missing, sends a message asking for more information. This tool inserts a message into the chat and returns a success message. CRITICAL: After calling this tool, you MUST immediately return 'Final Answer' with a confirmation. DO NOT call any other tools. Use this as your final action when ready to respond to the user."
			.to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {
				"message": {
					"type": "string",
					"description": "Optional message to send to the user as a string. If not provided, will generate based on itinerary status."
				}
			},
			"required": []
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let start_time = Instant::now();
		let input_clone = input.clone(); // Clone for tracking

		crate::tool_trace!(agent: "orchestrator", tool: "respond_to_user", status: "start");

		// Update progress to FinalizingItinerary
		let chat_id = self.chat_session_id.load(Ordering::Relaxed);
		if chat_id > 0 {
			_ = sqlx::query!(
				r#"UPDATE chat_sessions
				SET llm_progress=$1
				WHERE id=$2;"#,
				LlmProgress::FinalizingItinerary as _,
				chat_id
			)
			.execute(&self.pool)
			.await;
		}

		debug!(
			target: "orchestrator_tool",
			tool = "respond_to_user",
			input_raw = %serde_json::to_string(&input).unwrap_or_else(|_| "failed to serialize".to_string()),
			"Received input in respond_to_user"
		);

		// Get chat_session_id from shared atomic (set by controller before agent invocation)
		if chat_id == 0 {
			return Err("chat_session_id not set. This should be set by the controller before invoking the agent.".into());
		}

		// langchain_rust passes action_input as a STRING, so we need to parse it first
		let parsed_input: Value = if input.is_string() {
			// If input is a string (JSON string from action_input), parse it
			serde_json::from_str(input.as_str().unwrap_or("{}")).unwrap_or_else(|_| json!({}))
		} else {
			// If it's already a Value object, use it directly
			input
		};

		// Handle message as either string or object (convert object to string)
		let optional_message = parsed_input.get("message").map(|m| {
			if m.is_string() {
				m.as_str().unwrap_or("").to_string()
			} else if m.is_object() {
				serde_json::to_string(m).unwrap_or_else(|_| "{}".to_string())
			} else {
				m.to_string()
			}
		});

		info!(target: "orchestrator_tool", tool = "respond_to_user", chat_id = chat_id, "Responding to user - pipeline stopped");
		debug!(target: "orchestrator_tool", tool = "respond_to_user", input = %serde_json::to_string(&parsed_input)?, "Tool input");

		// Get context to check for active_itinerary
		let store_guard = self.context_store.read().await;
		let context_data = store_guard
			.get(&chat_id)
			.cloned()
			.unwrap_or_else(|| ContextData {
				chat_session_id: chat_id,
				user_id: 0,
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
			});

		// Check if we have an active itinerary
		let has_itinerary = context_data.active_itinerary.is_some()
			&& context_data
				.active_itinerary
				.as_ref()
				.map(|it| {
					// Check if itinerary is not empty (has some structure)
					!it.is_null() && (!it.is_object() || !it.as_object().unwrap().is_empty())
				})
				.unwrap_or(false);

		let (message_text, message_id) = if has_itinerary {
			// Parse and save the itinerary to database
			let itinerary_json = context_data.active_itinerary.clone().unwrap();

			// Get user_id from chat_session
			let user_id = sqlx::query!(
				r#"
			SELECT cs.account_id
			FROM chat_sessions cs
			WHERE cs.id = $1
			"#,
				chat_id
			)
			.fetch_one(&self.pool)
			.await
			.map_err(|e| format!("Failed to get user_id from chat_session: {}", e))?
			.account_id;

			// Extract event IDs from the LLM-generated itinerary
			let mut all_event_ids = Vec::new();
			if let Some(event_days) = itinerary_json.get("event_days").and_then(|v| v.as_array()) {
				for day in event_days {
					for time_block in &["morning_events", "afternoon_events", "evening_events"] {
						if let Some(events) = day.get(time_block).and_then(|v| v.as_array()) {
							for event in events {
								if let Some(id) = event.get("id").and_then(|v| v.as_i64()) {
									all_event_ids.push(id as i32);
								}
							}
						}
					}
				}
			}
			if let Some(unassigned) = itinerary_json
				.get("unassigned_events")
				.and_then(|v| v.as_array())
			{
				for event in unassigned {
					if let Some(id) = event.get("id").and_then(|v| v.as_i64()) {
						all_event_ids.push(id as i32);
					}
				}
			}

			// Fetch full event objects from database
			use crate::http_models::event::Event as HttpEvent;
			let full_events = if !all_event_ids.is_empty() {
				sqlx::query_as!(
					HttpEvent,
					r#"
				SELECT 
					id, event_name, event_description, street_address, city, country, postal_code,
					lat, lng, event_type, user_created, hard_start, hard_end, timezone, place_id,
					wheelchair_accessible_parking, wheelchair_accessible_entrance,
					wheelchair_accessible_restroom, wheelchair_accessible_seating,
					serves_vegetarian_food, price_level, utc_offset_minutes, website_uri, types,
					photo_name, photo_width, photo_height, photo_author, photo_author_uri,
					photo_author_photo_uri, weekday_descriptions, secondary_hours_type,
					next_open_time, next_close_time, open_now,
					periods as "periods!: Vec<crate::sql_models::Period>",
					special_days,
					NULL::int4 as block_index
				FROM events
				WHERE id = ANY($1)
				"#,
					&all_event_ids
				)
				.fetch_all(&self.pool)
				.await
				.map_err(|e| format!("Failed to fetch full events: {}", e))?
			} else {
				Vec::new()
			};

			// Create a map of event ID -> full event for quick lookup
			let event_map: std::collections::HashMap<i32, HttpEvent> =
				full_events.into_iter().map(|e| (e.id, e)).collect();

			// Helper function to hydrate events with full data from database
			let hydrate_events = |partial_events: &Value| -> Vec<HttpEvent> {
				if let Some(events_arr) = partial_events.as_array() {
					events_arr
						.iter()
						.filter_map(|e| {
							e.get("id")
								.and_then(|v| v.as_i64())
								.and_then(|id| event_map.get(&(id as i32)).cloned())
						})
						.collect()
				} else {
					Vec::new()
				}
			};

			// Parse itinerary structure (dates, title, days)
			use crate::http_models::itinerary::EventDay as HttpEventDay;
			let mut event_days = Vec::new();
			if let Some(days) = itinerary_json.get("event_days").and_then(|v| v.as_array()) {
				for day in days {
					let date_str = day
						.get("date")
						.and_then(|v| v.as_str())
						.unwrap_or("2025-01-01");
					let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
						.unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());

					event_days.push(HttpEventDay {
						morning_events: hydrate_events(
							&day.get("morning_events").cloned().unwrap_or(json!([])),
						),
						afternoon_events: hydrate_events(
							&day.get("afternoon_events").cloned().unwrap_or(json!([])),
						),
						evening_events: hydrate_events(
							&day.get("evening_events").cloned().unwrap_or(json!([])),
						),
						date,
					});
				}
			}

			let unassigned_events = hydrate_events(
				&itinerary_json
					.get("unassigned_events")
					.cloned()
					.unwrap_or(json!([])),
			);

			// Create HttpItinerary with hydrated events
			let start_date_str = itinerary_json
				.get("start_date")
				.and_then(|v| v.as_str())
				.unwrap_or("2025-01-01");
			let end_date_str = itinerary_json
				.get("end_date")
				.and_then(|v| v.as_str())
				.unwrap_or("2025-01-01");
			let start_date = chrono::NaiveDate::parse_from_str(start_date_str, "%Y-%m-%d")
				.unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
			let end_date = chrono::NaiveDate::parse_from_str(end_date_str, "%Y-%m-%d")
				.unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
			let title = itinerary_json
				.get("title")
				.and_then(|v| v.as_str())
				.unwrap_or("Trip Itinerary")
				.to_string();

			let mut itinerary = HttpItinerary {
				id: 0, // Temporary, will be set after insert
				start_date,
				end_date,
				event_days,
				chat_session_id: Some(chat_id),
				title,
				unassigned_events,
			};

			// Extract unassigned event IDs
			let unassigned_event_ids: Vec<i32> =
				itinerary.unassigned_events.iter().map(|e| e.id).collect();

			// Insert itinerary into database
			let itinerary_id = sqlx::query!(
			r#"
			INSERT INTO itineraries (account_id, is_public, start_date, end_date, chat_session_id, saved, title, unassigned_event_ids)
			VALUES ($1, FALSE, $2, $3, $4, FALSE, $5, $6)
			RETURNING id;
			"#,
			user_id,
			itinerary.start_date,
			itinerary.end_date,
			chat_id,
			itinerary.title,
			&unassigned_event_ids
		)
		.fetch_one(&self.pool)
		.await
		.map_err(|e| format!("Failed to insert itinerary: {}", e))?
		.id;

			info!(
				target: "orchestrator_tool",
				tool = "respond_to_user",
				chat_id = chat_id,
				itinerary_id = itinerary_id,
				"Created itinerary in database"
			);

			// Update itinerary ID for insert_event_list
			itinerary.id = itinerary_id;

			// Capture the number of days before moving itinerary
			let num_days = itinerary.event_days.len();

			// Insert all events into event_list table
			insert_event_list(itinerary, &self.pool)
				.await
				.map_err(|e| format!("Failed to insert event list: {}", e))?;

			info!(
				target: "orchestrator_tool",
				tool = "respond_to_user",
				itinerary_id = itinerary_id,
				"Inserted event list for itinerary"
			);

			// Create user-friendly message
			let default_message = format!(
				"I've created your travel itinerary! It includes {} days with events scheduled throughout. You can view and edit it in your saved itineraries.",
				num_days
			);
			let message = optional_message
				.map(|s| s.to_string())
				.unwrap_or(default_message);

			// Insert message with itinerary_id
			let record = sqlx::query!(
				r#"
			INSERT INTO messages (chat_session_id, itinerary_id, is_user, timestamp, text)
			VALUES ($1, $2, FALSE, NOW(), $3)
			RETURNING id;
			"#,
				chat_id,
				itinerary_id,
				message
			)
			.fetch_one(&self.pool)
			.await
			.map_err(|e| format!("Database error: {}", e))?;

			info!(
				target: "orchestrator_tool",
				tool = "respond_to_user",
				chat_id = chat_id,
				message_id = record.id,
				itinerary_id = itinerary_id,
				"Sent itinerary to user"
			);

			(message, record.id)
		} else {
			// No itinerary - ask for more information
			let default_message = "I need more information to create your itinerary. Could you please provide:\n- Your travel destination\n- Travel dates (start and end)\n- Budget\n- Any preferences or constraints you have?";
			let message = optional_message.unwrap_or(default_message.to_string());

			// Insert message asking for more info
			let record = sqlx::query!(
				r#"
				INSERT INTO messages (chat_session_id, itinerary_id, is_user, timestamp, text)
				VALUES ($1, NULL, FALSE, NOW(), $2)
				RETURNING id;
				"#,
				chat_id,
				message
			)
			.fetch_one(&self.pool)
			.await
			.map_err(|e| format!("Database error: {}", e))?;

			info!(
				target: "orchestrator_tool",
				tool = "respond_to_user",
				chat_id = chat_id,
				message_id = record.id,
				"Asked user for more information"
			);

			(message, record.id)
		};

		// Mark pipeline as Ready now that we've sent the final response to the user.
		// This ensures the frontend progress indicator leaves the "FinalizingItinerary"
		// state as soon as the itinerary/message are fully persisted.
		{
			let pool = self.pool.clone();
			let chat_id_copy = chat_id;
			tokio::spawn(async move {
				if chat_id_copy > 0 {
					if let Err(e) = sqlx::query!(
						r#"
						UPDATE chat_sessions
						SET llm_progress = $1
						WHERE id = $2;
						"#,
						crate::sql_models::LlmProgress::Ready as _,
						chat_id_copy,
					)
					.execute(&pool)
					.await
					{
						tracing::error!(
							target: "orchestrator_pipeline",
							chat_session_id = chat_id_copy,
							error = %e,
							"Failed to reset llm_progress to Ready after respond_to_user"
						);
					}
				}
			});
		}

		// Return a special marker that send_message_to_llm can detect
		// Format: "MESSAGE_INSERTED:<message_id>:<message_text>"
		let result = format!("MESSAGE_INSERTED:{}:{}", message_id, message_text);

		let elapsed = start_time.elapsed();
		info!(
			target: "orchestrator_tool",
			tool = "respond_to_user",
			elapsed_ms = elapsed.as_millis() as u64,
			"Tool completed"
		);

		// Track this tool execution
		track_tool_execution(
			&self.context_store,
			&self.chat_session_id,
			"respond_to_user",
			&input_clone,
			&result,
		)
		.await?;

		Ok(result)
	}
}

/// Tool: Update Trip Context
/// Updates the trip context with new information from the user's latest message.
/// This tool should be called AFTER retrieve_chat_context to incrementally fill in trip details.
#[derive(Clone)]
pub struct UpdateTripContextTool {
	llm: Arc<dyn LLM + Send + Sync>,
	chat_session_id: Arc<AtomicI32>,
	context_store: SharedContextStore,
}

impl UpdateTripContextTool {
	pub fn new(
		llm: Arc<dyn LLM + Send + Sync>,
		chat_session_id: Arc<AtomicI32>,
		context_store: SharedContextStore,
	) -> Self {
		Self {
			llm,
			chat_session_id,
			context_store,
		}
	}
}

#[async_trait]
impl Tool for UpdateTripContextTool {
	fn name(&self) -> String {
		"update_trip_context".to_string()
	}

	fn description(&self) -> String {
		"Updates the trip context with new information extracted from the user's latest message in the chat history. Call this AFTER retrieve_chat_context to incrementally fill in destination, dates, budget, preferences. Automatically extracts the most recent user message from chat_history. Only updates fields that are present in the new information - existing fields are preserved. Returns the updated trip context showing what information we now have and what is still missing."
			.to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {},
			"required": []
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let start_time = Instant::now();
		let input_clone = input.clone();

		crate::tool_trace!(agent: "task", tool: "update_trip_context", status: "start");

		let chat_id = self.chat_session_id.load(Ordering::Relaxed);
		if chat_id == 0 {
			return Err("chat_session_id not set".into());
		}

		info!(
			target: "orchestrator_tool",
			tool = "update_trip_context",
			chat_id = chat_id,
			"Updating trip context from chat history"
		);

		// Get current trip context AND extract the last 5 user messages from chat_history
		// We need multiple messages because user provides info across multiple turns
		let (current_context, user_messages) = {
			let store_guard = self.context_store.read().await;
			let context_data = store_guard
				.get(&chat_id)
				.ok_or("Context not found for chat_id")?;

			// Extract the last 5 user messages from chat_history (most recent first)
			let recent_user_msgs: Vec<String> = context_data
				.chat_history
				.iter()
				.rev() // Start from the end (most recent)
				.filter(|msg| msg.get("role").and_then(|r| r.as_str()) == Some("user"))
				.take(5) // Get last 5 user messages
				.filter_map(|msg| {
					msg.get("content")
						.and_then(|c| c.as_str())
						.map(|s| s.to_string())
				})
				.collect();

			// Combine them into one string (most recent first)
			let combined_messages = recent_user_msgs.join("\n");

			info!(
				target: "trip_context",
				tool = "update_trip_context",
				chat_id = chat_id,
				message_count = recent_user_msgs.len(),
				combined_length = combined_messages.len(),
				"Extracted recent user messages from chat history"
			);
			debug!(
				target: "trip_context",
				messages = %combined_messages,
				"Combined user messages for extraction"
			);

			(context_data.trip_context.clone(), combined_messages)
		};

		info!(
			target: "trip_context",
			tool = "update_trip_context",
			chat_id = chat_id,
			"BEFORE UPDATE - Current trip context",
		);
		debug!(
			target: "trip_context",
			current_destination = ?current_context.destination,
			current_start_date = ?current_context.start_date,
			current_end_date = ?current_context.end_date,
			current_budget = ?current_context.budget,
			current_preferences = ?current_context.preferences,
			current_constraints = ?current_context.constraints,
			"Current trip context details"
		);

		// Use LLM to extract trip information from the messages
		let extraction_prompt = format!(
			r#"Extract trip planning information from these recent user messages. Return ONLY a JSON object.

Current context (preserve these if not mentioned in new messages):
- destination: {}
- start_date: {}
- end_date: {}
- budget: {}
- preferences: {}

Recent user messages (newest first):
"{}"

IMPORTANT: Extract information from ALL the messages above, not just the first one.

Return JSON with the information found across all messages:
{{
  "destination": "string or null",
  "start_date": "YYYY-MM-DD or null",
  "end_date": "YYYY-MM-DD or null",
  "budget": number or null,
  "preferences": ["array", "of", "strings"] or [],
  "action": "create|modify|view|delete or null"
}}

Examples:
- "Brazil" + "10/8 to 10/20" → {{"destination": "Brazil", "start_date": "2023-10-08", "end_date": "2023-10-20"}}
- "no preferences" → {{"preferences": []}}

Return valid JSON only."#,
			current_context.destination.as_deref().unwrap_or("null"),
			current_context.start_date.as_deref().unwrap_or("null"),
			current_context.end_date.as_deref().unwrap_or("null"),
			current_context
				.budget
				.map(|b| b.to_string())
				.as_deref()
				.unwrap_or("null"),
			serde_json::to_string(&current_context.preferences)
				.unwrap_or_else(|_| "[]".to_string()),
			user_messages
		);

		let llm_response = self
			.llm
			.invoke(&extraction_prompt)
			.await
			.map_err(|e| format!("LLM error: {}", e))?;

		info!(
			target: "trip_context",
			tool = "update_trip_context",
			chat_id = chat_id,
			"LLM extraction response",
		);
		debug!(
			target: "trip_context",
			llm_response = %llm_response,
			"Raw LLM response for extraction"
		);

		// Parse LLM response
		let extracted: Value = serde_json::from_str(&llm_response).unwrap_or_else(|e| {
			info!(
				target: "trip_context",
				error = %e,
				raw_response = %llm_response,
				"Failed to parse LLM response as JSON, using empty object"
			);
			json!({})
		});

		// Merge with current context (only update non-null fields)
		let mut updated_context = current_context;

		if let Some(dest) = extracted["destination"].as_str() {
			updated_context.destination = Some(dest.to_string());
		}
		if let Some(start) = extracted["start_date"].as_str() {
			updated_context.start_date = Some(start.to_string());
		}
		if let Some(end) = extracted["end_date"].as_str() {
			updated_context.end_date = Some(end.to_string());
		}
		if let Some(budget) = extracted["budget"].as_f64() {
			updated_context.budget = Some(budget);
		}
		if let Some(prefs) = extracted["preferences"].as_array() {
			let new_prefs: Vec<String> = prefs
				.iter()
				.filter_map(|v| v.as_str().map(|s| s.to_string()))
				.collect();
			if !new_prefs.is_empty() {
				updated_context.preferences.extend(new_prefs);
				updated_context.preferences.dedup();
			}
		}
		if let Some(action) = extracted["action"].as_str() {
			updated_context.action = Some(action.to_string());
		}

		// Save updated context
		{
			let mut store_guard = self.context_store.write().await;
			if let Some(context_data) = store_guard.get_mut(&chat_id) {
				context_data.trip_context = updated_context.clone();

				info!(
					target: "trip_context",
					tool = "update_trip_context",
					chat_id = chat_id,
					"AFTER UPDATE - Updated trip context saved",
				);
				debug!(
					target: "trip_context",
					updated_destination = ?updated_context.destination,
					updated_start_date = ?updated_context.start_date,
					updated_end_date = ?updated_context.end_date,
					updated_budget = ?updated_context.budget,
					updated_preferences = ?updated_context.preferences,
					updated_constraints = ?updated_context.constraints,
					"Updated trip context details"
				);
			}
		}

		// Determine what's still missing - ONLY require destination and dates
		// Budget, preferences, and constraints are ALL optional
		let mut missing = Vec::new();
		if updated_context.destination.is_none() {
			missing.push("destination");
		}
		if updated_context.start_date.is_none() {
			missing.push("start_date");
		}
		if updated_context.end_date.is_none() {
			missing.push("end_date");
		}
		// Budget, preferences, and constraints are optional - don't add to missing

		// Check if we've asked clarification at least once
		let has_asked_before = updated_context.asked_clarification;

		// Ready for pipeline only if:
		// 1. No missing required fields AND
		// 2. We've asked clarification at least once
		let ready_for_pipeline = missing.is_empty() && has_asked_before;

		let result = json!({
			"trip_context": updated_context,
			"missing_info": missing,
			"ready_for_pipeline": ready_for_pipeline,
			"asked_clarification_before": has_asked_before
		});

		let result_str = serde_json::to_string(&result)?;

		let elapsed = start_time.elapsed();
		info!(
			target: "orchestrator_tool",
			tool = "update_trip_context",
			elapsed_ms = elapsed.as_millis() as u64,
			chat_id = chat_id,
			missing_count = missing.len(),
			ready = missing.is_empty(),
			"Trip context update complete - SUMMARY",
		);
		debug!(
			target: "trip_context",
			missing_fields = ?missing,
			"Missing information details"
		);

		track_tool_execution(
			&self.context_store,
			&self.chat_session_id,
			"update_trip_context",
			&input_clone,
			&result_str,
		)
		.await?;

		Ok(result_str)
	}
}

/// Tool 6: Update Chat Title
/// Automatically updates the chat session title based on trip context
/// Only updates if the title is still "New Chat" (default)
#[derive(Clone)]
pub struct UpdateChatTitleTool {
	pool: PgPool,
	chat_session_id: Arc<AtomicI32>,
	context_store: SharedContextStore,
}

impl UpdateChatTitleTool {
	pub fn new(
		pool: PgPool,
		chat_session_id: Arc<AtomicI32>,
		context_store: SharedContextStore,
	) -> Self {
		Self {
			pool,
			chat_session_id,
			context_store,
		}
	}
}

#[async_trait]
impl Tool for UpdateChatTitleTool {
	fn name(&self) -> String {
		"update_chat_title".to_string()
	}

	fn description(&self) -> String {
		"Automatically updates the chat session title with destination and dates when trip context is available. Only updates if title is still 'New Chat'. Call this after update_trip_context when you have destination and dates. No parameters needed."
			.to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {},
			"required": []
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let start_time = Instant::now();
		let input_clone = input.clone();

		crate::tool_trace!(agent: "task", tool: "update_chat_title", status: "start");

		let chat_id = self.chat_session_id.load(Ordering::Relaxed);
		if chat_id == 0 {
			return Err("chat_session_id not set".into());
		}

		// Get trip context
		let trip_context = {
			let store_guard = self.context_store.read().await;
			store_guard
				.get(&chat_id)
				.map(|ctx| ctx.trip_context.clone())
				.ok_or("Context not found")?
		};

		// Check if we have enough info to make a title
		if trip_context.destination.is_none() {
			return Ok(json!({
				"updated": false,
				"reason": "No destination set yet"
			})
			.to_string());
		}

		// Check current title - only update if it's "New Chat"
		let current_title =
			sqlx::query!(r#"SELECT title FROM chat_sessions WHERE id = $1"#, chat_id)
				.fetch_one(&self.pool)
				.await
				.map_err(|e| format!("Database error: {}", e))?
				.title;

		if current_title != "New Chat" {
			info!(
				target: "orchestrator_tool",
				tool = "update_chat_title",
				chat_id = chat_id,
				current_title = %current_title,
				"Title already set, skipping update"
			);
			return Ok(json!({
				"updated": false,
				"reason": "Title already set",
				"current_title": current_title
			})
			.to_string());
		}

		// Build new title from trip context
		let mut title_parts = Vec::new();

		if let Some(dest) = &trip_context.destination {
			title_parts.push(dest.clone());
		}

		// Format dates if we have both
		if let (Some(start), Some(end)) = (&trip_context.start_date, &trip_context.end_date) {
			// Try to format as "MMM DD-DD" if same month
			if let (Ok(start_date), Ok(end_date)) = (
				chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d"),
				chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d"),
			) {
				if start_date.month() == end_date.month() {
					title_parts.push(format!(
						"{} {}-{}",
						start_date.format("%b"),
						start_date.day(),
						end_date.day()
					));
				} else {
					title_parts.push(format!(
						"{} - {}",
						start_date.format("%b %d"),
						end_date.format("%b %d")
					));
				}
			}
		}

		let new_title = if title_parts.is_empty() {
			"New Trip".to_string()
		} else {
			title_parts.join(", ")
		};

		// Update the title
		sqlx::query!(
			r#"UPDATE chat_sessions SET title = $1 WHERE id = $2"#,
			new_title,
			chat_id
		)
		.execute(&self.pool)
		.await
		.map_err(|e| format!("Database error: {}", e))?;

		info!(
			target: "orchestrator_tool",
			tool = "update_chat_title",
			chat_id = chat_id,
			new_title = %new_title,
			"Updated chat session title"
		);

		let result = json!({
			"updated": true,
			"new_title": new_title
		});

		let elapsed = start_time.elapsed();
		info!(
			target: "orchestrator_tool",
			tool = "update_chat_title",
			elapsed_ms = elapsed.as_millis() as u64,
			"Tool completed"
		);

		track_tool_execution(
			&self.context_store,
			&self.chat_session_id,
			"update_chat_title",
			&input_clone,
			&result.to_string(),
		)
		.await?;

		Ok(result.to_string())
	}
}

/// Gets the tools used by the Task Agent to build planning context.
/// These tools are focused on:
/// - retrieving user profile
/// - retrieving chat history/context
/// - updating trip context incrementally
/// - asking for clarification when information is missing
pub fn get_task_tools(
	llm: Arc<dyn LLM + Send + Sync>,
	pool: PgPool,
	chat_session_id: Arc<AtomicI32>,
	user_id: Arc<AtomicI32>,
	context_store: SharedContextStore,
) -> Vec<Arc<dyn Tool>> {
	vec![
		Arc::new(ParseUserIntentTool::new(
			Arc::clone(&llm),
			Arc::clone(&chat_session_id),
			context_store.clone(),
		)),
		Arc::new(RetrieveChatContextTool::new(
			pool.clone(),
			Arc::clone(&chat_session_id),
			context_store.clone(),
		)),
		Arc::new(RetrieveUserProfileTool::new(
			pool.clone(),
			Arc::clone(&chat_session_id),
			Arc::clone(&user_id),
			context_store.clone(),
		)),
		Arc::new(UpdateTripContextTool::new(
			Arc::clone(&llm),
			Arc::clone(&chat_session_id),
			context_store.clone(),
		)),
		Arc::new(UpdateChatTitleTool::new(
			pool.clone(),
			Arc::clone(&chat_session_id),
			context_store.clone(),
		)),
		Arc::new(AskForClarificationTool::new(
			Arc::clone(&llm),
			pool.clone(),
			Arc::clone(&chat_session_id),
			context_store.clone(),
		)),
	]
}
