/*
	src/agent/models/user.rs
	File for Agent User Models
	Purpose:
		Store Agent User Models

*/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIntent {
	pub action: String, // "create_itinerary", "modify", "query"
	pub destination: Option<String>,
	pub start_date: Option<String>,
	pub end_date: Option<String>,
	pub budget: Option<f64>,
	pub preferences: Vec<String>,
	pub constraints: Vec<String>,
	pub missing_info: Vec<String>, // What information is still needed
}
