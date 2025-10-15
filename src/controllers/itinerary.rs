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
use crate::middleware::{AuthUser, middleware_auth};
use crate::models::itinerary::*;
use crate::models::event::Event;

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
    let itineraries: Vec<Itinerary> = sqlx::query_as!(
        Itinerary,
        r#"SELECT id, account_id, is_public, date FROM itineraries WHERE account_id = $1"#,
        user.id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::from(e))?;

    Ok(Json(SavedResponse { itineraries }))
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
) -> ApiResult<Json<ItineraryResponse>> {
    debug!(
        "HANDLER ->> /api/itinerary/{} 'api_get_itinerary' - User ID: {}",
        itinerary_id, user.id
    );

    // Fetch the itinerary for the user
    let itinerary: Itinerary = sqlx::query_as!(
        Itinerary,
        r#"SELECT id, account_id, is_public, date FROM itineraries WHERE id = $1 AND account_id = $2"#,
        itinerary_id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::from(e))?
    .ok_or(AppError::NotFound)?;

    Ok(Json(ItineraryResponse { itinerary }))
}

/// Get events for a specific itinerary.
///
/// # Method
/// `GET /api/itinerary/{id}/events`
///
/// # Auth
/// Protected by `auth_middleware` which validates the `auth-token` private cookie,
/// checks expiration, and injects `Extension<AuthUser>`.
///
/// # Responses
/// - `200 OK` - JSON body `Vec<Event>` containing events for the itinerary
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
/// - `404 NOT_FOUND` - When itinerary doesn't exist or doesn't belong to user
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X GET http://localhost:3001/api/itinerary/123/events
///   -H "Cookie: auth-token=..."
/// ```
///
pub async fn api_get_itinerary_events(
    Extension(user): Extension<AuthUser>,
    Path(itinerary_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> ApiResult<Json<Vec<Event>>> {
    debug!(
        "HANDLER ->> /api/itinerary/{}/events 'api_get_itinerary_events' - User ID: {}",
        itinerary_id, user.id
    );

    // Verify itinerary belongs to user
    let _itinerary: Itinerary = sqlx::query_as!(
        Itinerary,
        r#"SELECT id, account_id, is_public, date FROM itineraries WHERE id = $1 AND account_id = $2"#,
        itinerary_id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::from(e))?
    .ok_or(AppError::NotFound)?;

    // Fetch events for this itinerary by joining event_list with events
    let events: Vec<Event> = sqlx::query_as::<_, Event>(
        r#"SELECT e.id, e.street_address, e.postal_code, e.city, e.event_type, e.event_description, e.event_name
           FROM events e
           INNER JOIN event_list el ON e.id = el.event_id
           WHERE el.itinerary_id = $1"#,
    )
    .bind(itinerary_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::from(e))?;

    Ok(Json(events))
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
        .route("/:id/events", get(api_get_itinerary_events))
        .route_layer(axum::middleware::from_fn(middleware_auth))
}