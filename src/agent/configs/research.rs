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
	llm::openai::{OpenAI, OpenAIModel},
	memory::SimpleMemory,
};

use crate::agent::tools::research::*;

// Use a type alias for the agent type to make it easier to use
pub type AgentType = Arc<
	tokio::sync::Mutex<
		langchain_rust::agent::AgentExecutor<langchain_rust::agent::ConversationalAgent>,
	>,
>;

const SYSTEM_PROMPT: &str = "
You are an expert travel itinerary researcher responsible for discovering places of interest.

## Your Role
Create a list of Points of Interest (POIs) that travelers may be interest in by querying our database
and using the Google Maps Nearby Search API to gather POI data.

## Your Responsibilities

1. **Setup Queries**
   - Get the coordinates of the user's target location using geocoding
   - Use the location and keywords to create search filters for querying the database
   - Use the provided information to construct a request (or requests) for Google Maps Nearby Search

2. **Query the Database**
   - Search events in the database for each filter
   - Collect event data from database searches
   - Avoid duplicate or irrelevant events

3. **Nearby Search**
   - Prepare coordinates, field mask, type include/exclude list
   -
   -

4. **Update Database**
   -
   -
   -

## Output Requirements

Your final output must be a **complete structured itinerary** formatted for database storage, including:
- Day-by-day breakdown with dates
- Time-blocked activities (start time, end time, duration)
- POI details for each activity (name, location, category, cost estimate)
- Travel segments between activities (distance, duration, mode)
- Meal and rest breaks with suggestions
- Total daily costs and time commitments
- Energy level indicators for each day

## Optimization Priorities (in order)

1. **Safety & Accessibility**: Never recommend POIs that conflict with user disabilities or severe allergies
2. **Budget Compliance**: Stay within user's specified budget constraints
3. **Feasibility**: Ensure realistic timing with adequate travel and break time
4. **Enjoyment**: Maximize alignment with user interests and preferences
5. **Efficiency**: Minimize unnecessary travel and backtracking
6. **Balance**: Maintain sustainable energy levels throughout the trip

## Important Considerations

- Always account for real-world factors: traffic, lines, rest needs, meal times
- Be conservative with timing - it's better to under-schedule than over-schedule
- Consider the cumulative fatigue effect over multi-day trips
- Weather and seasonal factors may affect outdoor activities
- Some POIs may require advance booking or have limited availability
- Cultural and social norms may dictate appropriate timing for certain activities

When you receive POIs and user profile information, create an actionable plan to optimize the itinerary by methodically applying your tools.
";

pub fn create_research_agent() -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
	// Load environment variables
	dotenvy::dotenv().ok();

	// Note: Even when DEPLOY_LLM != "1", we still need to create an agent
	// (it won't be used at runtime). OpenAI API key is still required for agent creation.

	// Create memory
	let memory = SimpleMemory::new();

	// Select model (will read key from environment variable)
	let llm = OpenAI::default().with_model(OpenAIModel::Gpt4oMini);

	// Create agent with system prompt and tools
	let system_prompt = SYSTEM_PROMPT.to_string();

	let agent = ConversationalAgentBuilder::new()
		.prefix(system_prompt)
		.tools(&[Arc::new(GeocodeTool), Arc::new(NearbySearchTool)])
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

	// Select model
	let llm = OpenAI::default().with_model(OpenAIModel::Gpt4Turbo);

	// Create agent with system prompt and tools
	let system_prompt = SYSTEM_PROMPT.to_string();

	let agent = ConversationalAgentBuilder::new()
		.prefix(system_prompt)
		.tools(&[Arc::new(GeocodeTool), Arc::new(NearbySearchTool)])
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
