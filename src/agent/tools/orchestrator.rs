/*
 * src/agent/tools/orchestrator.rs
 *
 * Orchestrator Agent Tools Implementation - LLM-based extraction
 */

use crate::agent::models::context::{ContextData, ToolExecution};
use crate::agent::models::user::UserIntent;
use crate::http_models::event::Event;
use async_trait::async_trait;
use langchain_rust::chain::Chain;
use langchain_rust::language_models::llm::LLM;
use langchain_rust::tools::Tool;
use serde_json::{Value, json};
use sqlx::PgPool;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Tool 1: Parse User Intent
/// Parses user input to extract intent, destination, dates, budget, and constraints.
/// Returns a UserIntent object.
#[derive(Clone)]
pub struct ParseUserIntentTool {
	llm: Arc<dyn LLM + Send + Sync>,
}

impl ParseUserIntentTool {
	pub fn new<L: LLM + Send + Sync + 'static>(llm: L) -> Self {
		Self { llm: Arc::new(llm) }
	}
}

#[async_trait]
impl Tool for ParseUserIntentTool {
	fn name(&self) -> String {
		"parse_user_intent".to_string()
	}

	fn description(&self) -> String {
		"Parses user input using an LLM to extract intent, destination, dates, budget, preferences, and constraints. Returns a UserIntent object with constraints array that should be stored in context for other agents to access."
             .to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {
				"user_message": {
					"type": "string",
					"description": "The raw user message to parse"
				}
			},
			"required": ["user_message"]
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let user_message = input["user_message"]
			.as_str()
			.ok_or("user_message must be a string")?;

		let prompt = format!(
			r#"Extract travel planning information from the user's message.
 
 User message: "{}"
 
 Extract the following information and return ONLY a valid JSON object with these fields:
 {{
   "action": "create_itinerary" | "modify_itinerary" | "query" | "other",
   "destination": string or null (city/country name),
   "start_date": string or null (ISO format YYYY-MM-DD if mentioned),
   "end_date": string or null (ISO format YYYY-MM-DD if mentioned),
   "budget": number or null (total budget in USD),
   "preferences": [array of strings - interests like "museums", "food", "nightlife", etc.],
   "constraints": [array of strings - dietary restrictions, accessibility needs, etc.],
   "missing_info": [array of strings - what critical information is missing]
 }}
 
 Rules:
 - If dates are relative (e.g., "next month", "in June"), convert to approximate ISO dates
 - Budget should be extracted as a number without currency symbols
 - Preferences include activities, interests, and travel style
 - Constraints include dietary restrictions, accessibility needs, budget limitations
 - missing_info should list critical missing information like "destination", "dates", "budget"
 
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

		// Return serialized UserIntent
		Ok(serde_json::to_string(&intent)?)
	}
}

/// Tool 2: Retrieve Chat Context
/// Retrieves chat history and context for the current conversation.
/// Returns a vector of Message objects.
#[derive(Clone)]
pub struct RetrieveChatContextTool {
	pool: PgPool,
}

impl RetrieveChatContextTool {
	pub fn new(pool: PgPool) -> Self {
		Self { pool }
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
			"properties": {
				"chat_id": {
					"type": "string",
					"description": "The ID of the chat/conversation to retrieve"
				}
			},
			"required": ["chat_id"]
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let chat_id_str = input["chat_id"]
			.as_str()
			.ok_or("chat_id must be a string")?;
		let chat_id: i32 = chat_id_str
			.parse()
			.map_err(|_| "chat_id must be a valid integer")?;

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

		// Return full context including pipeline state
		Ok(serde_json::to_string(&context_data)?)
	}
}

/// Tool 3: Retrieve User Profile
/// Retrieves user profile information including preferences and past trips.
/// Returns a UserProfile object.
#[derive(Clone)]
pub struct RetrieveUserProfileTool {
	pool: PgPool,
}

impl RetrieveUserProfileTool {
	pub fn new(pool: PgPool) -> Self {
		Self { pool }
	}
}

#[async_trait]
impl Tool for RetrieveUserProfileTool {
	fn name(&self) -> String {
		"retrieve_user_profile".to_string()
	}

