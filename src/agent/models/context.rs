/*
   src/agent/models/context.rs
   File for Agent Context Models
   Purpose:
	   Store Agent Context Models

*/

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRoute {
	pub task_type: String, // "research", "constraint", "optimize"
	pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextData {
	pub user_profile: Option<Value>,
	pub chat_history: Vec<Value>,
	pub active_itinerary: Option<Value>,
	pub events: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialResult {
	pub agent: String,
	pub data: Value,
	pub success: bool,
	pub error: Option<String>,
}
