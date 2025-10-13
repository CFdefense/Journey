/*
 * src/controllers/event.rs
 *
 * File for Event Controller API Endpoints
 *
 * Purpose:
 *   Serve Event Related API Requests
 *
 * Include:
 *   api_get_event  - GET /api/event/{id} -> returns individual event details
 */

use axum::{extract::Path, Extension, Json, Router, routing::get};
use sqlx::PgPool;
use tracing::info;

use crate::error::{ApiResult, AppError};
use crate::middleware::{AuthUser, middleware_auth};
use crate::models::event::Event;

/// Get a single event by ID.
///
/// # Method
/// `GET /api/event/{id}`
///
/// # Auth
/// Public endpoint - no authentication required
///
/// # Responses
/// - `200 OK` - JSON body `Event` containing event details
/// - `404 NOT_FOUND` - When event doesn't exist
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X GET http://localhost:3000/api/event/123
/// ```
///
pub async fn api_get_event(
    Path(event_id): Path<i32>,
    Extension(user): Extension<AuthUser>,
    Extension(pool): Extension<PgPool>,
) -> ApiResult<Json<Event>> {
    info!(
        "HANDLER ->> /api/event/{} 'api_get_event' - User ID: {}",
        event_id, user.id
    );

    // Fetch the event
    let event: Event = sqlx::query_as::<_, Event>(
        r#"SELECT id, street_address, postal_code, city, event_type, event_description, event_name FROM events WHERE id = $1"#,
    )
    .bind(event_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::from(e))?
    .ok_or(AppError::NotFound)?;

    Ok(Json(event))
}

/// Create the event routes.
///
/// # Routes
/// - `GET /{id}` - Get individual event details (public)
///
/// # Middleware
/// No authentication required for event details.
///
pub fn event_routes() -> Router {
    Router::new()
        .route("/:id", get(api_get_event))
        .route_layer(axum::middleware::from_fn(middleware_auth))
}