/*
 * src/agent/tools/orchestrator.rs
 *
 * Orchestrator Agent Tools Implementation - LLM-based extraction
 */

use crate::agent::models::user::UserIntent;
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
		"Parses user input using an LLM to extract intent, destination, dates, budget, and constraints."
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

		// Retrieve tool execution history from database
		let context_row = sqlx::query!(
			r#"SELECT context as "context: serde_json::Value" FROM chat_sessions WHERE id = $1"#,
			chat_id
		)
		.fetch_optional(&self.pool)
		.await
		.map_err(|e| format!("Database error: {}", e))?;

		let tool_history: Vec<crate::agent::models::context::ToolExecution> = context_row
			.and_then(|row| row.context)
			.and_then(|ctx| {
				ctx.get("tool_history")
					.cloned()
					.and_then(|th| serde_json::from_value::<Vec<crate::agent::models::context::ToolExecution>>(th).ok())
			})
			.unwrap_or_default();

		// Return chat history and tool history
		let result = json!({
			"chat_history": chat_history,
			"tool_history": tool_history
		});

		Ok(serde_json::to_string(&result)?)
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

/// Tool 5: Merge Partial Results
/// Merges results from multiple sub-agents into a coherent final output using an LLM.
/// Returns a TravelPlan object.
#[derive(Clone)]
pub struct MergePartialResultsTool {
	llm: Arc<dyn LLM + Send + Sync>,
}

impl MergePartialResultsTool {
	pub fn new<L: LLM + Send + Sync + 'static>(llm: L) -> Self {
		Self { llm: Arc::new(llm) }
	}
}

#[async_trait]
impl Tool for MergePartialResultsTool {
	fn name(&self) -> String {
		"merge_partial_results".to_string()
	}

	fn description(&self) -> String {
		"Merges results from multiple sub-agents into a coherent final output using an LLM."
			.to_string()
	}

	fn parameters(&self) -> Value {
		// Parameters match array of PartialResult model structure
		json!({
			"type": "object",
			"properties": {
				"results": {
					"type": "array",
					"description": "Array of PartialResult objects from sub-agents",
					"items": {
						"type": "object",
						"properties": {
							"agent": {
								"type": "string",
								"description": "Name of the agent that produced this result"
							},
							"data": {
								"type": "object",
								"description": "The result data from the agent (any JSON object)"
							},
							"success": {
								"type": "boolean",
								"description": "Whether the agent execution was successful"
							},
							"error": {
								"type": ["string", "null"],
								"description": "Error message if success is false"
							}
						},
						"required": ["agent", "data", "success"]
					}
				}
			},
			"required": ["results"]
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let results = input["results"]
			.as_array()
			.ok_or("results must be an array")?;

		let prompt = format!(
			r#"You are merging results from multiple travel planning agents.
 
 Agent Results:
 {}
 
 Create a cohesive travel plan that combines all the information. Return ONLY a JSON object with:
 {{
   "status": "success" or "partial_success" or "failed",
   "summary": "A natural language summary of the complete plan",
   "itinerary": {{
     "destination": string,
     "attractions": [array of places],
     "accommodation": [array of hotels],
     "daily_plan": [array of day objects with activities],
     "budget_breakdown": object with costs
   }},
   "constraints_validated": boolean,
   "warnings": [array of any issues or warnings],
   "next_steps": [array of actions user should take]
 }}
 
 Return ONLY the JSON object."#,
			serde_json::to_string_pretty(&results)?
		);

		let response = self.llm.invoke(&prompt).await?;

		let cleaned = response
			.trim()
			.trim_start_matches("```json")
			.trim_start_matches("```")
			.trim_end_matches("```")
			.trim();

		// Validate it's proper JSON
		let _validated: Value = serde_json::from_str(cleaned)?;

		Ok(cleaned.to_string())
	}
}

/// Tool 6: Ask for Clarification
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

		// Store tool execution history if provided
		if let Some(tool_exec) = updates.get("tool_execution") {
			// Get existing context
			let context_row = sqlx::query!(
				r#"SELECT context as "context: serde_json::Value" FROM chat_sessions WHERE id = $1"#,
				chat_id
			)
			.fetch_optional(&self.pool)
			.await
			.map_err(|e| format!("Database error: {}", e))?;

			let mut existing_context: Value = context_row
				.and_then(|row| row.context)
				.unwrap_or_else(|| json!({ "tool_history": [] }));

			// Parse tool execution and add to history
			if let Ok(tool_exec_obj) = serde_json::from_value::<crate::agent::models::context::ToolExecution>(tool_exec.clone()) {
				// Get or create tool_history array
				if !existing_context.get("tool_history").is_some() {
					existing_context["tool_history"] = json!([]);
				}

				// Add new tool execution (keep last 100 entries)
				if let Some(tool_history_array) = existing_context.get_mut("tool_history").and_then(|th| th.as_array_mut()) {
					tool_history_array.push(json!(tool_exec_obj));
					if tool_history_array.len() > 100 {
						tool_history_array.remove(0);
					}
				}

				sqlx::query!(
					r#"UPDATE chat_sessions SET context = $1::jsonb WHERE id = $2"#,
					existing_context as serde_json::Value,
					chat_id
				)
				.execute(&self.pool)
				.await
				.map_err(|e| format!("Database error: {}", e))?;
			}
		}

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
		Arc::new(MergePartialResultsTool {
			llm: Arc::clone(&llm),
		}),
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
 2. Retrieve relevant context (user profile, chat history, active itineraries)
 3. Route tasks to specialized sub-agents:
    - Research Agent: For gathering destination information, attractions, and recommendations
    - Constraint Agent: For validating dietary restrictions, accessibility needs, and budget
    - Optimize Agent: For creating optimal itineraries and routes
 4. Validate and merge results from sub-agents
 5. Ask for clarification when information is missing or ambiguous
 6. Update conversation context as needed
 
 Workflow:
 1. Use parse_user_intent to understand what the user wants
 2. Check if critical information is missing - if so, use ask_for_clarification
 3. Use retrieve_user_profile and retrieve_chat_context to get relevant background
 4. Use route_task to delegate work to appropriate sub-agents (can call multiple times)
 5. Use merge_partial_results to combine outputs into a coherent response
 6. Use update_context to save important decisions
 
 Always maintain context awareness and ensure a smooth, conversational experience.
 Be proactive - if the user's request is clear and complete, proceed with planning.
 If information is missing, ask for it naturally and conversationally.
 "#;
