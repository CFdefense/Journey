/*
 * src/controllers/itinerary.rs
 *
 * File for Itinerary Controller API Endpoints
 *
 * Purpose:
 *   Serve Itinerary Related API Requests
 */

use axum::routing::{delete, post};
use axum::{Extension, Json, extract::Path, routing::get};
use sqlx::PgPool;
use tracing::debug;
use utoipa::OpenApi;

use crate::controllers::AxumRouter;
use crate::error::{ApiResult, AppError};
use crate::global::EVENT_SEARCH_RESULT_LEN;
use crate::http_models::event::{
	SearchEventRequest, SearchEventResponse, UserEventRequest, UserEventResponse,
};
use crate::http_models::itinerary::*;
use crate::middleware::{AuthUser, middleware_auth};
use crate::sql_models::TimeOfDay;
use crate::sql_models::event_list::EventListJoinRow;
use crate::sql_models::itinerary::ItineraryRow;
use crate::swagger::SecurityAddon;

#[derive(OpenApi)]
#[openapi(
	paths(
		api_get_itinerary,
		api_saved_itineraries,
		api_save,
		api_unsave,
		api_user_event,
		api_search_event,
		api_delete_user_event
	),
	modifiers(&SecurityAddon),
	security(("set-cookie"=[])),
    info(
    	title="Itinerary Routes",
    	description = "API endpoints dealing with managing and viewing itineraries."
    ),
    tags((name="Itinerary"))
)]
pub struct ItineraryApiDoc;

/// Returns the [EventDay]s associated with this itinerary
async fn itinerary_events(itinerary_id: i32, pool: &PgPool) -> ApiResult<Vec<EventDay>> {
	let event_list: Vec<EventListJoinRow> = sqlx::query_as!(
		EventListJoinRow,
		r#"
		SELECT
			e.id,
			el.time_of_day as "time_of_day: TimeOfDay",
			el.date,
			e.street_address,
			e.postal_code,
			e.city,
			e.country,
			e.event_type,
			e.event_description,
			e.event_name,
			e.user_created,
			e.hard_start,
			e.hard_end
		FROM event_list el
		JOIN events e ON e.id = el.event_id
		WHERE el.itinerary_id = $1
		"#,
		itinerary_id
	)
	.fetch_all(pool)
	.await
	.map_err(AppError::from)?;

	let mut event_days = Vec::with_capacity(event_list.len());
	for event_day in event_list.chunk_by(|a, b| a.date == b.date) {
		let mut morning_events = Vec::with_capacity(event_list.len());
		let mut afternoon_events = Vec::with_capacity(event_list.len());
		let mut evening_events = Vec::with_capacity(event_list.len());

		for event in event_day.into_iter() {
			match event.time_of_day {
				TimeOfDay::Morning => morning_events.push(event.into()),
				TimeOfDay::Afternoon => afternoon_events.push(event.into()),
				TimeOfDay::Evening => evening_events.push(event.into()),
			}
		}

		if let Some(event) = event_day.first() {
			event_days.push(EventDay {
				morning_events,
				afternoon_events,
				evening_events,
				date: event.date,
			});
		}
	}
	event_days.sort_by(|a, b| a.date.cmp(&b.date));

	Ok(event_days)
}

/// Inserts the events associated with this itinerary into the `event_list` table.
/// Assumes the itinerary was already inserted into `itineraries` table.
pub async fn insert_event_list(itinerary: Itinerary, pool: &PgPool) -> ApiResult<()> {
	let mut cap = 0;
	for day in itinerary.event_days.iter() {
		cap += day.morning_events.len();
		cap += day.afternoon_events.len();
		cap += day.evening_events.len();
	}

	let mut times = Vec::with_capacity(cap);
	let mut dates = Vec::with_capacity(cap);
	let mut events = Vec::with_capacity(cap);
	for day in itinerary.event_days.into_iter() {
		let morning_len = day.morning_events.len();
		let afternoon_len = day.afternoon_events.len();
		let evening_len = day.evening_events.len();
		let len = morning_len + afternoon_len + evening_len;

		times.extend(std::iter::repeat_n(TimeOfDay::Morning, morning_len));
		times.extend(std::iter::repeat_n(TimeOfDay::Afternoon, afternoon_len));
		times.extend(std::iter::repeat_n(TimeOfDay::Evening, evening_len));

		dates.extend(std::iter::repeat_n(day.date, len));

		events.extend(day.morning_events.into_iter().map(|event| event.id));
		events.extend(day.afternoon_events.into_iter().map(|event| event.id));
		events.extend(day.evening_events.into_iter().map(|event| event.id));
	}

	sqlx::query!(
		r#"
		INSERT INTO event_list (itinerary_id, event_id, time_of_day, date)
		SELECT $1, events, times, dates
		FROM UNNEST($2::int4[], $3::time_of_day[], $4::date[]) as u(events, times, dates);
		"#,
		itinerary.id,
		events.as_slice(),
		times.as_slice() as &[TimeOfDay],
		dates.as_slice()
	)
	.execute(pool)
	.await
	.map_err(AppError::from)?;

	Ok(())
}

