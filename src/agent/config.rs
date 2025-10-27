/*
 * src/agent/config.rs
 *
 * File for Agent Configuration
 *
 * Purpose:
 *   Store Agent Configuration
 */


use std::sync::Arc;

use langchain_rust::{
	agent::{AgentExecutor, ConversationalAgentBuilder, ConversationalAgent, AgentError},
	chain::options::ChainCallOptions,
	llm::openai::{OpenAI, OpenAIModel},
	memory::SimpleMemory,
	tools::CommandExecutor,
};

pub fn create_agent() -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
	// Load environment variables
	dotenvy::dotenv().ok();

	// Create memory
	let memory = SimpleMemory::new();
	let command_executor = CommandExecutor::default();

	// Select model (will read key from environment variable)
	let llm = OpenAI::default().with_model(OpenAIModel::Gpt4Turbo);

	// Create agent
	let agent = ConversationalAgentBuilder::new()
		.tools(&[Arc::new(command_executor)])
		.options(ChainCallOptions::new().with_max_tokens(1000))
		.build(llm)
		.unwrap();

	Ok(AgentExecutor::from_agent(agent).with_memory(memory.into()))
}