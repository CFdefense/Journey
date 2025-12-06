/*
 * src/agent/configs/orchestrator.rs
 *
 * File for Orchestrator Agent Configuration (Updated with shared LLM)
 */

use std::sync::Arc;

use langchain_rust::{
	agent::{AgentError, AgentExecutor, ConversationalAgent, ConversationalAgentBuilder},
	chain::options::ChainCallOptions,
	llm::openai::{OpenAI, OpenAIConfig, OpenAIModel},
	memory::SimpleMemory,
};

use crate::agent::configs::constraint::create_constraint_agent;
use crate::agent::configs::optimizer::create_optimize_agent;
use crate::agent::configs::research::create_research_agent;
use crate::agent::tools::orchestrator::{ORCHESTRATOR_SYSTEM_PROMPT, get_orchestrator_tools};

// Use a type alias for the agent type to make it easier to use
pub type AgentType = Arc<
	tokio::sync::Mutex<
		langchain_rust::agent::AgentExecutor<langchain_rust::agent::ConversationalAgent>,
	>,
>;

pub fn create_orchestrator_agent() -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
	// Load environment variables
	dotenvy::dotenv().ok();

	// Create a shared LLM instance for the orchestrator and its tools
	let llm = OpenAI::default().with_model(OpenAIModel::Gpt4oMini);

	// Create memory for conversation history
	let memory = SimpleMemory::new();

	// Create research agent
	let research_agent = Arc::new(tokio::sync::Mutex::new(Arc::new(tokio::sync::Mutex::new(
		create_research_agent(llm.clone()).unwrap(),
	))));

	// Create constraint agent
	let constraint_agent = Arc::new(tokio::sync::Mutex::new(Arc::new(tokio::sync::Mutex::new(
		create_constraint_agent(llm.clone()).unwrap(),
	))));

	// Create optimize agent
	let optimize_agent = Arc::new(tokio::sync::Mutex::new(Arc::new(tokio::sync::Mutex::new(
		create_optimize_agent(llm.clone()).unwrap(),
	))));

	// Get orchestrator tools
	let tools = get_orchestrator_tools(
		llm.clone(),
		research_agent,
		constraint_agent,
		optimize_agent,
	);

	// Create agent with system prompt and tools
	let agent = ConversationalAgentBuilder::new()
		.prefix(ORCHESTRATOR_SYSTEM_PROMPT.to_string())
		.tools(&tools)
		.options(ChainCallOptions::new().with_max_tokens(2000))
		.build(llm)
		.unwrap();

	Ok(AgentExecutor::from_agent(agent).with_memory(memory.into()))
}

/// Creates a dummy agent for testing purposes.
/// This agent will have an invalid API key and will panic if invoked,
/// but when DEPLOY_LLM != "1", the agent is never invoked, so this is safe.
/// This allows tests to run without requiring a valid OPENAI_API_KEY.
#[cfg(test)]
pub fn create_dummy_orchestrator_agent() -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
	// Set a dummy API key temporarily so agent creation doesn't fail
	let original_key = std::env::var("OPENAI_API_KEY").ok();

	unsafe {
		std::env::set_var("OPENAI_API_KEY", "sk-dummy-key-for-testing-only");
	}

	// Create a shared LLM instance
	let llm = OpenAI::default().with_model(OpenAIModel::Gpt4Turbo);

	// Create memory
	let memory = SimpleMemory::new();

	// For testing, create dummy sub-agents (they won't be invoked)
	let dummy_agent = Arc::new(tokio::sync::Mutex::new(create_dummy_sub_agent()?));
	let research_agent = Arc::clone(&dummy_agent);
	let constraint_agent = Arc::clone(&dummy_agent);
	let optimize_agent = Arc::clone(&dummy_agent);

	// Get tools with shared LLM
	let tools = get_orchestrator_tools(
		llm.clone(),
		Arc::new(tokio::sync::Mutex::new(research_agent)),
		Arc::new(tokio::sync::Mutex::new(constraint_agent)),
		Arc::new(tokio::sync::Mutex::new(optimize_agent)),
	);

	// Create agent with system prompt and tools
	let agent = ConversationalAgentBuilder::new()
		.prefix(ORCHESTRATOR_SYSTEM_PROMPT.to_string())
		.tools(&tools)
		.options(ChainCallOptions::new().with_max_tokens(2000))
		.build(llm)
		.unwrap();

	// Restore original key if it existed
	unsafe {
		if let Some(key) = original_key {
			std::env::set_var("OPENAI_API_KEY", key);
		} else {
			std::env::remove_var("OPENAI_API_KEY");
		}
	}

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
