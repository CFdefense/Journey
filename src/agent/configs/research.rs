/*
 * src/agent/configs/research.rs
 *
 * File for Research Agent Configuration
 *
 * Purpose:
 *   Store Research Agent Configuration
 */

use std::sync::Arc;

use langchain_rust::{
	agent::{AgentError, AgentExecutor, ConversationalAgent, ConversationalAgentBuilder},
	chain::options::ChainCallOptions,
	llm::openai::{OpenAI, OpenAIConfig, OpenAIModel},
	memory::SimpleMemory,
};

use sqlx::PgPool;

use crate::agent::tools::research::*;

// Use a type alias for the agent type to make it easier to use
pub type AgentType = Arc<
	tokio::sync::Mutex<
		langchain_rust::agent::AgentExecutor<langchain_rust::agent::ConversationalAgent>,
	>,
>;

pub fn create_research_agent(
	llm: OpenAI<OpenAIConfig>,
	pool: PgPool,
) -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
	// Load environment variables
	dotenvy::dotenv().ok();

	// Note: Even when DEPLOY_LLM != "1", we still need to create an agent
	// (it won't be used at runtime). OpenAI API key is still required for agent creation.

	// Create memory
	let memory = SimpleMemory::new();

	// Get tools
	// TODO: Add tools here

	// TODO initialize llm with the provided llm

	// Select model (will read key from environment variable)
	let llm = OpenAI::default().with_model(OpenAIModel::Gpt4oMini);

	// Create agent with system prompt and tools
	let system_prompt = "".to_string(); // TODO: Add agent-specific system prompt here

	let agent = ConversationalAgentBuilder::new()
		.prefix(system_prompt)
		// TODO: Add tools here
		.options(ChainCallOptions::new().with_max_tokens(1000))
		.build(llm)
		.unwrap();

	Ok(AgentExecutor::from_agent(agent).with_memory(memory.into()))
}

/// Creates a dummy agent for testing purposes.
/// This agent will have an invalid API key and will panic if invoked,
/// but when DEPLOY_LLM != "1", the agent is never invoked, so this is safe.
/// This allows tests to run without requiring a valid OPENAI_API_KEY.
#[cfg(test)]
pub fn create_dummy_research_agent() -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
	// Set a dummy API key temporarily so agent creation doesn't fail
	// The agent won't actually be used when DEPLOY_LLM != "1"
	let original_key = std::env::var("OPENAI_API_KEY").ok();

	// Set a dummy API key temporarily so agent creation doesn't fail
	unsafe {
		std::env::set_var("OPENAI_API_KEY", "sk-dummy-key-for-testing-only");
	}

	// Create memory
	let memory = SimpleMemory::new();

	// Get tools
	// TODO: Add tools here

	// Select model
	let llm = OpenAI::default().with_model(OpenAIModel::Gpt4Turbo);

	// Create agent with system prompt and tools
	let system_prompt = "".to_string(); // TODO: Add agent-specific system prompt here

	let agent = ConversationalAgentBuilder::new()
		.prefix(system_prompt)
		// TODO: Add tools here
		.options(ChainCallOptions::new().with_max_tokens(1000))
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
