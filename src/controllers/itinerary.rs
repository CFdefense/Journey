/*
 * src/controllers/itinerary.rs
 *
 * File for Itinerary Controller API Endpoints
 *
 * Purpose:
 *   Serve Itinerary Related API Requests
 */

use axum::routing::post;
use axum::{extract::Path, routing::get, Extension, Json, Router};
use sqlx::PgPool;
use tracing::debug;

use crate::error::{ApiResult, AppError};
use crate::middleware::{AuthUser, middleware_auth};
use crate::http_models::itinerary::*;
use crate::sql_models::event::EventRow;
use crate::sql_models::event_list::EventListJoinRow;
use crate::sql_models::itinerary::ItineraryJoinedRow;
use crate::sql_models::TimeOfDay;

/// Returns the [EventListJoinRow]s associated with this itinerary
async fn itinerary_events(itinerary_id: i32, pool: &PgPool) -> ApiResult<Vec<EventListJoinRow>> {
	sqlx::query_as!(
		EventListJoinRow,
		r#"
		SELECT
			e.id,
			el.time_of_day as "time_of_day: TimeOfDay",
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
	.map_err(|e| AppError::from(e))
}

/// Filter-maps the slice of [EventListJoinRow]s to a Vec of [Event]s
fn as_events(el: &[EventListJoinRow], tod: TimeOfDay) -> Vec<EventRow> {
	let mut events = Vec::with_capacity(el.len());
	for e in el.iter() {
		if e.time_of_day == tod {
			events.push(e.into());
		}
	}
	events
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
    let itineraries: Vec<ItineraryJoinedRow> = sqlx::query_as!(
        ItineraryJoinedRow,
        r#"SELECT id, account_id, date, chat_session_id FROM itineraries WHERE account_id = $1"#,
        user.id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::from(e))?;

    let mut res = Vec::with_capacity(itineraries.len());

    for itinerary in itineraries.iter() {
    	let event_list = itinerary_events(itinerary.id, &pool).await?;

		res.push(Itinerary {
			id: itinerary.id,
		    date: itinerary.date,
		    morning_events: as_events(event_list.as_slice(), TimeOfDay::Morning),
		    noon_events: as_events(event_list.as_slice(), TimeOfDay::Noon),
		    afternoon_events: as_events(event_list.as_slice(), TimeOfDay::Afternoon),
		    evening_events: as_events(event_list.as_slice(), TimeOfDay::Evening)
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
///
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
    let itinerary: ItineraryJoinedRow = sqlx::query_as!(
        ItineraryJoinedRow,
        r#"SELECT id, account_id, date, chat_session_id FROM itineraries WHERE id = $1 AND account_id = $2"#,
        itinerary_id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::from(e))?
    .ok_or(AppError::NotFound)?;

    let event_list: Vec<EventListJoinRow> = sqlx::query_as!(
        EventListJoinRow,
        r#"
		SELECT
			e.id,
		    el.time_of_day as "time_of_day: TimeOfDay",
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
        itinerary.id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::from(e))?;

    Ok(Json(Itinerary {
    	id: itinerary.id,
	    date: itinerary.date,
	    morning_events: as_events(event_list.as_slice(), TimeOfDay::Morning),
	    noon_events: as_events(event_list.as_slice(), TimeOfDay::Noon),
	    afternoon_events: as_events(event_list.as_slice(), TimeOfDay::Afternoon),
	    evening_events: as_events(event_list.as_slice(), TimeOfDay::Evening)
	}))
}

pub async fn api_save(
	Extension(user): Extension<AuthUser>,
    Extension(pool): Extension<PgPool>,
    Json(itinerary): Json<Itinerary>
) -> ApiResult<SaveResponse> {
	// TODO: delete temporary data and actually implement controller
	Ok(SaveResponse {id: 1})
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
#[cfg(test)]
pub fn itinerary_routes() -> Router {
    Router::new()
        .route("/saved", get(api_saved_itineraries))
        .route("/:id", get(api_get_itinerary))
        .route_layer(axum::middleware::from_fn(middleware_auth))
}
#[cfg(not(test))]
pub fn itinerary_routes() -> Router {
    Router::new()
        .route("/saved", get(api_saved_itineraries))
        .route("/save", post(api_save))
        .route("/:id", get(api_get_itinerary))
        .route_layer(axum::middleware::from_fn(middleware_auth))
}