/// Get all saved itineraries for the authenticated user.
///
/// # Method
/// `GET /api/itinerary/saved`
///
/// # Auth
/// Protected by `auth_middleware` which validates the `auth-token` private cookie,
/// checks expiration, and injects `Extension<AuthUser>`.
///
/// # Responses
/// - `200 OK` - JSON body `{ "itineraries": [Itinerary] }` containing user's saved itineraries with eventlist
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X GET http://localhost:3001/api/itinerary/saved
///   -H "Cookie: auth-token=..."
/// ```
///
#[utoipa::path(
	get,
	path="/saved",
	summary="Fetch all the saved itineraries from this user",
	description="Fetches all the itineraries from this user that are marked as saved.",
	responses(
		(
			status=200,
			description="An array of itineraries",
			body=SavedResponse,
			content_type="application/json",
			//TODO example
		),
		(status=400, description="Bad Request"),
		(status=401, description="User has an invalid cookie/no cookie"),
		(status=405, description="Method Not Allowed - Must be GET"),
		(status=408, description="Request Timed Out"),
		(status=500, description="Internal Server Error")
	),
	security(("set-cookie"=[])),
	tag="Itinerary"
)]
pub async fn api_saved_itineraries(
	Extension(user): Extension<AuthUser>,
	Extension(pool): Extension<PgPool>,
) -> ApiResult<Json<SavedResponse>> {
	debug!(
		"HANDLER ->> /api/itinerary/saved 'api_saved_itineraries' - User ID: {}",
		user.id
	);

	// Fetch all itineraries for the user
	let itineraries: Vec<ItineraryRow> = sqlx::query_as!(
		ItineraryRow,
		r#"SELECT
        	id,
         	account_id,
          	start_date,
           	end_date,
            chat_session_id,
            title
        FROM itineraries WHERE account_id=$1 AND saved=TRUE"#,
		user.id
	)
	.fetch_all(&pool)
	.await
	.map_err(AppError::from)?;

	let mut res = Vec::with_capacity(itineraries.len());
	for itinerary in itineraries.into_iter() {
		res.push(Itinerary {
			id: itinerary.id,
			start_date: itinerary.start_date,
			end_date: itinerary.end_date,
			event_days: itinerary_events(itinerary.id, &pool).await?,
			chat_session_id: itinerary.chat_session_id,
			title: itinerary.title,
		});
	}

	Ok(Json(SavedResponse { itineraries: res }))
}

