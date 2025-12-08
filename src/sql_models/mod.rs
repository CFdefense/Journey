use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use utoipa::ToSchema;

pub mod account;
pub mod event_list;
pub mod itinerary;
pub mod message;

/// Budget preference enum mapped to Postgres `budget_bucket`.
/// Used in account preferences and returned by account APIs.
/// - Fields:
///   - Enum variants representing budget bands
#[derive(Debug, Serialize, Deserialize, Clone, Type, ToSchema)]
#[sqlx(type_name = "budget_bucket")]
pub enum BudgetBucket {
	VeryLowBudget,
	LowBudget,
	MediumBudget,
	HighBudget,
	LuxuryBudget,
}

/// Risk tolerance enum mapped to Postgres `risk_tolerence`.
/// Used in account preferences and returned by account APIs.
/// - Fields:
///   - Enum variants representing risk appetite
#[derive(Debug, Serialize, Deserialize, Clone, Type, ToSchema)]
#[sqlx(type_name = "risk_tolerence")]
pub enum RiskTolerence {
	ChillVibes,
	LightFun,
	Adventurer,
	RiskTaker,
}

/// The time of day the event will take place in the itinerary
#[derive(Debug, Serialize, Deserialize, Clone, Type, PartialEq)]
#[sqlx(type_name = "time_of_day")]
pub enum TimeOfDay {
	Morning,
	Afternoon,
	Evening,
}

/// The status of the LLM pipeline
#[derive(Debug, Serialize, Deserialize, Clone, Type, PartialEq, ToSchema)]
#[sqlx(type_name = "llm_progress")]
pub enum LlmProgress {
	Ready,
	Searching,
	Scheduling,
	Optimizing,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type, PartialEq, ToSchema)]
#[sqlx(type_name = "event_period")]
pub struct Period {
	pub open_date: Option<NaiveDate>,
	pub open_truncated: Option<bool>,
	pub open_day: i32,
	pub open_hour: i32,
	pub open_minute: i32,
	pub close_date: Option<NaiveDate>,
	pub close_truncated: Option<bool>,
	pub close_day: Option<i32>,
	pub close_hour: Option<i32>,
	pub close_minute: Option<i32>,
}
