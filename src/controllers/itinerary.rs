/*
 * src/controllers/itinerary.rs
 *
 * File for Itinerary Controller API Endpoints
 *
 * Purpose:
 *   Serve Itinerary Related API Requests
 */

use axum::routing::post;
use axum::{extract::Path, routing::get, Extension, Json};
use sqlx::PgPool;
use tracing::debug;

use crate::controllers::AxumRouter;
use crate::error::{ApiResult, AppError};
use crate::middleware::{AuthUser, middleware_auth};
use crate::http_models::itinerary::*;
use crate::sql_models::event_list::EventListJoinRow;
use crate::sql_models::itinerary::ItineraryRow;
use crate::sql_models::TimeOfDay;

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
			e.event_type,
			e.event_description,
			e.event_name
		FROM event_list el
		JOIN events e ON e.id = el.event_id
		WHERE el.itinerary_id = $1
		"#,
		itinerary_id
	)
	.fetch_all(pool)
	.await
	.map_err(|e| AppError::from(e))?;

	let mut event_days = Vec::with_capacity(event_list.len());
	for event_day in event_list.chunk_by(|a,b| a.date == b.date) {
		let mut morning_events = Vec::with_capacity(event_list.len());
		let mut noon_events = Vec::with_capacity(event_list.len());
		let mut afternoon_events = Vec::with_capacity(event_list.len());
		let mut evening_events = Vec::with_capacity(event_list.len());

		for event in event_day.into_iter() {
			match event.time_of_day {
			    TimeOfDay::Morning => morning_events.push(event.into()),
			    TimeOfDay::Noon => noon_events.push(event.into()),
			    TimeOfDay::Afternoon => afternoon_events.push(event.into()),
			    TimeOfDay::Evening => evening_events.push(event.into()),
			}
		}

		if let Some(event) = event_day.first() {
			event_days.push(EventDay {
			    morning_events,
			    noon_events,
			    afternoon_events,
			    evening_events,
			    date: event.date
			});
		}
	}
	event_days.sort_by(|a,b| a.date.cmp(&b.date));

	Ok(event_days)
}

/// Inserts the events associated with this itinerary into the `event_list` table.
/// Assumes the itinerary was already inserted into `itineraries` table.
pub async fn insert_event_list(itinerary: Itinerary, pool: &PgPool) -> ApiResult<()> {
	let mut cap = 0;
	for day in itinerary.event_days.iter() {
		cap += day.morning_events.len();
		cap += day.noon_events.len();
		cap += day.afternoon_events.len();
		cap += day.evening_events.len();
	}

	let mut times = Vec::with_capacity(cap);
	let mut dates = Vec::with_capacity(cap);
	let mut events = Vec::with_capacity(cap);
	for day in itinerary.event_days.into_iter() {
		let morning_len = day.morning_events.len();
		let noon_len = day.noon_events.len();
		let afternoon_len = day.afternoon_events.len();
		let evening_len = day.evening_events.len();
		let len = morning_len + noon_len + afternoon_len + evening_len;

		times.extend(std::iter::repeat_n(TimeOfDay::Morning, morning_len));
		times.extend(std::iter::repeat_n(TimeOfDay::Noon, noon_len));
		times.extend(std::iter::repeat_n(TimeOfDay::Afternoon, afternoon_len));
		times.extend(std::iter::repeat_n(TimeOfDay::Evening, evening_len));

		dates.extend(std::iter::repeat_n(day.date, len));

		events.extend(day.morning_events.into_iter().map(|event| event.id));
		events.extend(day.noon_events.into_iter().map(|event| event.id));
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
    .map_err(|e| AppError::from(e))?;

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
    .map_err(|e| AppError::from(e))?;

    let mut res = Vec::with_capacity(itineraries.len());
    for itinerary in itineraries.into_iter() {
		res.push(Itinerary {
			id: itinerary.id,
		    start_date: itinerary.start_date,
		    end_date: itinerary.end_date,
		    event_days: itinerary_events(itinerary.id, &pool).await?,
			chat_session_id: itinerary.chat_session_id,
			title: itinerary.title
		});
    }

    Ok(Json(SavedResponse { itineraries: res }))
}

/// Get a single saved itinerary for the authenticated user.
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
pub async fn api_get_itinerary(
    Extension(user): Extension<AuthUser>,
    Path(itinerary_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> ApiResult<Json<Itinerary>> {
    debug!(
        "HANDLER ->> /api/itinerary/{} 'api_get_itinerary' - User ID: {}",
        itinerary_id, user.id
    );

    // Fetch the itinerary for the user
    let itinerary: ItineraryRow = sqlx::query_as!(
        ItineraryRow,
        r#"SELECT
        	id,
         	account_id,
          	start_date,
           	end_date,
            chat_session_id,
            title
        FROM itineraries WHERE id = $1 AND account_id = $2"#,
        itinerary_id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::from(e))?
    .ok_or(AppError::NotFound)?;

    Ok(Json(Itinerary {
    	id: itinerary.id,
	    start_date: itinerary.start_date,
	    end_date: itinerary.end_date,
	    event_days: itinerary_events(itinerary_id, &pool).await?,
		chat_session_id: itinerary.chat_session_id,
		title: itinerary.title
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
///             "noon_events": [
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
///             "afternoon_events": [],
///             "evening_events": [],
///             "date": "2025-07-21"
///           }
///         ],
///         "chat_session_id": 4,
///         "title": "Poughkeepsie 7/15-21 2025"
///       }'
/// ```
pub async fn api_save(
	Extension(user): Extension<AuthUser>,
    Extension(pool): Extension<PgPool>,
    Json(itinerary): Json<Itinerary>
) -> ApiResult<Json<SaveResponse>> {
	// check if itinerary id already exists for this user
	let id_opt = sqlx::query!(
        r#"SELECT id FROM itineraries WHERE id=$1 AND account_id=$2"#,
        itinerary.id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::from(e))?
    .map(|record| record.id);

	// if it doesn't exist, insert a new one
	let id = match id_opt {
		Some(id) => id,
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
			.map_err(|e| AppError::from(e))?
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
	.map_err(|e| AppError::from(e))?;

	insert_event_list(itinerary, &pool).await?;

	Ok(Json(SaveResponse {id}))
}

/// Create the itinerary routes with authentication middleware.
///
/// # Routes
/// - `GET /saved` - Get user's saved itineraries (protected)
/// - `POST /save` - Inserts into or updates the user's itinerary in the db (protected)
/// - `GET /{id}` - Get single itinerary metadata (protected)
///
/// # Middleware
/// All routes are protected by `middleware_auth` which validates the `auth-token` cookie.
pub fn itinerary_routes() -> AxumRouter {
    AxumRouter::new()
        .route("/saved", get(api_saved_itineraries))
        .route("/save", post(api_save))
        .route("/:id", get(api_get_itinerary))
        .route_layer(axum::middleware::from_fn(middleware_auth))
}