/*
 * src/agent/configs/optimizer.rs
 *
 * File for Optimizer Agent Configuration
 *
 * Purpose:
 *   Store Optimizer Agent Configuration
 */

use std::sync::Arc;

use langchain_rust::{
	agent::{AgentError, AgentExecutor, ConversationalAgent, ConversationalAgentBuilder},
	chain::options::ChainCallOptions,
	llm::openai::{OpenAI, OpenAIConfig, OpenAIModel},
	memory::SimpleMemory,
	tools::Tool,
};

use crate::agent::tools::optimizer::{
	ClusterPOIsTool, DeserializeEventsTool, OptimizeRouteTool, RankPOIsByPreferenceTool,
	SequenceDayTool,
};

use sqlx::PgPool;

// Use a type alias for the agent type to make it easier to use
pub type AgentType = Arc<
	tokio::sync::Mutex<
		langchain_rust::agent::AgentExecutor<langchain_rust::agent::ConversationalAgent>,
	>,
>;

pub fn create_optimize_agent(
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
	let tools: Vec<Arc<dyn Tool>> = vec![
		Arc::new(RankPOIsByPreferenceTool),
		Arc::new(ClusterPOIsTool),
		Arc::new(SequenceDayTool),
		Arc::new(OptimizeRouteTool),
		Arc::new(DeserializeEventsTool),
	];

	// Select model (will read key from environment variable)
	let llm = OpenAI::default().with_model(OpenAIModel::Gpt4oMini);

	// Create agent with system prompt
	let system_prompt = "
You are an expert travel itinerary optimizer responsible for creating feasible, enjoyable, and well-balanced travel schedules.

## Your Role
Create complete day-by-day itineraries from Points of Interest (POIs) that maximize traveler enjoyment while minimizing travel time and respecting user constraints.

## Your Responsibilities

1. **Rank and Filter POIs**
   - Evaluate POIs against user profile: budget, risk tolerance, allergies, disabilities
   - Score based on user interests and preferences
   - Filter out incompatible or inaccessible options

2. **Ensure Diversity**
   - Prevent clustering of similar activity types (e.g., avoid 3 museums in a row)
   - Balance indoor/outdoor activities
   - Mix cultural, recreational, dining, and relaxation experiences

3. **Build Daily Schedules**
   - Organize POIs into time blocks: Morning (6am-12pm), Afternoon (12pm-6pm), Evening (6pm-12am)
   - Respect venue operating hours and typical activity durations
   - Consider optimal times for specific activities (outdoor activities during daylight, etc.)

4. **Optimize Routes**
   - Minimize travel time and distance between locations
   - Group geographically proximate POIs when sensible
   - Consider transportation modes and accessibility

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
	".to_string();

	let agent = ConversationalAgentBuilder::new()
		.prefix(system_prompt)
		.tools(&tools)
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
pub fn create_dummy_optimize_agent() -> Result<AgentExecutor<ConversationalAgent>, AgentError> {
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
	let tools: Vec<Arc<dyn Tool>> = vec![
		Arc::new(RankPOIsByPreferenceTool),
		Arc::new(ClusterPOIsTool),
		Arc::new(SequenceDayTool),
		Arc::new(OptimizeRouteTool),
		Arc::new(DeserializeEventsTool),
	];

	// Select model
	let llm = OpenAI::default().with_model(OpenAIModel::Gpt4Turbo);

	// Create agent with system prompt
	let system_prompt = "
You are an expert travel itinerary optimizer responsible for creating feasible, enjoyable, and well-balanced travel schedules.

## Your Role
Create complete day-by-day itineraries from Points of Interest (POIs) that maximize traveler enjoyment while minimizing travel time and respecting user constraints.

## Your Responsibilities

1. **Rank and Filter POIs**
   - Evaluate POIs against user profile: budget, risk tolerance, allergies, disabilities
   - Score based on user interests and preferences
   - Filter out incompatible or inaccessible options

2. **Ensure Diversity**
   - Prevent clustering of similar activity types (e.g., avoid 3 museums in a row)
   - Balance indoor/outdoor activities
   - Mix cultural, recreational, dining, and relaxation experiences

3. **Build Daily Schedules**
   - Organize POIs into time blocks: Morning (6am-12pm), Afternoon (12pm-6pm), Evening (6pm-12am)
   - Respect venue operating hours and typical activity durations
   - Consider optimal times for specific activities (outdoor activities during daylight, etc.)

4. **Optimize Routes**
   - Minimize travel time and distance between locations
   - Group geographically proximate POIs when sensible
   - Consider transportation modes and accessibility

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
	".to_string();

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

	Ok(AgentExecutor::from_agent(agent).with_memory(memory.into()))
}
