/*
 * src/agent/tools/orchestrator.rs
 *
 * Orchestrator Agent Tools Implementation - LLM-based extraction
 */

use async_trait::async_trait;
use langchain_rust::language_models::llm::LLM;
use langchain_rust::llm::openai::{OpenAI, OpenAIModel, OpenAIConfig};
use langchain_rust::tools::Tool;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use langchain_rust::schemas::{Message};
use crate::agent::models::user::UserIntent;

/// Tool 1: Parse User Intent
/// Parses user input to extract intent, destination, dates, budget, and constraints.
/// Returns a UserIntent object.
#[derive(Clone)]
pub struct ParseUserIntentTool {
	llm: OpenAI<OpenAIConfig>,
}

impl ParseUserIntentTool {
	pub fn new(llm: OpenAI<OpenAIConfig>) -> Self {
		Self {
			llm: OpenAI::default().with_model(OpenAIModel::Gpt4oMini),
		}
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

		// Validate it's proper JSON
		let intent: UserIntent = serde_json::from_str(cleaned).map_err(|e| {
			format!(
				"Failed to parse LLM response as JSON: {}. Response was: {}",
				e, cleaned
			)
		})?;

		Ok(serde_json::to_string(&intent)?)
	}
}

/// Tool 2: Retrieve Chat Context
/// Retrieves chat history and context for the current conversation.
/// Returns a vector of Message objects.
#[derive(Clone)]
pub struct RetrieveChatContextTool {
	// TODO: Add database connection
	// db: Arc<Mutex<DbConnection>>,
}

impl RetrieveChatContextTool {
	pub fn new() -> Self {
		Self {}
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
		let chat_id = input["chat_id"]
			.as_str()
			.ok_or("chat_id must be a string")?;

		// TODO: Query database for chat history

		// Mock implementation
		let context = vec![
			json!({
				"role": "user",
				"content": "I want to visit Paris",
				"timestamp": "2024-01-01T10:00:00Z"
			}),
			json!({
				"role": "assistant",
				"content": "I can help you plan a trip to Paris!",
				"timestamp": "2024-01-01T10:00:05Z"
			}),
		];

		Ok(serde_json::to_string(&context)?)
	}
}

/// Tool 3: Retrieve User Profile
/// Retrieves user profile information including preferences and past trips.
/// Returns a UserProfile object.
#[derive(Clone)]
pub struct RetrieveUserProfileTool {
	// db: Arc<Mutex<DbConnection>>,
}

impl RetrieveUserProfileTool {
	pub fn new() -> Self {
		Self {}
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
		let user_id = input["user_id"]
			.as_str()
			.ok_or("user_id must be a string")?;

		// TODO: Query database for user profile

		let profile = json!({
			"user_id": user_id,
			"preferences": ["museums", "local_food", "walking_tours"],
			"dietary_restrictions": ["vegetarian"],
			"past_destinations": ["Rome", "Barcelona"],
			"budget_preference": "moderate"
		});

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
		json!({
			"type": "object",
			"properties": {
				"task_type": {
					"type": "string",
					"description": "The type of task to route",
					"enum": ["research", "constraint", "optimize"]
				},
				"payload": {
					"type": "object",
					"description": "The data to send to the sub-agent"
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

		let result = match task_type {
			"research" => {
				let agent = self.research_agent.lock().await;
				let _agent_inner = agent.lock().await;

				// TODO: Well Actually invoke agent when agents are fully implemented
				// let payload_str = serde_json::to_string(&payload)?;
				// let response = agent_inner.invoke(&payload_str).await?;

				json!({
					"agent": "research",
					"status": "completed",
					"data": {
						"attractions": ["Eiffel Tower", "Louvre Museum", "Notre-Dame"],
						"restaurants": ["Le Comptoir", "L'Ami Jean"],
						"hotels": ["Hotel Plaza", "Le Meurice"]
					}
				})
			}
			"constraint" => {
				let agent = self.constraint_agent.lock().await;
				let _agent_inner = agent.lock().await;

				json!({
					"agent": "constraint",
					"status": "completed",
					"data": {
						"budget_valid": true,
						"dietary_options_available": true,
						"accessibility_compatible": true,
						"warnings": []
					}
				})
			}
			"optimize" => {
				let agent = self.optimize_agent.lock().await;
				let _agent_inner = agent.lock().await;

				json!({
					"agent": "optimize",
					"status": "completed",
					"data": {
						"itinerary": [
							{"day": 1, "activities": ["Eiffel Tower", "Seine River Cruise"]},
							{"day": 2, "activities": ["Louvre Museum", "Tuileries Garden"]}
						],
						"estimated_cost": 2800,
						"optimization_score": 0.92
					}
				})
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
	llm: OpenAI<OpenAIConfig>,
}

impl MergePartialResultsTool {
	pub fn new(llm: OpenAI<OpenAIConfig>) -> Self {
		Self {
			llm: OpenAI::default().with_model(OpenAIModel::Gpt4oMini),
		}
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
		json!({
			"type": "object",
			"properties": {
				"results": {
					"type": "array",
					"description": "Array of PartialResult objects from sub-agents",
					"items": {
						"type": "object",
						"properties": {
							"agent": {"type": "string"},
							"data": {"type": "object"},
							"success": {"type": "boolean"},
							"error": {"type": ["string", "null"]}
						}
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
	llm: OpenAI<OpenAIConfig>,
}

impl AskForClarificationTool {
	pub fn new(llm: OpenAI<OpenAIConfig>) -> Self {
		Self {
			llm: OpenAI::default().with_model(OpenAIModel::Gpt4oMini),
		}
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
/// TODO: Implement DB connection for this.
#[derive(Clone)]
pub struct UpdateContextTool {
	// db: Arc<Mutex<DbConnection>>,
}

impl UpdateContextTool {
	pub fn new() -> Self {
		Self {}
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
		let _chat_id = input["chat_id"]
			.as_str()
			.ok_or("chat_id must be a string")?;
		let _updates = input["updates"].clone();

		// TODO: Persist to database

		Ok(json!({
			"status": "updated",
			"timestamp": chrono::Utc::now().to_rfc3339()
		})
		.to_string())
	}
}

/// Gets all the orchestrator tools.
/// Returns a vector of Arc<dyn Tool> objects.
pub fn get_orchestrator_tools(
	llm: OpenAI<OpenAIConfig>,
	research_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
	constraint_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
	optimize_agent: Arc<Mutex<crate::agent::configs::orchestrator::AgentType>>,
) -> Vec<Arc<dyn Tool>> {
	vec![
		Arc::new(ParseUserIntentTool::new(llm.clone())),
		Arc::new(RetrieveChatContextTool::new()),
		Arc::new(RetrieveUserProfileTool::new()),
		Arc::new(RouteTaskTool::new(
			research_agent,
			constraint_agent,
			optimize_agent,
		)),
		Arc::new(MergePartialResultsTool::new(llm.clone())),
		Arc::new(AskForClarificationTool::new(llm.clone())),
		Arc::new(UpdateContextTool::new()),
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