/// Get a single saved itinerary either from the user or a public one
///
/// # Method
/// `GET /api/itinerary/{id}`
///
/// # Auth
/// Protected by `auth_middleware` which validates the `auth-token` private cookie,
/// checks expiration, and injects `Extension<AuthUser>`.
///
/// # Responses
/// - `200 OK` - JSON body `{ "itinerary": Itinerary }` containing itinerary metadata
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
/// - `404 NOT_FOUND` - When itinerary doesn't exist or doesn't belong to user
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X GET http://localhost:3001/api/itinerary/123
///   -H "Cookie: auth-token=..."
/// ```
#[utoipa::path(
	get,
	path="/saved/{id}",
	summary="Fetch a specific itinerary",
	description="Fetches the specified itinerary if it belongs to this user or is public.",
	responses(
		(
			status=200,
			description="The desired itinerary",
			body=Itinerary,
			content_type="application/json",
			//TODO example
		),
		(status=400, description="Bad Request"),
		(status=401, description="User has an invalid cookie/no cookie"),
		(status=404, description="Itinerary not found"),
		(status=405, description="Method Not Allowed - Must be GET"),
		(status=408, description="Request Timed Out"),
		(status=500, description="Internal Server Error")
	),
	security(("set-cookie"=[])),
	tag="Itinerary"
)]
pub async fn api_get_itinerary(
	Extension(user): Extension<AuthUser>,
	Path(itinerary_id): Path<i32>,
	Extension(pool): Extension<PgPool>,
) -> ApiResult<Json<Itinerary>> {
	debug!(
		"HANDLER ->> /api/itinerary/{} 'api_get_itinerary' - User ID: {}",
		itinerary_id, user.id
	);

	// Fetch the itinerary - from user or public
	let itinerary: ItineraryRow = sqlx::query_as!(
		ItineraryRow,
		r#"SELECT
        	id,
         	account_id,
          	start_date,
           	end_date,
            chat_session_id,
            title
        FROM itineraries WHERE id = $1 AND (account_id = $2 OR is_public=TRUE)"#,
		itinerary_id,
		user.id
	)
	.fetch_optional(&pool)
	.await
	.map_err(AppError::from)?
	.ok_or(AppError::NotFound)?;

	Ok(Json(Itinerary {
		id: itinerary.id,
		start_date: itinerary.start_date,
		end_date: itinerary.end_date,
		event_days: itinerary_events(itinerary_id, &pool).await?,
		chat_session_id: itinerary.chat_session_id,
		title: itinerary.title,
	}))
}

/// Update an existing or save a new itinerary for the user
///
/// # Method
/// `POST /api/itinerary/save`
///
/// # Request Body
/// - [Itinerary]
///
/// # Responses
/// - `200 OK` - with body: [SaveResponse]
/// - `400 BAD_REQUEST` - Request payload contains invalid data (public error)
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X POST http://localhost:3001/api/itinerary/save
///   -H "Content-Type: application/json"
///   -d '{
///         "id": 3,
///         "start_date": "2025-07-15",
///         "end_date": "2025-07-21",
///         "event_days": [
///           {
///             "morning_events": [],
///             "afternoon_events": [
///               {
///                 "id": 4,
///                 "street_address": "3399 North Rd",
///                 "postal_code": 12601,
///                 "city": "Poughkeepsie",
///                 "event_type": "Park",
///                 "event_description": "Take a tour of Marist University",
///                 "event_name": "Marist University"
///               }
///             ],
///             "evening_events": [],
///             "date": "2025-07-21"
///           }
///         ],
///         "chat_session_id": 4,
///         "title": "Poughkeepsie 7/15-21 2025"
///       }'
/// ```
#[utoipa::path(
	post,
	path="/save",
	summary="Save a new or update an existing itinerary",
	description="If the itinerary id is already saved for this user, it's updated with the provided values. Otherwise a new one is created.",
	request_body(
		content=Itinerary,
		content_type="application/json",
		description="The itinerary to save for the user.",
		//TODO example
	),
	responses(
		(
			status=200,
			description="The id of the itinerary that was just saved. It may be the same as the id passed in the request.",
			body=SaveResponse,
			content_type="application/json",
			//TODO example
		),
		(status=400, description="Bad Request"),
		(status=401, description="User has an invalid cookie/no cookie"),
		(status=405, description="Method Not Allowed - Must be POST"),
		(status=408, description="Request Timed Out"),
		(status=500, description="Internal Server Error")
	),
	security(("set-cookie"=[])),
	tag="Itinerary"
)]
pub async fn api_save(
	Extension(user): Extension<AuthUser>,
	Extension(pool): Extension<PgPool>,
	Json(itinerary): Json<Itinerary>,
) -> ApiResult<Json<SaveResponse>> {
	// check if itinerary id already exists for this user
	let id_opt = sqlx::query!(
		r#"SELECT id FROM itineraries WHERE id=$1 AND account_id=$2"#,
		itinerary.id,
		user.id
	)
	.fetch_optional(&pool)
	.await
	.map_err(AppError::from)?
	.map(|record| record.id);

	// if it doesn't exist, insert a new one
	let id = match id_opt {
		Some(id) => {
			// UPDATE existing itinerary and set saved=TRUE
			sqlx::query!(
				r#"
				UPDATE itineraries
				SET start_date = $1, end_date = $2, title = $3, chat_session_id = $4, saved = TRUE
				WHERE id = $5 AND account_id = $6;
				"#,
				itinerary.start_date,
				itinerary.end_date,
				itinerary.title,
				itinerary.chat_session_id,
				id,
				user.id
			)
			.execute(&pool)
			.await
			.map_err(AppError::from)?;

			id
		}
		None => {
			sqlx::query!(
				r#"
				INSERT INTO itineraries (account_id, is_public, start_date, end_date, chat_session_id, saved, title)
				VALUES ($1, FALSE, $2, $3, $4, TRUE, $5)
				RETURNING id;
				"#,
				user.id,
				itinerary.start_date,
				itinerary.end_date,
				itinerary.chat_session_id,
				itinerary.title
			)
			.fetch_one(&pool)
			.await
			.map_err(AppError::from)?
			.id
		}
	};

	// delete event_list for this itinerary and make a new one
	sqlx::query!(
		r#"
		DELETE FROM event_list
		WHERE itinerary_id=$1;
		"#,
		id
	)
	.execute(&pool)
	.await
	.map_err(AppError::from)?;

	insert_event_list(itinerary, &pool).await?;

	Ok(Json(SaveResponse { id }))
}

