/*
 * src/agent/configs/orchestrator.rs
 *
 * File for Orchestrator Agent Configuration
 */

use std::sync::Arc;

use langchain_rust::{
	agent::{AgentError, AgentExecutor, ConversationalAgent, ConversationalAgentBuilder},
	chain::options::ChainCallOptions,
	llm::openai::{OpenAI, OpenAIModel},
	memory::SimpleMemory,
};

use sqlx::PgPool;

use crate::agent::configs::constraint::create_constraint_agent;
use crate::agent::configs::mock::MockLLM;
use crate::agent::configs::optimizer::create_optimize_agent;
use crate::agent::configs::research::create_research_agent;
use crate::agent::tools::orchestrator::get_orchestrator_tools;
use langchain_rust::language_models::llm::LLM;

// Use a type alias for the agent type to make it easier to use
pub type AgentType = Arc<
	tokio::sync::Mutex<
		langchain_rust::agent::AgentExecutor<langchain_rust::agent::ConversationalAgent>,
	>,
>;

pub fn create_orchestrator_agent(
	pool: PgPool,
) -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
	// Load environment variables
	dotenvy::dotenv().ok();

	// Create a shared LLM instance for the orchestrator and its tools
	// Use MockLLM if DEPLOY_LLM != "1", otherwise use OpenAI
	let use_mock = std::env::var("DEPLOY_LLM").unwrap_or_default() != "1";

	let llm_for_subagents = OpenAI::default().with_model(OpenAIModel::Gpt4oMini);
	let llm_for_tools: Arc<dyn LLM + Send + Sync> = if use_mock {
		Arc::new(MockLLM)
	} else {
		Arc::new(llm_for_subagents.clone())
	};

	// Create memory for conversation history
	let memory = SimpleMemory::new();

	// Create research agent
	let research_agent = Arc::new(tokio::sync::Mutex::new(Arc::new(tokio::sync::Mutex::new(
		create_research_agent(llm_for_subagents.clone(), pool.clone()).unwrap(),
	))));

	// Create constraint agent
	let constraint_agent = Arc::new(tokio::sync::Mutex::new(Arc::new(tokio::sync::Mutex::new(
		create_constraint_agent(llm_for_subagents.clone(), pool.clone()).unwrap(),
	))));

	// Create optimize agent
	let optimize_agent = Arc::new(tokio::sync::Mutex::new(Arc::new(tokio::sync::Mutex::new(
		create_optimize_agent(llm_for_subagents.clone(), pool.clone()).unwrap(),
	))));

	// Get orchestrator tools
	let tools = get_orchestrator_tools(
		llm_for_tools,
		pool.clone(),
		research_agent,
		constraint_agent,
		optimize_agent,
	);

	// Create agent with system prompt and tools
	let agent = if use_mock {
		let mock_llm = MockLLM;
		ConversationalAgentBuilder::new()
			.prefix(ORCHESTRATOR_SYSTEM_PROMPT.to_string())
			.tools(&tools)
			.options(ChainCallOptions::new().with_max_tokens(2000))
			.build(mock_llm)
			.unwrap()
	} else {
		ConversationalAgentBuilder::new()
			.prefix(ORCHESTRATOR_SYSTEM_PROMPT.to_string())
			.tools(&tools)
			.options(ChainCallOptions::new().with_max_tokens(2000))
		.build(llm_for_subagents)
		.unwrap()
	};

	// Create executor with increased max iterations for complex multi-agent workflows
	// Default is 10, but we need more for orchestrator → sub-agent → tools chains
	Ok(AgentExecutor::from_agent(agent)
		.with_memory(memory.into())
		.with_max_iterations(30))
}

/// Creates a dummy agent for testing purposes.
/// This agent will have an invalid API key and will panic if invoked,
/// but when DEPLOY_LLM != "1", the agent is never invoked, so this is safe.
/// This allows tests to run without requiring a valid OPENAI_API_KEY.
#[cfg(test)]
pub fn create_dummy_orchestrator_agent(
	pool: PgPool,
) -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
	// Use MockLLM for testing to avoid API key requirements
	let llm = MockLLM;

	// Create memory
	let memory = SimpleMemory::new();

	// Dummy sub-agents
	let dummy_agent = Arc::new(tokio::sync::Mutex::new(create_dummy_sub_agent()?));
	let research_agent = Arc::clone(&dummy_agent);
	let constraint_agent = Arc::clone(&dummy_agent);
	let optimize_agent = Arc::clone(&dummy_agent);

	let llm_arc = Arc::new(llm.clone());
	let tools = get_orchestrator_tools(
		llm_arc,
		pool,
		Arc::new(tokio::sync::Mutex::new(research_agent)),
		Arc::new(tokio::sync::Mutex::new(constraint_agent)),
		Arc::new(tokio::sync::Mutex::new(optimize_agent)),
	);

	let agent = ConversationalAgentBuilder::new()
		.prefix(ORCHESTRATOR_SYSTEM_PROMPT.to_string())
		.tools(&tools)
		.options(ChainCallOptions::new().with_max_tokens(2000))
		.build(llm)
		.unwrap();

	Ok(AgentExecutor::from_agent(agent).with_memory(memory.into()))
}

#[cfg(test)]
fn create_dummy_sub_agent() -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
	let memory = SimpleMemory::new();
	let llm = OpenAI::default().with_model(OpenAIModel::Gpt4Turbo);

	let agent = ConversationalAgentBuilder::new()
		.prefix("Dummy sub-agent".to_string())
		.options(ChainCallOptions::new().with_max_tokens(1000))
		.build(llm)
		.unwrap();

	Ok(AgentExecutor::from_agent(agent).with_memory(memory.into()))
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
   - If critical information is missing, use ask_for_clarification with the chat_id - this tool will STOP the pipeline by inserting the clarification message into the chat. After calling this tool, STOP processing immediately. Do NOT call any other tools after ask_for_clarification.
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

IMPORTANT: When calling tools that require JSON parameters (like missing_info, payload, updates, context):
- Always pass these as JSON STRINGS, not as JSON objects
- For arrays, serialize them: '["item1", "item2"]' not ["item1", "item2"]
- For objects, serialize them: '{"key":"value"}' not {"key":"value"}
- Example: missing_info should be '["destination", "dates"]' not ["destination", "dates"]
"#;
