/*
   src/agent/models/context.rs
   File for Agent Context Models
   Purpose:
	   Store Agent Context Models

*/

use crate::http_models::event::Event;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRoute {
	pub task_type: String, // "research", "constraint", "optimize"
	pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecution {
	pub tool_name: String,
	pub timestamp: String, // ISO 8601 format
	pub input: Value,
	pub output: Value,
	pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStage {
	Initial,           // Just started, parsing user input
	Researching,       // Research agent gathering events
	Constraining,      // Constraint agent validating events
	Optimizing,        // Optimizer agent ranking and scheduling
	Validating,        // Orchestrator validating final itinerary
	Complete,          // Pipeline complete, ready to display
	UserFeedback,      // Waiting for or processing user feedback
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextData {
	pub user_profile: Option<Value>,
	pub chat_history: Vec<Value>,
	pub active_itinerary: Option<Value>,
	pub events: Vec<Event>,              // Current running list of events being processed
	pub tool_history: Vec<ToolExecution>,
	pub pipeline_stage: Option<String>,  // Current stage in the pipeline
	pub researched_events: Vec<Event>,   // Events from research agent
	pub constrained_events: Vec<Event>,  // Events validated by constraint agent
	pub optimized_events: Vec<Event>,   // Events ranked/optimized by optimizer agent
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialResult {
	pub agent: String,
	pub data: Value,
	pub success: bool,
	pub error: Option<String>,
}