	fn description(&self) -> String {
		"Retrieves user profile information including preferences and past trips.".to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {
				"user_id": {
					"type": "string",
					"description": "The ID of the user whose profile to retrieve"
				}
			},
			"required": ["user_id"]
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let user_id_str = input["user_id"]
			.as_str()
			.ok_or("user_id must be a string")?;
		let user_id: i32 = user_id_str
			.parse()
			.map_err(|_| "user_id must be a valid integer")?;

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

		Ok(serde_json::to_string(&profile)?)
	}
}

#[derive(Clone)]
pub struct RouteTaskTool {
	pub research_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
	pub constraint_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
	pub optimize_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
}

impl RouteTaskTool {
	pub fn new(
		research_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
		constraint_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
		optimize_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
	) -> Self {
		Self {
			research_agent,
			constraint_agent,
			optimize_agent,
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
		"Routes a task to the appropriate sub-agent (research, constraint, or optimize)."
			.to_string()
	}

	fn parameters(&self) -> Value {
		// Parameters match TaskRoute model structure
		json!({
			"type": "object",
			"properties": {
				"task_type": {
					"type": "string",
					"description": "The type of task to route: 'research', 'constraint', or 'optimize'",
					"enum": ["research", "constraint", "optimize"]
				},
				"payload": {
					"type": "object",
					"description": "The data to send to the sub-agent (any JSON object)"
				}
			},
			"required": ["task_type", "payload"]
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let task_type = input["task_type"]
			.as_str()
			.ok_or("task_type must be a string")?;
		let payload = input["payload"].clone();
		let payload_str = serde_json::to_string(&payload)?;

		let result = match task_type {
			"research" => {
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
						json!({
							"agent": "research",
							"status": "completed",
							"data": data
						})
					}
					Err(e) => json!({
						"agent": "research",
						"status": "error",
						"error": format!("{}", e)
					}),
				}
			}
			"constraint" => {
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
						json!({
							"agent": "constraint",
							"status": "completed",
							"data": data
						})
					}
					Err(e) => json!({
						"agent": "constraint",
						"status": "error",
						"error": format!("{}", e)
					}),
				}
			}
			"optimize" => {
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
						json!({
							"agent": "optimize",
							"status": "completed",
							"data": data
						})
					}
					Err(e) => json!({
						"agent": "optimize",
						"status": "error",
						"error": format!("{}", e)
					}),
				}
			}
			_ => {
				return Err(format!("Unknown task type: {}", task_type).into());
			}
		};

		Ok(serde_json::to_string(&result)?)
	}
}

/// Tool 5: Ask for Clarification
/// Generates a natural clarification question using an LLM when user input is ambiguous.
/// Returns a string containing the clarification question.
#[derive(Clone)]
pub struct AskForClarificationTool {
	llm: Arc<dyn LLM + Send + Sync>,
}

impl AskForClarificationTool {
	pub fn new<L: LLM + Send + Sync + 'static>(llm: L) -> Self {
		Self { llm: Arc::new(llm) }
	}
}

#[async_trait]
impl Tool for AskForClarificationTool {
	fn name(&self) -> String {
		"ask_for_clarification".to_string()
	}

	fn description(&self) -> String {
		"Generates a natural clarification question using an LLM when user input is ambiguous."
			.to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {
				"missing_info": {
					"type": "array",
					"description": "Array of strings describing what information is missing",
					"items": {"type": "string"}
				},
				"context": {
					"type": "string",
					"description": "Additional context about the conversation"
				}
			},
			"required": ["missing_info"]
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let missing_info = input["missing_info"]
			.as_array()
			.ok_or("missing_info must be an array")?;
		let context = input.get("context").unwrap_or(&Value::Null);

		let prompt = format!(
			r#"Generate a friendly, natural clarification question for a travel planning conversation.
 
 Missing Information: {}
 Conversation Context: {}
 
 Create a single, conversational question that asks for the missing information in a natural way.
 The question should be helpful and friendly.
 
 Return ONLY the question text, nothing else."#,
			serde_json::to_string(&missing_info)?,
			context
		);

		let response = self.llm.invoke(&prompt).await?;

		Ok(response.trim().to_string())
	}
}