/// Unsave an existing itinerary for the user
///
/// # Method
/// `POST /api/itinerary/unsave`
///
/// # Request Body
/// - [UnsaveRequest]
///
/// # Responses
/// - `200 OK` - Successfully unsaved itinerary for this user
/// - `400 BAD_REQUEST` - Request payload contains invalid data (public error)
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
/// - `404 NOT_FOUND` - Itinerary not found or doesn't belong to user (public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X POST http://localhost:3001/api/itinerary/unsave
///   -H "Content-Type: application/json"
///   -d '{
///         "id": 3
///       }'
/// ```
#[utoipa::path(
	post,
	path="/unsave",
	summary="Unsave an existing itinerary",
	description="Sets the saved field to false for the given itinerary. Verifies the itinerary belongs to the user.",
	request_body(
		content=UnsaveRequest,
		content_type="application/json",
		description="The itinerary id to unsave."
	),
	responses(
		(status=200, description="Successfully unsaved itinerary"),
		(status=400, description="Bad Request"),
		(status=401, description="User has an invalid cookie/no cookie"),
		(status=404, description="Itinerary not found or doesn't belong to user"),
		(status=405, description="Method Not Allowed - Must be POST"),
		(status=408, description="Request Timed Out"),
		(status=500, description="Internal Server Error")
	),
	security(("set-cookie"=[])),
	tag="Itinerary"
)]
pub async fn api_unsave(
	Extension(user): Extension<AuthUser>,
	Extension(pool): Extension<PgPool>,
	Json(UnsaveRequest{ id }): Json<UnsaveRequest>,
) -> ApiResult<()> {
	// Update the itinerary to set saved=FALSE
	sqlx::query!(
		r#"
		UPDATE itineraries
		SET saved = FALSE
		WHERE id = $1 AND account_id = $2
		RETURNING id;
		"#,
		id,
		user.id
	)
	.fetch_optional(&pool)
	.await
	.map_err(AppError::from)?
	.ok_or(AppError::NotFound)?;

	Ok(())
}


