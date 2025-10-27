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
	//tools:: <- Some tools can be gotten from here
};

use crate::agent::tools::GreetingTool;

pub fn create_agent() -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
	// Load environment variables
	dotenvy::dotenv().ok();

	// Create memory
	let memory = SimpleMemory::new();

	// Get tools
	let greeting_tool = GreetingTool;

	// Select model (will read key from environment variable)
	let llm = OpenAI::default().with_model(OpenAIModel::Gpt4Turbo);

	// Create agent with system prompt and tools
	let system_prompt = format!(
		"You are a helpful AI assistant for planning travel itineraries. \
		You help users create and manage their trip plans with a friendly and professional demeanor. \
		Always be concise, clear, and focus on providing practical travel planning advice. \
		\
		User Info: \
		Name: {} \
		Location: {} \
		Preferences: {} \
		Budget: {} \
		Travel Dates: {} \
		Travel Type: {} \
		Travel Style: {} \
		Travel Budget: {} \
	",  "christian", 
		"Philadelphia", 
		"Adventurous", 
		"Cheap", 
		"August 1st - August 8th", 
		"Vacation", 
		"Adventurous", 
		"Cheap");
	
	let agent = ConversationalAgentBuilder::new()
		.prefix(system_prompt)
		.tools(&[
			Arc::new(greeting_tool),
		])
		.options(ChainCallOptions::new().with_max_tokens(1000))
		.build(llm)
		.unwrap();

	Ok(AgentExecutor::from_agent(agent).with_memory(memory.into()))
}