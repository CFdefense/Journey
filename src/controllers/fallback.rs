// pray to our lord and savior Terry Davis that this works

use std::{
	error::Error,
	sync::{Arc, atomic::AtomicI32},
};

use async_trait::async_trait;
use axum::{Extension, Json};
use google_maps::{
	Client,
	places_new::{Field, FieldMask, PlaceType},
};
use langchain_rust::{
	agent::{AgentError, AgentExecutor, ConversationalAgent, ConversationalAgentBuilder},
	chain::options::ChainCallOptions,
	language_models::llm::LLM,
	llm::{OpenAI, OpenAIModel},
	memory::SimpleMemory,
	tools::Tool,
};
use num_traits::ToPrimitive;
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool};

use crate::{
	agent::models::context::SharedContextStore,
	error::{ApiResult, AppError},
	global::GOOGLE_MAPS_API_KEY,
	http_models::{event::Event, itinerary::Itinerary},
	sql_models::{Period, RiskTolerence, BudgetBucket},
	middleware::AuthUser,
};

pub struct FallbackAgent {
	pub agent: AgentExecutor<ConversationalAgent>,
	pub chat_session_id: Arc<AtomicI32>,
	pub account_id: Arc<AtomicI32>,
	pub context: SharedContextStore,
}

// Define a struct to capture the RETURNING clause results
#[derive(FromRow)]
struct EventInsertResult {
	id: i32,
	event_name: String,
}

#[derive(Deserialize)]
struct FallbackRequest {
	user_prompt: String,
}

pub async fn fallback_gen_itinerary(
	Extension(user): Extension<AuthUser>,
	Extension(pool): Extension<PgPool>,
	Json(FallbackRequest { user_prompt }): Json<FallbackRequest>,
) -> ApiResult<Json<Itinerary>> {
	todo!("invoke fallback agent")
}

pub fn create_fallback_agent(pool: PgPool) -> Result<FallbackAgent, AgentError> {
	let llm = OpenAI::default().with_model(OpenAIModel::Gpt4oMini);
	let memory = SimpleMemory::new();

	// Create shared atomics for chat_session_id and user_id (will be set per request)
	let chat_session_id = Arc::new(AtomicI32::new(0));
	let user_id = Arc::new(AtomicI32::new(0));

	// In-memory context store shared by orchestrator + sub-agents
	let context_store: SharedContextStore =
		Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));

	// let tools = get_orchestrator_tools(
	// 	llm,
	// 	pool,
	// 	chat_session_id.clone(),
	// 	user_id.clone(),
	// 	context_store.clone(),
	// );
	let tools: [Arc<dyn Tool>; 2] = [
		Arc::new(InputTool {llm: Arc::new(llm.clone())}),
		Arc::new(GenTool {llm: Arc::new(llm.clone()), pool})
	];

	// Create agent with system prompt and tools
	let agent = ConversationalAgentBuilder::new()
		.prefix(include_str!("../agent/prompts/fallback.md").to_string())
		.tools(&tools)
		.options(ChainCallOptions::new().with_max_tokens(2000))
		.build(llm)
		.unwrap();

	// Create executor with increased max iterations for complex multi-agent workflows
	// Default is 10, but we need more for orchestrator → sub-agent → tools chains
	Ok(FallbackAgent {
		agent: AgentExecutor::from_agent(agent)
			.with_memory(memory.into())
			.with_max_iterations(30),
		chat_session_id,
		account_id: user_id,
		context: context_store,
	})
}

#[derive(Clone)]
pub struct InputTool {
	llm: Arc<dyn LLM + Send + Sync>,
}

#[derive(Clone)]
pub struct GenTool {
	llm: Arc<dyn LLM + Send + Sync>,
	pool: PgPool,
}

#[async_trait]
impl Tool for InputTool {
	fn name(&self) -> String {
		"Input Handling Tool".to_string()
	}

