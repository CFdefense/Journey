/*
 * src/agent/configs/orchestrator.rs
 *
 * File for Orchestrator Agent Configuration
 *
 * Purpose:
 *   Store Orchestrator Agent Configuration
 */

use std::sync::Arc;

use langchain_rust::{
	agent::{AgentError, AgentExecutor, ConversationalAgent, ConversationalAgentBuilder},
	chain::options::ChainCallOptions,
	llm::openai::{OpenAI, OpenAIModel},
	memory::SimpleMemory,
};

use crate::agent::tools::orchestrator::*;

// Use a type alias for the agent type to make it easier to use
pub type AgentType = Arc<
	tokio::sync::Mutex<
		langchain_rust::agent::AgentExecutor<langchain_rust::agent::ConversationalAgent>,
	>,
>;

pub fn create_orchestrator_agent(
	research_agent: AgentType,
	constraint_agent: AgentType,
	optimize_agent: AgentType,
) -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
	// Load environment variables
	dotenvy::dotenv().ok();

	// Note: Even when DEPLOY_LLM != "1", we still need to create an agent
	// (it won't be used at runtime). OpenAI API key is still required for agent creation.

	// Create memory
	let memory = SimpleMemory::new();

	// Get tools
	// TODO: Add tools here will use the research, constraint, and optimize agents

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
pub fn create_dummy_orchestrator_agent() -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
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