/// Insert or update a user-created custom event
///
/// # Method
/// `POST /api/itinerary/userEvent`
///
/// # Request Body
/// - [UserEventRequest]
///
/// # Responses
/// - `200 OK` - with body: [UserEventResponse] - event id that was just inserted or updated
/// - `400 BAD_REQUEST` - Request payload contains invalid data (public error)
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
/// - `404 NOT_FOUND` - The provided event id does not belong to the user or does not exist (public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X POST http://localhost:3001/api/itinerary/userEvent
///   -H "Content-Type: application/json"
///   -d '{
///         "event_name": "Custom Event",
///	        "event_description": "I want to do something and it's easier to make a custom event than to tell the LLM exactly how I want it."
///       }'
/// ```
#[utoipa::path(
	post,
	path="/userEvent",
	summary="Insert or update a user-created custom event",
	description="Insert a new or updates an existing user-created event with the values passed in the request, returning the event id.",
	request_body(
		content=UserEventRequest,
		content_type="application/json",
		description="If id is provided, the event will be updated, otherwise it is inserted. The event name is required.",
		example=json!({
			"event_name": "Custom Event",
			"event_description": "I want to do something and it's easier to make a custom event than to tell the LLM exactly how I want it."
		})
	),
	responses(
		(
			status=200,
			description="Contains the id of the event that was just inserted or updated.",
			body=UserEventResponse,
			content_type="application/json",
			example=json!({
				"id": 43
			})
		),
		(status=400, description="Bad Request"),
		(status=401, description="User has an invalid cookie/no cookie"),
		(status=404, description="User-event not found for this user"),
		(status=405, description="Method Not Allowed - Must be POST"),
		(status=408, description="Request Timed Out"),
		(status=500, description="Internal Server Error")
	),
	security(("set-cookie"=[])),
	tag="Itinerary"
)]
pub async fn api_user_event(
	Extension(user): Extension<AuthUser>,
	Extension(pool): Extension<PgPool>,
	Json(event): Json<UserEventRequest>,
) -> ApiResult<Json<UserEventResponse>> {
	if event.event_name.is_empty() {
		return Err(AppError::BadRequest(String::from(
			"Event name must not be empty",
		)));
	}
	let id = if let Some(id) = event.id {
		sqlx::query!(
			r#"
			UPDATE events
			SET
				street_address    = COALESCE($1, street_address),
				postal_code       = COALESCE($2, postal_code),
				city              = COALESCE($3, city),
				country           = COALESCE($4, city),
				event_type        = COALESCE($5, event_type),
				event_description = COALESCE($6, event_description),
				event_name        = $7,
				user_created      = TRUE,
				account_id        = $8,
				hard_start        = COALESCE($9, hard_start),
				hard_end          = COALESCE($10, hard_end)
			WHERE id=$11 AND user_created=TRUE AND account_id=$8
			RETURNING id
			"#,
			event.street_address,
			event.postal_code,
			event.city,
			event.country,
			event.event_type,
			event.event_description,
			event.event_name,
			user.id,
			event.hard_start,
			event.hard_end,
			id
		)
		.fetch_optional(&pool)
		.await
		.map_err(AppError::from)?
		.ok_or(AppError::NotFound)?;
		id
	} else {
		sqlx::query!(
			r#"
			INSERT INTO events(
				street_address, postal_code, city, country,
				event_type, event_description, event_name,
				user_created, account_id, hard_start, hard_end
			)
			VALUES($1, $2, $3, $4, $5, $6, $7, TRUE, $8, $9, $10)
			RETURNING id
			"#,
			event.street_address,
			event.postal_code,
			event.city,
			event.country,
			event.event_type,
			event.event_description,
			event.event_name,
			user.id,
			event.hard_start,
			event.hard_end,
		)
		.fetch_one(&pool)
		.await
		.map_err(AppError::from)?
		.id
	};
	Ok(Json(UserEventResponse { id }))
}