	fn description(&self) -> String {
		"Gather information from the user's prompt and return a JSON object with important data needed to generate an itinerary."
			.to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {
				"input_prompt": {
					"type": "string",
					"description": "The user's input prompt."
				}
			},
			"required": ["input_prompt"]
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let user_prompt = input["input_prompt"]
			.as_str()
			.ok_or("input_prompt must be a string")?;
		let response = self
			.llm
			.invoke(
				format!(
					include_str!("../agent/prompts/fallback_input.md"),
					user_prompt
				)
				.as_str(),
			)
			.await?;

		// TODO we can check if all the fields are provided in the object
		Ok(response
			.trim_start_matches("```json")
			.trim_end_matches("```")
			.trim()
			.to_string())
	}
}

#[async_trait]
impl Tool for GenTool {
	fn name(&self) -> String {
		"Itinerary Generator".to_string()
	}

	fn description(&self) -> String {
		"Use the information provided by the Input Handling Tool to generate an itinerary."
			.to_string()
	}

	fn parameters(&self) -> Value {
		json!({
			"type": "object",
			"properties": {
				"city": {
					"type": "string",
					"description": "The city the user wants to go to."
				},
				"country": {
					"type": "string",
					"description": "The country of the city the user wants to go to."
				},
				"start_date": {
					"type": "string",
					"description": "The first date in the generated itinerary in ISO 8601 date format."
				},
				"end_date": {
					"type": "string",
					"description": "The last date in the generated itinerary in ISO 8601 date format."
				},
				"context": {
					"type": "string",
					"description": "Any relevant context to help generate the itinerary according to the user's preferences."
				},
			},
			"required": ["city", "country", "start_date", "end_date", "context"]
		})
	}

	async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
		let city = input["city"].as_str().ok_or("city must be a string")?;
		let country = input["country"]
			.as_str()
			.ok_or("country must be a string")?;
		let start_date = input["start_date"]
			.as_str()
			.ok_or("start_date must be a string")?;
		let end_date = input["end_date"]
			.as_str()
			.ok_or("end_date must be a string")?;
		let context = input["context"]
			.as_str()
			.ok_or("context must be a string")?;
		let location = format!("{city}, {country}");

		let gm_api_key = std::env::var(GOOGLE_MAPS_API_KEY)?;
		let gm_client = google_maps::Client::try_new(gm_api_key).map_err(AppError::from)?;

		// use geocoding to get coords of location
		let geocode_res = gm_client
			.geocoding()
			.with_address(location.as_str())
			.execute()
			.await?;
		if let Some(err) = geocode_res.error_message {
			return Err(format!(
				"Geocoding failed with status {} - {err}",
				geocode_res.status
			)
			.into());
		}
		if !matches!(geocode_res.status, google_maps::geocoding::Status::Ok) {
			return Err(format!("Geocoding failed with status {}", geocode_res.status).into());
		}
		if geocode_res.results.is_empty() {
			return Err(format!("Geocoding could not get coordinates for {location}").into());
		}

		let lat = geocode_res.results[0]
			.geometry
			.location
			.lat
			.to_f64()
			.ok_or("lat could not be obtained from geocoding")?;
		let lng = geocode_res.results[0]
			.geometry
			.location
			.lng
			.to_f64()
			.ok_or("lng could not be obtained from geocoding")?;

		// use google maps api to get nearby places
		// Query DB and do nearby search concurrently
		let db_query_task = async move {
			sqlx::query_as!(
				Event,
				r#"SELECT
					id,
					event_name,
					event_description,
					street_address,
					city,
					country,
					postal_code,
					lat,
					lng,
					event_type,
					user_created,
					hard_start,
					hard_end,
					timezone,
					place_id,
					wheelchair_accessible_parking,
					wheelchair_accessible_entrance,
					wheelchair_accessible_restroom,
					wheelchair_accessible_seating,
					serves_vegetarian_food,
					price_level,
					utc_offset_minutes,
					website_uri,
					types,
					photo_name,
					photo_width,
					photo_height,
					photo_author,
					photo_author_uri,
					photo_author_photo_uri,
					weekday_descriptions,
					secondary_hours_type,
					next_open_time,
					next_close_time,
					open_now,
					periods as "periods: Vec<Period>",
					special_days,
					NULL::int as block_index
				FROM events
				WHERE city ILIKE $1 AND country ILIKE $2
				"#,
				city,
				country
			)
			.fetch_all(&self.pool)
			.await
			.map_err(|e| format!("Failed to fetch events from database - {e}"))
		};

