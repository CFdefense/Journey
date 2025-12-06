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
use crate::agent::configs::optimizer::create_optimize_agent;
use crate::agent::configs::research::create_research_agent;
use crate::agent::tools::orchestrator::{ORCHESTRATOR_SYSTEM_PROMPT, get_orchestrator_tools};
use async_trait::async_trait;
use futures::stream::{self, Stream};
use langchain_rust::language_models::GenerateResult;
use langchain_rust::language_models::LLMError;
use langchain_rust::language_models::llm::LLM;
use langchain_rust::schemas::{Message, StreamData};
use serde_json::Value;
use std::pin::Pin;

/// Mock LLM implementation for testing that returns dummy responses
/// without making actual API calls
#[derive(Clone)]
pub struct MockLLM;

#[async_trait]
impl LLM for MockLLM {
	async fn generate(&self, _messages: &[Message]) -> Result<GenerateResult, LLMError> {
		Ok(GenerateResult {
			generation: "This is a mock response for testing.".to_string(),
			tokens: None,
		})
	}

	async fn stream(
		&self,
		_messages: &[Message],
	) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, LLMError>> + Send>>, LLMError> {
		let response = StreamData::new(
			Value::String("This is a mock response for testing.".to_string()),
			None,
			"This is a mock response for testing.",
		);
		let stream = stream::once(async move { Ok(response) });
		Ok(Box::pin(stream))
	}
}

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

	Ok(AgentExecutor::from_agent(agent).with_memory(memory.into()))
}

/// Creates a dummy agent for testing purposes.
/// This agent will have an invalid API key and will panic if invoked,
/// but when DEPLOY_LLM != "1", the agent is never invoked, so this is safe.
/// This allows tests to run without requiring a valid OPENAI_API_KEY.
#[cfg(test)]
pub fn create_dummy_orchestrator_agent() -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
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