/// Searches for events that match the filter and returns a list of possible events
///
/// # Method
/// `POST /api/itinerary/searchEvent`
///
/// # Request Body
/// - [SearchEventRequest]
///   - Example filters:
///     - `event_name`: Partial name of the event (case-insensitive)
///     - `city`: Partial city name
///     - `event_type`: Type of event
///     - `hard_start_after`: ISO 8601 timestamp to filter events starting after this time
///     - `hard_start_before`: ISO 8601 timestamp to filter events starting before this time
///
/// # Responses
/// - `200 OK` - with body: [SearchEventResponse] - the best matching events for the query
/// - `400 BAD_REQUEST` - Request payload contains invalid data (public error)
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Example Request
/// ```bash
/// curl -X POST http://localhost:3001/api/itinerary/searchEvent \
///   -H "Content-Type: application/json" \
///   -d '{
///         "event_name": "Music Festival",
///         "city": "New York",
///         "event_type": "Concert",
///         "hard_start_after": "2025-11-01T00:00:00",
///         "hard_start_before": "2025-11-30T23:59:59"
///       }'
/// ```
///
/// # Example Response
/// ```json
/// {
///   "events": [
///     {
///       "id": 1,
///       "street_address": "123 Main St",
///       "postal_code": 10001,
///       "city": "New York",
///       "event_type": "Concert",
///       "event_description": "Annual music festival in the park",
///       "event_name": "NY Music Festival",
///       "user_created": false,
///       "account_id": 2,
///       "hard_start": "2025-11-15T18:00:00",
///       "hard_end": "2025-11-15T23:00:00"
///     },
///     {
///       "id": 2,
///       "street_address": "456 Broadway",
///       "postal_code": 10002,
///       "city": "New York",
///       "event_type": "Concert",
///       "event_description": "Indie music showcase",
///       "event_name": "Indie Night",
///       "user_created": true,
///       "account_id": 3,
///       "hard_start": "2025-11-20T19:00:00",
///       "hard_end": "2025-11-20T22:00:00"
///     }
///   ]
/// }
/// ```
#[utoipa::path(
    post,
    path="/searchEvent",
    summary="Search for events with the given filters and return a list of the best matching events",
    description="Returns a limited number of events that best match the filters provided in the request.",
    request_body(
        content=SearchEventRequest,
        content_type="application/json",
        description="Uses the filters, if provided, to search for the best matching events.",
        example=json!({
            "event_name": "Music Festival",
            "city": "New York",
            "event_type": "Concert",
            "hard_start_after": "2025-11-01T00:00:00",
            "hard_start_before": "2025-11-30T23:59:59"
        })
    ),
    responses(
        (
            status=200,
            description="A list of the best matching events for the given filters.",
            body=SearchEventResponse,
            content_type="application/json",
            example=json!({
                "events": [
                    {
                        "id": 1,
                        "street_address": "123 Main St",
                        "postal_code": 10001,
                        "city": "New York",
                        "event_type": "Concert",
                        "event_description": "Annual music festival in the park",
                        "event_name": "NY Music Festival",
                        "user_created": false,
                        "account_id": 2,
                        "hard_start": "2025-11-15T18:00:00",
                        "hard_end": "2025-11-15T23:00:00"
                    },
                    {
                        "id": 2,
                        "street_address": "456 Broadway",
                        "postal_code": 10002,
                        "city": "New York",
                        "event_type": "Concert",
                        "event_description": "Indie music showcase",
                        "event_name": "Indie Night",
                        "user_created": true,
                        "account_id": 3,
                        "hard_start": "2025-11-20T19:00:00",
                        "hard_end": "2025-11-20T22:00:00"
                    }
                ]
            })
        ),
        (status=400, description="Bad Request"),
        (status=401, description="User has an invalid cookie/no cookie"),
        (status=405, description="Method Not Allowed - Must be POST"),
        (status=408, description="Request Timed Out"),
        (status=500, description="Internal Server Error")
    ),
    security(("set-cookie"=[])),
    tag="Itinerary"
)]
pub async fn api_search_event(
	Extension(user): Extension<AuthUser>,
	Extension(pool): Extension<PgPool>,
	Json(query): Json<SearchEventRequest>,
) -> ApiResult<Json<SearchEventResponse>> {
	let mut qb =
		sqlx::QueryBuilder::new("SELECT * FROM events WHERE (user_created=FALSE OR account_id=");
	qb.push_bind(user.id).push(")");
	// Dynamically add filters if present
	if let Some(id) = query.id {
		qb.push(" AND id = ").push_bind(id);
	}
	if let Some(street_address) = query.street_address {
		qb.push(" AND street_address ILIKE ")
			.push_bind(format!("%{}%", street_address));
	}
	if let Some(postal_code) = query.postal_code {
		qb.push(" AND postal_code = ").push_bind(postal_code);
	}
	if let Some(city) = query.city {
		qb.push(" AND city ILIKE ").push_bind(format!("%{}%", city));
	}
	if let Some(event_type) = query.event_type {
		qb.push(" AND event_type ILIKE ")
			.push_bind(format!("%{}%", event_type));
	}
	if let Some(event_description) = query.event_description {
		qb.push(" AND event_description ILIKE ")
			.push_bind(format!("%{}%", event_description));
	}
	if let Some(event_name) = query.event_name {
		qb.push(" AND event_name ILIKE ")
			.push_bind(format!("%{}%", event_name));
	}
	if let Some(hard_start_before) = query.hard_start_before {
		qb.push(" AND hard_start < ").push_bind(hard_start_before);
	}
	if let Some(hard_start_after) = query.hard_start_after {
		qb.push(" AND hard_start > ").push_bind(hard_start_after);
	}
	if let Some(hard_end_before) = query.hard_end_before {
		qb.push(" AND hard_end < ").push_bind(hard_end_before);
	}
	if let Some(hard_end_after) = query.hard_end_after {
		qb.push(" AND hard_end > ").push_bind(hard_end_after);
	}
	qb.push(" ORDER BY hard_start ASC LIMIT ")
		.push_bind(EVENT_SEARCH_RESULT_LEN);
	Ok(Json(SearchEventResponse {
		events: qb.build_query_as().fetch_all(&pool).await?,
	}))
}