		let (
			nearby_search_50,
			nearby_search_5,
			db_query_events
		) = tokio::join!(
			nearby_search(lat, lng, 50_000., &gm_client),
			nearby_search(lat, lng, 5_000., &gm_client),
			db_query_task
		);
		let mut nearby_search_events = nearby_search_50?;
		nearby_search_events.append(&mut nearby_search_5?);
		nearby_search_events.sort_unstable_by(|a, b| a.place_id.cmp(&b.place_id));
		nearby_search_events.dedup_by(|a, b| a.place_id == b.place_id);

		// Insert nearby search events into db
		let mut results = db_query_events?;
		results.reserve(nearby_search_events.len());

		for mut ev in nearby_search_events.into_iter() {
			let inserted = sqlx::query!(
				r#"
				INSERT INTO events (
					event_name,
					event_description,
					street_address,
					city,
					country,
					postal_code,
					lat,
					lng,
					event_type,
					user_created,
					hard_start,
					hard_end,
					timezone,
					place_id,
					wheelchair_accessible_parking,
					wheelchair_accessible_entrance,
					wheelchair_accessible_restroom,
					wheelchair_accessible_seating,
					serves_vegetarian_food,
					price_level,
					utc_offset_minutes,
					website_uri,
					types,
					photo_name,
					photo_width,
					photo_height,
					photo_author,
					photo_author_uri,
					photo_author_photo_uri,
					weekday_descriptions,
					secondary_hours_type,
					next_open_time,
					next_close_time,
					open_now,
					periods,
					special_days
				)
				VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36)
				ON CONFLICT (place_id) DO UPDATE SET
					event_name = EXCLUDED.event_name,
					event_description = EXCLUDED.event_description,
					street_address = EXCLUDED.street_address,
					city = EXCLUDED.city,
					country = EXCLUDED.country,
					postal_code = EXCLUDED.postal_code,
					lat = EXCLUDED.lat,
					lng = EXCLUDED.lng,
					event_type = EXCLUDED.event_type,
					user_created = EXCLUDED.user_created,
					hard_start = EXCLUDED.hard_start,
					hard_end = EXCLUDED.hard_end,
					timezone = EXCLUDED.timezone,
					wheelchair_accessible_parking = EXCLUDED.wheelchair_accessible_parking,
					wheelchair_accessible_entrance = EXCLUDED.wheelchair_accessible_entrance,
					wheelchair_accessible_restroom = EXCLUDED.wheelchair_accessible_restroom,
					wheelchair_accessible_seating = EXCLUDED.wheelchair_accessible_seating,
					serves_vegetarian_food = EXCLUDED.serves_vegetarian_food,
					price_level = EXCLUDED.price_level,
					utc_offset_minutes = EXCLUDED.utc_offset_minutes,
					website_uri = EXCLUDED.website_uri,
					types = EXCLUDED.types,
					photo_name = EXCLUDED.photo_name,
					photo_width = EXCLUDED.photo_width,
					photo_height = EXCLUDED.photo_height,
					photo_author = EXCLUDED.photo_author,
					photo_author_uri = EXCLUDED.photo_author_uri,
					photo_author_photo_uri = EXCLUDED.photo_author_photo_uri,
					weekday_descriptions = EXCLUDED.weekday_descriptions,
					secondary_hours_type = EXCLUDED.secondary_hours_type,
					next_open_time = EXCLUDED.next_open_time,
					next_close_time = EXCLUDED.next_close_time,
					open_now = EXCLUDED.open_now,
					periods = EXCLUDED.periods,
					special_days = EXCLUDED.special_days
				RETURNING id
				"#,
				&ev.event_name,
				ev.event_description.as_ref(),
				ev.street_address.as_ref(),
				ev.city.as_ref(),
				ev.country.as_ref(),
				ev.postal_code,
				ev.lat,
				ev.lng,
				ev.event_type.as_ref(),
				ev.user_created,
				ev.hard_start,
				ev.hard_end,
				ev.timezone.as_ref(),
				ev.place_id.as_ref(),
				ev.wheelchair_accessible_parking,
				ev.wheelchair_accessible_entrance,
				ev.wheelchair_accessible_restroom,
				ev.wheelchair_accessible_seating,
				ev.serves_vegetarian_food,
				ev.price_level,
				ev.utc_offset_minutes,
				ev.website_uri.as_ref(),
				ev.types.as_ref(),
				ev.photo_name.as_ref(),
				ev.photo_width,
				ev.photo_height,
				ev.photo_author.as_ref(),
				ev.photo_author_uri.as_ref(),
				ev.photo_author_photo_uri.as_ref(),
				ev.weekday_descriptions.as_ref(),
				ev.secondary_hours_type,
				ev.next_open_time,
				ev.next_close_time,
				ev.open_now,
				&ev.periods as _,
				&ev.special_days as _,
			)
			.fetch_one(&self.pool)
			.await?;
			ev.id = inserted.id;
			results.push(ev);
		}

		// Extract event IDs and names for the response and debugging
		results.sort_unstable_by(|a, b| a.id.cmp(&b.id));
		results.dedup_by(|a, b| a.id == b.id);

		// get user preferences
		// TODO use real account id
		let account_id = 3;
		let preferences = sqlx::query!(
			r#"
			SELECT
				budget_preference as "budget_preference: BudgetBucket",
				risk_preference as "risk_preference: RiskTolerence",
				food_allergies,
				disabilities
			FROM accounts where id=$1;
			"#,
			account_id
		)
		.fetch_one(&self.pool)
		.await
		.map_err(|e| format!("Failed to fetch user preferences from database - {e}"))?;
		let preferences = json!({
			"budget_preference": preferences.budget_preference,
			"risk_preference": preferences.risk_preference,
			"food_allergies": preferences.food_allergies,
			"disabilities": preferences.disabilities
		}).to_string();

		// Invoke llm to generate itinerary
		let response = self
			.llm
			.invoke(
				format!(
					include_str!("../agent/prompts/fallback_gen.md"),
					start_date,
					end_date,
					preferences,
					context,
					include_str!("../agent/prompts/fallback_itinerary.ts"),
					json!(results)
				)
				.as_str(),
			)
			.await?;

		// TODO we can check if all the fields are provided in the object
		Ok(response
			.trim_start_matches("```json")
			.trim_end_matches("```")
			.trim()
			.to_string())
	}
}

