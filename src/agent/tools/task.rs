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

use crate::agent::models::context::{ContextData, ToolExecution};
use crate::agent::models::user::UserIntent;
use crate::agent::tools::orchestrator::track_tool_execution;
use crate::http_models::event::Event;
use async_trait::async_trait;
use langchain_rust::language_models::llm::LLM;
use langchain_rust::tools::Tool;
use serde_json::{Value, json};
use sqlx::PgPool;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use tracing::{debug, info};

/// Tool 1: Parse User Intent
/// Parses user input to extract intent, destination, dates, budget, and constraints.
/// Returns a UserIntent object.
#[derive(Clone)]
pub struct ParseUserIntentTool {
	llm: Arc<dyn LLM + Send + Sync>,
	pool: PgPool,
	chat_session_id: Arc<AtomicI32>,
}

impl ParseUserIntentTool {
	pub fn new(llm: Arc<dyn LLM + Send + Sync>, pool: PgPool, chat_session_id: Arc<AtomicI32>) -> Self {
		Self { 
			llm,
			pool,
			chat_session_id,
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
		let input_clone = input.clone(); // Clone for tracking
		
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
		
		// Track this tool execution
		track_tool_execution(&self.pool, &self.chat_session_id, "parse_user_intent", &input_clone, &result).await?;
		
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
}

impl RetrieveChatContextTool {
	pub fn new(pool: PgPool, chat_session_id: Arc<AtomicI32>) -> Self {
		Self { pool, chat_session_id }
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
		let input_clone = _input.clone(); // Clone for tracking
		
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

		// Retrieve context from database (includes pipeline state and events)
		let context_row = sqlx::query!(
			r#"SELECT context as "context: serde_json::Value" FROM chat_sessions WHERE id = $1"#,
			chat_id
		)
		.fetch_optional(&self.pool)
		.await
		.map_err(|e| format!("Database error: {}", e))?;

		let mut context_data = if let Some(row) = context_row {
			if let Some(ctx) = row.context {
				// Try to parse as ContextData, fallback to building from parts
				serde_json::from_value::<ContextData>(ctx.clone())
					.unwrap_or_else(|_| {
						// Build ContextData from existing context structure
						ContextData {
							user_profile: ctx.get("user_profile").cloned(),
							chat_history: vec![],
							active_itinerary: ctx.get("active_itinerary").cloned(),
							events: ctx.get("events")
								.and_then(|e| e.as_array())
								.map(|arr| arr.iter().filter_map(|v| serde_json::from_value::<Event>(v.clone()).ok()).collect())
								.unwrap_or_default(),
							tool_history: ctx.get("tool_history")
								.and_then(|th| serde_json::from_value::<Vec<ToolExecution>>(th.clone()).ok())
								.unwrap_or_default(),
							pipeline_stage: ctx.get("pipeline_stage").and_then(|s| s.as_str()).map(|s| s.to_string()),
							researched_events: ctx.get("researched_events")
								.and_then(|e| e.as_array())
								.map(|arr| arr.iter().filter_map(|v| serde_json::from_value::<Event>(v.clone()).ok()).collect())
								.unwrap_or_default(),
							constrained_events: ctx.get("constrained_events")
								.and_then(|e| e.as_array())
								.map(|arr| arr.iter().filter_map(|v| serde_json::from_value::<Event>(v.clone()).ok()).collect())
								.unwrap_or_default(),
							optimized_events: ctx.get("optimized_events")
								.and_then(|e| e.as_array())
								.map(|arr| arr.iter().filter_map(|v| serde_json::from_value::<Event>(v.clone()).ok()).collect())
								.unwrap_or_default(),
							constraints: ctx.get("constraints")
								.and_then(|c| c.as_array())
								.map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
								.unwrap_or_default(),
						}
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

		// Update chat_history with the messages we just retrieved
		context_data.chat_history = chat_history;

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
		debug!(
			target: "orchestrator_tool",
			tool = "retrieve_chat_context",
			context = %serde_json::to_string(&context_data)?,
			"Full context data"
		);

		// Return full context including pipeline state
		let result = serde_json::to_string(&context_data)?;
		
		// Track this tool execution
		track_tool_execution(&self.pool, &self.chat_session_id, "retrieve_chat_context", &input_clone, &result).await?;
		
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
	user_id: Arc<AtomicI32>,
}

impl RetrieveUserProfileTool {
	pub fn new(pool: PgPool, chat_session_id: Arc<AtomicI32>, user_id: Arc<AtomicI32>) -> Self {
		Self { pool, chat_session_id, user_id }
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
		let input_clone = input.clone(); // Clone for tracking
		
		debug!(
			target: "orchestrator_tool",
			tool = "retrieve_user_profile",
			input_raw = %serde_json::to_string(&input).unwrap_or_else(|_| "failed to serialize".to_string()),
			"Received input in retrieve_user_profile"
		);
		
		// Get user_id from shared atomic (set by controller before agent invocation)
		let user_id = self.user_id.load(Ordering::Relaxed);
		if user_id == 0 {
			return Err("user_id not set. This should be set by the controller before invoking the agent.".into());
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

		// Automatically save user profile to context
		let chat_id = self.chat_session_id.load(Ordering::Relaxed);
		if chat_id != 0 {
			// Get existing context
			let context_row = sqlx::query!(
				r#"SELECT context as "context: serde_json::Value" FROM chat_sessions WHERE id = $1"#,
				chat_id
			)
			.fetch_optional(&self.pool)
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

			// Update user_profile in context
			context_data.user_profile = Some(profile.clone());
			
			// Save updated context to database
			let context_json = serde_json::to_value(&context_data)?;
			sqlx::query!(
				r#"UPDATE chat_sessions SET context = $1::jsonb WHERE id = $2"#,
				context_json as serde_json::Value,
				chat_id
			)
			.execute(&self.pool)
			.await
			.map_err(|e| format!("Database error: {}", e))?;
			
			info!(
				target: "orchestrator_tool",
				tool = "retrieve_user_profile",
				chat_id = chat_id,
				"Saved user profile to context"
			);
		}

		let result = serde_json::to_string(&profile)?;
		
		// Track this tool execution
		track_tool_execution(&self.pool, &self.chat_session_id, "retrieve_user_profile", &input_clone, &result).await?;
		
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
}

impl AskForClarificationTool {
	pub fn new(llm: Arc<dyn LLM + Send + Sync>, pool: PgPool, chat_session_id: Arc<AtomicI32>) -> Self {
		Self { 
			llm,
			pool,
			chat_session_id,
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
		let input_clone = input.clone(); // Clone for tracking
		
		debug!(
			target: "orchestrator_tool",
			tool = "ask_for_clarification",
			input_raw = %serde_json::to_string(&input).unwrap_or_else(|_| "failed to serialize".to_string()),
			"Received input in ask_for_clarification"
		);
		
		// langchain_rust passes action_input as a STRING, so we need to parse it first
		let parsed_input: Value = if input.is_string() {
			// If input is a string (JSON string from action_input), parse it
			serde_json::from_str(input.as_str().unwrap_or("{}"))
				.unwrap_or_else(|_| json!({}))
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
			parsed_input["missing_info"].get("missing_info")
				.and_then(|v| v.as_array())
				.map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
				.or_else(|| {
					// Try other common field names
					parsed_input["missing_info"].get("items")
						.and_then(|v| v.as_array())
						.map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
				})
				.unwrap_or_else(|| vec!["destination".to_string(), "dates".to_string(), "budget".to_string()])
		} else if parsed_input.get("missing_info").is_some() {
			// Some other type - use defaults
			vec!["destination".to_string(), "dates".to_string(), "budget".to_string()]
		} else {
			// If missing_info is not provided at all, use default common missing information
			// This prevents the tool from failing and allows the agent to continue
			info!(target: "orchestrator_tool", tool = "ask_for_clarification", "missing_info not provided, using defaults");
			vec!["destination".to_string(), "travel dates".to_string(), "budget".to_string(), "preferences".to_string()]
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

		// Get context to check for parsed intent information
		let context_row = sqlx::query!(
			r#"SELECT context as "context: serde_json::Value" FROM chat_sessions WHERE id = $1"#,
			chat_id
		)
		.fetch_optional(&self.pool)
		.await
		.map_err(|e| format!("Database error: {}", e))?;

		// Extract known information from chat history and context
		let mut known_info: Vec<String> = Vec::new();
		let chat_text = messages.iter()
			.filter(|m| m.is_user)
			.map(|m| m.text.as_str())
			.collect::<Vec<&str>>()
			.join(" ");

		// First, try to get parsed intent from context (most reliable)
		if let Some(row) = &context_row {
			if let Some(ctx) = &row.context {
				// Check for parsed_intent stored in context
				if let Some(intent_str) = ctx.get("parsed_intent").and_then(|i| i.as_str()) {
					if let Ok(intent) = serde_json::from_str::<UserIntent>(intent_str) {
						if let Some(dest) = &intent.destination {
							known_info.push(format!("Destination: {}", dest));
						}
						if let Some(budget) = intent.budget {
							known_info.push(format!("Budget: ${}", budget));
						}
						if intent.start_date.is_some() || intent.end_date.is_some() {
							let dates = if let (Some(start), Some(end)) = (&intent.start_date, &intent.end_date) {
								format!("{} to {}", start, end)
							} else if let Some(start) = &intent.start_date {
								format!("Starting {}", start)
							} else if let Some(end) = &intent.end_date {
								format!("Ending {}", end)
							} else {
								"Dates mentioned".to_string()
							};
							known_info.push(format!("Dates: {}", dates));
						}
						if !intent.preferences.is_empty() {
							known_info.push(format!("Preferences: {}", intent.preferences.join(", ")));
						}
					}
				}
				// Also check for direct fields in context
				if let Some(dest) = ctx.get("destination").and_then(|d| d.as_str()) {
					if !known_info.iter().any(|i| i.starts_with("Destination:")) {
						known_info.push(format!("Destination: {}", dest));
					}
				}
			}
		}

		// If no parsed intent, try to extract from chat history using simple patterns
		if known_info.is_empty() {
			let chat_lower = chat_text.to_lowercase();
			
			// Check for destination (common country/city names)
			let destinations = vec!["brazil", "paris", "tokyo", "london", "new york", "rome", "barcelona", "amsterdam", "berlin", "dubai", "singapore", "sydney", "mumbai", "bangkok"];
			for dest in destinations {
				if chat_lower.contains(dest) {
					known_info.push(format!("Destination: {}", dest.split_whitespace().next().unwrap_or(dest).to_uppercase()));
					break;
				}
			}
			
			// Check for budget (numbers with $ or dollar/buck keywords)
			if chat_lower.contains("$") || chat_lower.contains("dollar") || chat_lower.contains("budget") || chat_lower.contains("buck") {
				let words: Vec<&str> = chat_text.split_whitespace().collect();
				for (i, word) in words.iter().enumerate() {
					let word_lower = word.to_lowercase();
					if word_lower.contains("$") || word_lower.contains("dollar") || word_lower.contains("budget") || word_lower.contains("buck") {
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
			let months = vec!["january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november", "december"];
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
			known_info_str,
			missing_info_str,
			context_str
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

		// Return a format that makes it absolutely clear this is the final answer
		// The message is already inserted in the database with the ID in record.id
		// Return format that forces agent to stop and use as Final Answer
		let result = format!("FINAL_ANSWER: {}", clarification);
		
		// Track this tool execution
		track_tool_execution(&self.pool, &self.chat_session_id, "ask_for_clarification", &input_clone, &result).await?;
		
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
}

impl RespondToUserTool {
	pub fn new(pool: PgPool, chat_session_id: Arc<AtomicI32>) -> Self {
		Self { pool, chat_session_id }
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
		let input_clone = input.clone(); // Clone for tracking
		
		debug!(
			target: "orchestrator_tool",
			tool = "respond_to_user",
			input_raw = %serde_json::to_string(&input).unwrap_or_else(|_| "failed to serialize".to_string()),
			"Received input in respond_to_user"
		);
		
		// Get chat_session_id from shared atomic (set by controller before agent invocation)
		let chat_id = self.chat_session_id.load(Ordering::Relaxed);
		if chat_id == 0 {
			return Err("chat_session_id not set. This should be set by the controller before invoking the agent.".into());
		}
		
		// langchain_rust passes action_input as a STRING, so we need to parse it first
		let parsed_input: Value = if input.is_string() {
			// If input is a string (JSON string from action_input), parse it
			serde_json::from_str(input.as_str().unwrap_or("{}"))
				.unwrap_or_else(|_| json!({}))
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
		let context_row = sqlx::query!(
			r#"SELECT context as "context: serde_json::Value" FROM chat_sessions WHERE id = $1"#,
			chat_id
		)
		.fetch_optional(&self.pool)
		.await
		.map_err(|e| format!("Database error: {}", e))?;

		let context_data = if let Some(row) = context_row {
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

		// Check if we have an active itinerary
		let has_itinerary = context_data.active_itinerary.is_some() 
			&& context_data.active_itinerary.as_ref().map(|it| {
				// Check if itinerary is not empty (has some structure)
				!it.is_null() && (!it.is_object() || !it.as_object().unwrap().is_empty())
			}).unwrap_or(false);

		let (message_text, message_id) = if has_itinerary {
			// We have an itinerary - for now, just create a placeholder message
			// TODO: In the future, another agent will create the actual itinerary in the DB
			// For now, we'll insert a message with the itinerary data as text
			let itinerary_text = serde_json::to_string_pretty(&context_data.active_itinerary)?;
			let default_message = format!(
				"I've created your itinerary! Here are the details:\n\n{}",
				itinerary_text
			);
			let message = optional_message.map(|s| s.to_string()).unwrap_or(default_message);

			// Insert message without itinerary_id for now (since we're not creating the itinerary yet)
			// TODO: When itinerary creation is implemented, create the itinerary and use its ID
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

		// Return a special marker that send_message_to_llm can detect
		// Format: "MESSAGE_INSERTED:<message_id>:<message_text>"
		let result = format!("MESSAGE_INSERTED:{}:{}", message_id, message_text);
		
		// Track this tool execution
		track_tool_execution(&self.pool, &self.chat_session_id, "respond_to_user", &input_clone, &result).await?;
		
		Ok(result)
	}
}

/// Gets the tools used by the Task Agent to build planning context.
/// These tools are focused on:
/// - retrieving user profile
/// - retrieving chat history/context
/// - parsing user intent
/// - asking for clarification when information is missing
/// - responding to the user
pub fn get_task_tools(
	llm: Arc<dyn LLM + Send + Sync>,
	pool: PgPool,
	chat_session_id: Arc<AtomicI32>,
	user_id: Arc<AtomicI32>,
) -> Vec<Arc<dyn Tool>> {
	vec![
		Arc::new(ParseUserIntentTool::new(
			Arc::clone(&llm),
			pool.clone(),
			Arc::clone(&chat_session_id),
		)),
		Arc::new(RetrieveChatContextTool::new(
			pool.clone(),
			Arc::clone(&chat_session_id),
		)),
		Arc::new(RetrieveUserProfileTool::new(
			pool.clone(),
			Arc::clone(&chat_session_id),
			Arc::clone(&user_id),
		)),
		Arc::new(AskForClarificationTool::new(
			Arc::clone(&llm),
			pool.clone(),
			Arc::clone(&chat_session_id),
		)),
		Arc::new(RespondToUserTool::new(pool, chat_session_id)),
	]
}


