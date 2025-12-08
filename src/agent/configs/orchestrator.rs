/*
 * src/agent/configs/orchestrator.rs
 *
 * File for Orchestrator Agent Configuration
 */

use std::sync::Arc;
use std::sync::atomic::AtomicI32;

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
use crate::agent::configs::task::create_task_agent;
use crate::agent::models::context::SharedContextStore;
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
) -> Result<
	(
		AgentExecutor<ConversationalAgent>,
		Arc<AtomicI32>,
		Arc<AtomicI32>,
		SharedContextStore,
	),
	AgentError,
> {
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

	// Create shared atomics for chat_session_id and user_id (will be set per request)
	let chat_session_id = Arc::new(AtomicI32::new(0));
	let user_id = Arc::new(AtomicI32::new(0));

	// In-memory context store shared by orchestrator + sub-agents
	let context_store: SharedContextStore =
		Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));

	// Create research agent
	let research_agent = Arc::new(tokio::sync::Mutex::new(Arc::new(tokio::sync::Mutex::new(
		create_research_agent(pool.clone()).unwrap(),
	))));

	// Create constraint agent
	let constraint_agent = Arc::new(tokio::sync::Mutex::new(Arc::new(tokio::sync::Mutex::new(
		create_constraint_agent(llm_for_subagents.clone(), pool.clone()).unwrap(),
	))));

	// Create optimize agent
	let optimize_agent = Arc::new(tokio::sync::Mutex::new(Arc::new(tokio::sync::Mutex::new(
		create_optimize_agent(llm_for_subagents.clone(), pool.clone()).unwrap(),
	))));

	// Create Task Agent (sub-agent used to build context and user profile)
	let task_agent_executor = create_task_agent(
		pool.clone(),
		Arc::clone(&chat_session_id),
		Arc::clone(&user_id),
		context_store.clone(),
	)?;
	let task_agent_inner: AgentType = Arc::new(tokio::sync::Mutex::new(task_agent_executor));
	let task_agent = Arc::new(tokio::sync::Mutex::new(task_agent_inner));

	// Get orchestrator tools
	let tools = get_orchestrator_tools(
		llm_for_tools,
		pool.clone(),
		task_agent,
		research_agent,
		constraint_agent,
		optimize_agent,
		chat_session_id.clone(),
		user_id.clone(),
		context_store.clone(),
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
	Ok((
		AgentExecutor::from_agent(agent)
			.with_memory(memory.into())
			.with_max_iterations(30),
		chat_session_id,
		user_id,
		context_store,
	))
}

/// Creates a dummy agent for testing purposes.
/// This agent will have an invalid API key and will panic if invoked,
/// but when DEPLOY_LLM != "1", the agent is never invoked, so this is safe.
/// This allows tests to run without requiring a valid OPENAI_API_KEY.
#[cfg(test)]
pub fn create_dummy_orchestrator_agent(
	pool: PgPool,
) -> Result<
	(
		AgentExecutor<ConversationalAgent>,
		Arc<AtomicI32>,
		Arc<AtomicI32>,
		SharedContextStore,
	),
	AgentError,
> {
	// Use MockLLM for testing to avoid API key requirements
	let llm = MockLLM;

	// Create memory
	let memory = SimpleMemory::new();

	let llm_arc = Arc::new(llm.clone());
	let chat_session_id = Arc::new(AtomicI32::new(0));
	let user_id = Arc::new(AtomicI32::new(0));
	let context_store: SharedContextStore =
		Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));

	// Dummy sub-agents (including a dummy Task Agent) share the same simple implementation
	let dummy_agent = Arc::new(tokio::sync::Mutex::new(create_dummy_sub_agent()?));
	let task_agent = Arc::clone(&dummy_agent);
	let research_agent = Arc::clone(&dummy_agent);
	let constraint_agent = Arc::clone(&dummy_agent);
	let optimize_agent = Arc::clone(&dummy_agent);
	let tools = get_orchestrator_tools(
		llm_arc,
		pool,
		Arc::new(tokio::sync::Mutex::new(task_agent)),
		Arc::new(tokio::sync::Mutex::new(research_agent)),
		Arc::new(tokio::sync::Mutex::new(constraint_agent)),
		Arc::new(tokio::sync::Mutex::new(optimize_agent)),
		chat_session_id.clone(),
		user_id.clone(),
		context_store.clone(),
	);

	let agent = ConversationalAgentBuilder::new()
		.prefix(ORCHESTRATOR_SYSTEM_PROMPT.to_string())
		.tools(&tools)
		.options(ChainCallOptions::new().with_max_tokens(2000))
		.build(llm)
		.unwrap();

	Ok((
		AgentExecutor::from_agent(agent).with_memory(memory.into()),
		chat_session_id,
		user_id,
		context_store,
	))
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
pub const ORCHESTRATOR_SYSTEM_PROMPT: &str = include_str!("../prompts/orchestrator.md");
