use serde::{Deserialize, Serialize};
use sqlx::Type;

pub mod account;
pub mod event_list;
pub mod itinerary;
pub mod message;

/// Budget preference enum mapped to Postgres `budget_bucket`.
/// Used in account preferences and returned by account APIs.
/// - Fields:
///   - Enum variants representing budget bands
#[derive(Debug, Serialize, Deserialize, Clone, Type)]
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
#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[sqlx(type_name = "risk_tolerence")]
pub enum RiskTolerence {
    ChillVibes,
    LightFun,
    Adventurer,
    RiskTaker,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type, PartialEq)]
#[sqlx(type_name = "time_of_day")]
pub enum TimeOfDay {
	Morning,
	Noon,
	Afternoon,
	Evening
}