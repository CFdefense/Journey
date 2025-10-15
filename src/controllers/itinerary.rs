/*
 * src/controllers/itinerary.rs
 *
 * File for Itinerary Controller API Endpoints
 *
 * Purpose:
 *   Serve Itinerary Related API Requests
 */

use axum::{extract::Path, routing::get, Extension, Json, Router};
use sqlx::PgPool;
use tracing::debug;

use crate::error::{ApiResult, AppError};
use crate::http_models::event::Event;
use crate::middleware::{AuthUser, middleware_auth};
use crate::http_models::itinerary::*;
use crate::sql_models::event_list::EventListJoinRow;
use crate::sql_models::itinerary::ItineraryJoinedRow;
use crate::sql_models::TimeOfDay;

async fn itinerary_events(itinerary_id: i32, pool: &PgPool) -> ApiResult<Vec<EventListJoinRow>> {
	sqlx::query_as!(
		EventListJoinRow,
		r#"
		SELECT
			el.time_of_day,
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
	.fetch_all(&pool)
	.await
	.map_err(|e| AppError::from(e))
}

fn as_events(el: &[EventListJoinRow], tod: TimeOfDay) -> Vec<Event> {
	el.iter()
  		.filter_map(|e| {
      		if e.time_of_day == tod {
         		Some(e.into())
      		} else {
				None
			}
 		})
		.collect()
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
        r#"SELECT id, account_id, date FROM itineraries WHERE account_id = $1"#,
        user.id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::from(e))?;

    let res = Vec::with_capacity(itineraries.len());

    for itinerary in itineraries.iter() {
    	let event_list = itinerary_events(itinerary.id, &pool).await?;

		res.push(Itinerary {
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
        r#"SELECT id, account_id, date FROM itineraries WHERE id = $1 AND account_id = $2"#,
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
		    el.time_of_day,
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
	    date: itinerary.date,
	    morning_events: as_events(event_list.as_slice(), TimeOfDay::Morning),
	    noon_events: as_events(event_list.as_slice(), TimeOfDay::Noon),
	    afternoon_events: as_events(event_list.as_slice(), TimeOfDay::Afternoon),
	    evening_events: as_events(event_list.as_slice(), TimeOfDay::Evening)
	}))
}

/// Create the itinerary routes with authentication middleware.
///
/// # Routes
/// - `GET /saved` - Get user's saved itineraries (protected)
/// - `GET /{id}` - Get single itinerary metadata (protected)
/// - `GET /{id}/events` - Get events for itinerary (protected)
///
/// # Middleware
/// All routes are protected by `middleware_auth` which validates the `auth-token` cookie.
///
pub fn itinerary_routes() -> Router {
    Router::new()
        .route("/saved", get(api_saved_itineraries))
        .route("/:id", get(api_get_itinerary))
        .route_layer(axum::middleware::from_fn(middleware_auth))
}