/// Tool 7: Update Context
/// Updates conversation context with new information.
/// Used to store the conversation context in the database.
#[derive(Clone)]
pub struct UpdateContextTool {
	pool: PgPool,
}

impl UpdateContextTool {
	pub fn new(pool: PgPool) -> Self {
		Self { pool }
	}
}

#[async_trait]
impl Tool for UpdateContextTool {
	fn name(&self) -> String {
		"update_context".to_string()
	}

	fn description(&self) -> String {
		"Updates conversation context with new information.".to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {
				"chat_id": {
					"type": "string",
					"description": "The chat/conversation ID"
				},
				"updates": {
					"type": "object",
					"description": "The context updates to persist"
				}
			},
			"required": ["chat_id", "updates"]
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let chat_id_str = input["chat_id"]
			.as_str()
			.ok_or("chat_id must be a string")?;
		let chat_id: i32 = chat_id_str
			.parse()
			.map_err(|_| "chat_id must be a valid integer")?;
		let updates = input["updates"].clone();

		// Update chat session title if provided
		if let Some(title) = updates.get("title").and_then(|t| t.as_str()) {
			sqlx::query!(
				"UPDATE chat_sessions SET title = $1 WHERE id = $2",
				title,
				chat_id
			)
			.execute(&self.pool)
			.await
			.map_err(|e| format!("Database error: {}", e))?;
		}

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
					.unwrap_or_else(|_| {
						// Build from existing structure
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

		// Update pipeline stage if provided
		if let Some(stage) = updates.get("pipeline_stage").and_then(|s| s.as_str()) {
			context_data.pipeline_stage = Some(stage.to_string());
		}

		// Update events list if provided (current running list)
		if let Some(events) = updates.get("events").and_then(|e| e.as_array()) {
			context_data.events = events.iter()
				.filter_map(|v| serde_json::from_value::<Event>(v.clone()).ok())
				.collect();
		}

		// Update researched_events if provided
		if let Some(researched) = updates.get("researched_events").and_then(|e| e.as_array()) {
			context_data.researched_events = researched.iter()
				.filter_map(|v| serde_json::from_value::<Event>(v.clone()).ok())
				.collect();
		}

		// Update constrained_events if provided
		if let Some(constrained) = updates.get("constrained_events").and_then(|e| e.as_array()) {
			context_data.constrained_events = constrained.iter()
				.filter_map(|v| serde_json::from_value::<Event>(v.clone()).ok())
				.collect();
		}

		// Update optimized_events if provided
		if let Some(optimized) = updates.get("optimized_events").and_then(|e| e.as_array()) {
			context_data.optimized_events = optimized.iter()
				.filter_map(|v| serde_json::from_value::<Event>(v.clone()).ok())
				.collect();
		}

		// Update active_itinerary if provided
		if let Some(itinerary) = updates.get("active_itinerary") {
			context_data.active_itinerary = Some(itinerary.clone());
		}

		// Update constraints if provided (from parsed user intent)
		if let Some(constraints) = updates.get("constraints").and_then(|c| c.as_array()) {
			context_data.constraints = constraints.iter()
				.filter_map(|v| v.as_str().map(|s| s.to_string()))
				.collect();
		}

		// Store tool execution history if provided
		if let Some(tool_exec) = updates.get("tool_execution") {
			if let Ok(tool_exec_obj) = serde_json::from_value::<ToolExecution>(tool_exec.clone()) {
				context_data.tool_history.push(tool_exec_obj);
				// Keep last 100 entries
				if context_data.tool_history.len() > 100 {
					context_data.tool_history.remove(0);
				}
			}
		}

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

		Ok(json!({
			"status": "updated",
			"chat_id": chat_id,
			"timestamp": chrono::Utc::now().to_rfc3339()
		})
		.to_string())
	}
}

/// Gets all the orchestrator tools.
/// Returns a vector of Arc<dyn Tool> objects.
pub fn get_orchestrator_tools(
	llm: Arc<dyn LLM + Send + Sync>,
	pool: PgPool,
	research_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
	constraint_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
	optimize_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
) -> Vec<Arc<dyn Tool>> {
	vec![
		Arc::new(ParseUserIntentTool {
			llm: Arc::clone(&llm),
		}),
		Arc::new(RetrieveChatContextTool::new(pool.clone())),
		Arc::new(RetrieveUserProfileTool::new(pool.clone())),
		Arc::new(RouteTaskTool::new(
			research_agent,
			constraint_agent,
			optimize_agent,
		)),
		Arc::new(AskForClarificationTool {
			llm: Arc::clone(&llm),
		}),
		Arc::new(UpdateContextTool::new(pool)),
	]
}

/// The system prompt for the Orchestrator Agent.
pub const ORCHESTRATOR_SYSTEM_PROMPT: &str = r#"
You are the Orchestrator Agent, the central brain of a multi-agent travel planning system.

Your responsibilities:
1. Parse user input to understand their travel intent, destination, dates, budget, and constraints
2. Retrieve relevant context (user profile, chat history, active itineraries, pipeline state)
3. Guide the workflow through the pipeline stages:
   - Initial: Parse intent, load context, check for missing info
   - Researching: Route to Research Agent to gather events and POIs
   - Constraining: Route to Constraint Agent to validate timing, budget, accessibility
   - Optimizing: Route to Optimizer Agent to rank POIs and build schedule
   - Validating: Validate pipeline completion and final itinerary
   - Complete: Display final readable itinerary to user
   - UserFeedback: Handle user feedback and route to relevant agent
4. Maintain the running list of events as they progress through the pipeline
5. Update context with pipeline stage and events at each stage
6. Ask for clarification when information is missing or ambiguous

Pipeline Workflow:
1. INITIAL STAGE:
   - Use parse_user_intent to understand what the user wants
   - Extract constraints from the parsed intent and use update_context to store them in context.constraints
   - Use retrieve_user_profile and retrieve_chat_context to get relevant background
   - Check pipeline_stage in context to see if we're continuing or starting fresh
   - If critical information is missing, use ask_for_clarification and set pipeline_stage to "initial"
   - If complete, set pipeline_stage to "researching" and proceed

2. RESEARCHING STAGE:
   - Use route_task with task_type "research" to gather events and POIs
   - Update context with researched_events from the research agent response
   - Set pipeline_stage to "constraining" when research is complete
   - Update events field with the researched events

3. CONSTRAINING STAGE:
   - Use route_task with task_type "constraint" to validate events
   - Pass the researched_events to the constraint agent
   - Update context with constrained_events from the constraint agent response
   - Set pipeline_stage to "optimizing" when constraints are validated
   - Update events field with the constrained events

4. OPTIMIZING STAGE:
   - Use route_task with task_type "optimize" to rank POIs and build schedule
   - Pass the constrained_events to the optimizer agent
   - Update context with optimized_events and active_itinerary from optimizer response
   - Set pipeline_stage to "validating" when optimization is complete
   - Update events field with the optimized events

5. VALIDATING STAGE:
   - Validate that the itinerary is complete and coherent
   - Set pipeline_stage to "complete" when validation passes
   - Update active_itinerary with the final itinerary

6. COMPLETE STAGE:
   - Display the final readable itinerary to the user
   - Wait for user feedback

7. USER FEEDBACK:
   - If user provides feedback, set pipeline_stage to "user_feedback"
   - Route to the appropriate agent based on feedback type
   - Update context and return to appropriate stage

Always use update_context to:
- Set pipeline_stage when moving between stages
- Update events with the current running list
- Update researched_events, constrained_events, optimized_events as they're produced
- Update active_itinerary when a complete itinerary is ready

Maintain context awareness and ensure a smooth, conversational experience.
Be proactive - if the user's request is clear and complete, proceed through the pipeline.
If information is missing, ask for it naturally and conversationally.
"#;
