/*
   src/agent/models/context.rs
   File for Agent Context Models
   Purpose:
	   Store Agent Context Models

*/

use crate::http_models::event::Event;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRoute {
	pub task_type: String, // "research", "constraint", "optimize"
	pub payload: Value,
}

/// TripContext: Single source of truth for all trip details
/// This object is progressively filled in as the user provides information
/// Instead of re-parsing chat history, we update this object incrementally
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TripContext {
	pub destination: Option<String>,
	pub start_date: Option<String>,  // ISO 8601 date format (YYYY-MM-DD)
	pub end_date: Option<String>,    // ISO 8601 date format (YYYY-MM-DD)
	pub budget: Option<f64>,         // Total budget in USD
	pub preferences: Vec<String>,    // ["cultural experiences", "beach time"] - OPTIONAL
	pub constraints: Vec<String>,    // Dietary, accessibility, etc. - pre-filled from profile
	pub action: Option<String>,      // "create", "modify", "view", "delete"
	pub itinerary_id: Option<i32>,   // For modify/view/delete actions
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
	pub chat_session_id: i32,            // Chat session this context belongs to
	pub user_id: i32,                     // User ID for this context
	pub user_profile: Option<Value>,
	pub chat_history: Vec<Value>,
	pub trip_context: TripContext,       // Single source of truth for trip details
	pub active_itinerary: Option<Value>,
	pub events: Vec<Event>,              // Current running list of events being processed
	pub tool_history: Vec<ToolExecution>,
	pub pipeline_stage: Option<String>,  // Current stage in the pipeline
	pub researched_events: Vec<Event>,   // Events from research agent
	pub constrained_events: Vec<Event>,  // Events validated by constraint agent
	pub optimized_events: Vec<Event>,    // Events ranked/optimized by optimizer agent
	pub constraints: Vec<String>,        // User constraints extracted from intent (dietary, accessibility, budget, etc.)
}

/// Shared in-memory store for per-chat ContextData.
///
/// Keyed by chat_session_id so all agents/tools in a conversation can
/// read/write the same contextual state without round-tripping through
/// the database on every tool call.
pub type SharedContextStore = Arc<RwLock<HashMap<i32, ContextData>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialResult {
	pub agent: String,
	pub data: Value,
	pub success: bool,
	pub error: Option<String>,
}
