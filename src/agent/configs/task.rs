/*
 * src/agent/configs/task.rs
 *
 * File for Task Agent Configuration
 *
 * Purpose:
 *   Task Agent is a sub-agent used by the Orchestrator.
 *   Its sole responsibility is to gather and prepare context for planning:
 *   - Retrieve user profile & chat history
 *   - Parse user intent
 *   - Ask for clarification when needed
 *   - Persist this information into the shared chat context
 *
 * The Orchestrator Agent then uses this prepared context to route work
 * to the research / constraint / optimize agents.
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

use crate::agent::configs::mock::MockLLM;
use crate::agent::models::context::SharedContextStore;
use crate::agent::tools::task::task_tools;
use langchain_rust::language_models::llm::LLM;

/// Creates the Task Agent used as a sub-agent by the Orchestrator.
///
/// The Task Agent shares the same `chat_session_id` and `user_id` atomics
/// as the Orchestrator so all tools operate on the same conversation context.
pub fn create_task_agent(
	pool: PgPool,
	chat_session_id: Arc<AtomicI32>,
	user_id: Arc<AtomicI32>,
	context_store: SharedContextStore,
) -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
	// Load environment variables
	dotenvy::dotenv().ok();

	// Use MockLLM when DEPLOY_LLM != "1" so local/dev can run without a real key
	let use_mock = std::env::var("DEPLOY_LLM").unwrap_or_default() != "1";

	let llm_for_agent = OpenAI::default().with_model(OpenAIModel::Gpt4oMini);
	let llm_for_tools: Arc<dyn LLM + Send + Sync> = if use_mock {
		Arc::new(MockLLM)
	} else {
		Arc::new(llm_for_agent.clone())
	};

	// Create memory for conversation history
	let memory = SimpleMemory::new();

	// Tools focused on context building (profile, chat history, intent, clarification, respond)
	let tools = task_tools(
		llm_for_tools,
		pool,
		Arc::clone(&chat_session_id),
		Arc::clone(&user_id),
		context_store,
	);

	// Create agent with system prompt and tools
	let agent = if use_mock {
		let mock_llm = MockLLM;
		ConversationalAgentBuilder::new()
			.prefix(TASK_SYSTEM_PROMPT.to_string())
			.tools(&tools)
			.options(ChainCallOptions::new().with_max_tokens(2000))
			.build(mock_llm)
			.unwrap()
	} else {
		ConversationalAgentBuilder::new()
			.prefix(TASK_SYSTEM_PROMPT.to_string())
			.tools(&tools)
			.options(ChainCallOptions::new().with_max_tokens(2000))
			.build(llm_for_agent)
			.unwrap()
	};

	Ok(AgentExecutor::from_agent(agent)
		.with_memory(memory.into())
		.with_max_iterations(20))
}

/// Creates a dummy Task Agent for testing purposes.
///
/// Mirrors the dummy orchestrator agent but uses the Task Agent system prompt.
#[cfg(test)]
pub fn create_dummy_task_agent(
	pool: PgPool,
	chat_session_id: Arc<AtomicI32>,
	user_id: Arc<AtomicI32>,
) -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
	// Use MockLLM for testing to avoid API key requirements
	let llm = MockLLM;

	// Create memory
	let memory = SimpleMemory::new();

	// Dummy sub-agents (all the same simple agent)
	let llm_arc = Arc::new(llm.clone());

	// In-memory context store for tests
	let context_store: SharedContextStore =
		Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));

	let tools = task_tools(
		llm_arc,
		pool,
		Arc::clone(&chat_session_id),
		Arc::clone(&user_id),
		context_store,
	);

	let agent = ConversationalAgentBuilder::new()
		.prefix(TASK_SYSTEM_PROMPT.to_string())
		.tools(&tools)
		.options(ChainCallOptions::new().with_max_tokens(2000))
		.build(llm)
		.unwrap();

	Ok(AgentExecutor::from_agent(agent).with_memory(memory.into()))
}

/// The system prompt for the Task Agent.
pub const TASK_SYSTEM_PROMPT: &str = include_str!("../prompts/task.md");