async fn nearby_search(
	lat: f64,
	lng: f64,
	radius: f64,
	gm_client: &Client,
) -> Result<Vec<Event>, String> {
	// TODO fill out include/exclude arrays
	let included_types = [PlaceType::Museum];
	let excluded_types = [PlaceType::LocalGovernmentOffice];
	let search_res = gm_client
		.nearby_search((lat, lng, radius))
		.map_err(|e| format!("Could not create nearby search call - {e}"))?
		.field_mask(FieldMask::Specific(vec![
			Field::PlacesAccessibilityOptions,
			Field::PlacesAdrFormatAddress,
			Field::PlacesDisplayName,
			Field::PlacesId,
			Field::PlacesPhotos,
			Field::PlacesUtcOffsetMinutes,
			Field::PlacesPriceLevel,
			Field::PlacesRegularOpeningHours,
			Field::PlacesWebsiteUri,
			Field::PlacesServesVegetarianFood,
			Field::PlacesTypes,
			Field::PlacesPrimaryType,
			Field::PlacesEditorialSummary,
		]))
		.included_types(included_types)
		.excluded_types(excluded_types)
		.execute()
		.await
		.map_err(|e| format!("Error performing nearby search - {e}"))?;

	if let Some(err) = search_res.error() {
		return Err(format!("Nearby Search failed - {err}"));
	}
	let places = search_res.places();
	if places.is_empty() {
		return Err(format!("Nearby Search returned an empty array of places"));
	}

	Ok(places
		.into_iter()
		.map(|p| Event::from(p))
		.collect::<Vec<_>>())
}
