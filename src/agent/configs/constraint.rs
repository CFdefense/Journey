/*
 * src/agent/configs/constraint.rs
 *
 * File for Constraint Agent Configuration
 *
 * Purpose:
 *   Store Constraint Agent Configuration
 */

use std::sync::Arc;

use langchain_rust::{
	agent::{AgentError, AgentExecutor, ConversationalAgent, ConversationalAgentBuilder},
	chain::options::ChainCallOptions,
	llm::openai::{OpenAI, OpenAIConfig, OpenAIModel},
	memory::SimpleMemory,
};

use crate::agent::tools::constraint::*;
use sqlx::PgPool;

pub fn create_constraint_agent(
	_llm: OpenAI<OpenAIConfig>,
	pool: PgPool,
) -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
	// Load environment variables
	dotenvy::dotenv().ok();

	// Note: Even when DEPLOY_LLM != "1", we still need to create an agent
	// (it won't be used at runtime). OpenAI API key is still required for agent creation.

	// Create memory
	let memory = SimpleMemory::new();

	// Select model (will read key from environment variable)
	let llm = OpenAI::default().with_model(OpenAIModel::Gpt4oMini);

	// Get tools - pass LLM as Arc<dyn LLM> and database pool
	let llm_arc: Arc<dyn langchain_rust::language_models::llm::LLM + Send + Sync> =
		Arc::new(llm.clone());
	let tools = constraint_tools(llm_arc, pool);

	// Create agent with system prompt and tools
	const SYSTEM_PROMPT: &str = include_str!("../prompts/constraint.md");
	let system_prompt = SYSTEM_PROMPT.to_string();

	let agent = ConversationalAgentBuilder::new()
		.prefix(system_prompt)
		.tools(&tools)
		.options(ChainCallOptions::new().with_max_tokens(1000))
		.build(llm)
		.unwrap();

	// Limit to 3 iterations - agent should: 1) call tool, 2) get result, 3) return final answer
	Ok(AgentExecutor::from_agent(agent)
		.with_memory(memory.into())
		.with_max_iterations(3))
}

/// Creates a dummy agent for testing purposes.
/// This agent will have an invalid API key and will panic if invoked,
/// but when DEPLOY_LLM != "1", the agent is never invoked, so this is safe.
/// This allows tests to run without requiring a valid OPENAI_API_KEY.
#[cfg(test)]
pub fn create_dummy_constraint_agent(
	pool: PgPool,
) -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
	// Set a dummy API key temporarily so agent creation doesn't fail
	// The agent won't actually be used when DEPLOY_LLM != "1"
	let original_key = std::env::var("OPENAI_API_KEY").ok();

	// Set a dummy API key temporarily so agent creation doesn't fail
	unsafe {
		std::env::set_var("OPENAI_API_KEY", "sk-dummy-key-for-testing-only");
	}

	// Create memory
	let memory = SimpleMemory::new();

	// Select model
	let llm = OpenAI::default().with_model(OpenAIModel::Gpt4Turbo);

	// Get tools - pass LLM as Arc<dyn LLM> and database pool
	let llm_arc: Arc<dyn langchain_rust::language_models::llm::LLM + Send + Sync> =
		Arc::new(llm.clone());
	let tools = constraint_tools(llm_arc, pool);

	// Create agent with system prompt and tools
	const SYSTEM_PROMPT: &str = include_str!("../prompts/constraint.md");
	let system_prompt = SYSTEM_PROMPT.to_string();

	let agent = ConversationalAgentBuilder::new()
		.prefix(system_prompt)
		.tools(&tools)
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

	Ok(AgentExecutor::from_agent(agent)
		.with_memory(memory.into())
		.with_max_iterations(3))
}