/// Deletes a user-created event from the db
///
/// # Method
/// `DELETE /api/itinerary/userEvent/:id`
///
/// # Responses
/// - `200 OK` - User-created event deleted successfully
/// - `400 BAD_REQUEST` - Request payload contains invalid data (public error)
/// - `404 NOT_FOUND` - User-created event was not found or does not belong to this user (public error)
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Example Request
/// ```bash
/// curl -X DELETE http://localhost:3001/api/itinerary/userEvent/:14 \
///   -H "Content-Type: application/json" \
/// ```
#[utoipa::path(
    delete,
    path="/userEvent/{id}",
    summary="Deletes a user-event from the DB",
    description="Deletes the user-created event from the DB using the provided event ID. Event must have been created by this user.",
    responses(
        (status=200, description="User-created event successfully deleted"),
        (status=400, description="Bad Request"),
        (status=401, description="User has an invalid cookie/no cookie"),
        (status=404, description="User-event not found or does not belong to this user"),
        (status=405, description="Method Not Allowed - Must be POST"),
        (status=408, description="Request Timed Out"),
        (status=500, description="Internal Server Error")
    ),
    security(("set-cookie"=[])),
    tag="Itinerary"
)]
pub async fn api_delete_user_event(
	Extension(user): Extension<AuthUser>,
	Extension(pool): Extension<PgPool>,
	Path(event_id): Path<i32>,
) -> ApiResult<()> {
	sqlx::query!(
		r#"
		DELETE FROM events
		WHERE
			id=$1 AND
			account_id=$2 AND
			user_created=TRUE
		RETURNING id;
		"#,
		event_id,
		user.id
	)
	.fetch_optional(&pool)
	.await
	.map_err(AppError::from)?
	.ok_or(AppError::NotFound)?;
	Ok(())
}

/// Create the itinerary routes with authentication middleware.
///
/// # Routes
/// - `GET /saved` - Get user's saved itineraries (protected)
/// - `POST /save` - Inserts into or updates the user's itinerary in the db (protected)
/// - `GET /{id}` - Get single itinerary metadata (protected)
/// - `POST /userEvent` - Insert or update a user-created custom event (protected)
/// - `POST /searchEvent` - queries the DB for an event that matches the provided filters (protected)
/// - `DELETE /userEvent/{id}` - Deletes the user-created event from the db (protected)
///
/// # Middleware
/// All routes are protected by `middleware_auth` which validates the `auth-token` cookie.
pub fn itinerary_routes() -> AxumRouter {
	AxumRouter::new()
		.route("/saved", get(api_saved_itineraries))
		.route("/save", post(api_save))
		.route("/unsave", post(api_unsave))
		.route("/{id}", get(api_get_itinerary))
		.route("/userEvent", post(api_user_event))
		.route("/searchEvent", post(api_search_event))
		.route("/userEvent/{id}", delete(api_delete_user_event))
		.route_layer(axum::middleware::from_fn(middleware_auth))
}
