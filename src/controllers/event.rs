/*
 * src/controllers/event.rs
 *
 * File for Event Controller API Endpoints
 *
 * Purpose:
 *   Serve Event Related API Requests
 */

use axum::{extract::Path, Extension, Json, Router, routing::{get, put}};
use sqlx::PgPool;
use tracing::debug;

use crate::{error::{ApiResult, AppError}, http_models::event::Event, sql_models::event::EventRow};
use crate::middleware::{AuthUser, middleware_auth};
use crate::http_models::event::{UpdateEventRequest};

/// Get a single event by ID.
///
/// # Method
/// `GET /api/event/{id}`
///
/// # Auth
/// Requires authentication
///
/// # Responses
/// - `200 OK` - JSON body [Event] containing event details
/// - `404 NOT_FOUND` - When event doesn't exist
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X GET http://localhost:3001/api/event/123
/// ```
///
pub async fn api_get_event(
    Extension(user): Extension<AuthUser>,
    Path(event_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> ApiResult<Json<Event>> {
    debug!(
        "HANDLER ->> /api/event/{} 'api_get_event' - User ID: {}",
        event_id, user.id
    );

    // Fetch the event
    let event: EventRow = sqlx::query_as!(
        EventRow,
        r#"
        SELECT
        	id,
         	street_address,
          	postal_code,
           	city,
            event_type,
            event_description,
            event_name
        FROM events
        WHERE id = $1
        "#,
        event_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::from(e))?
    .ok_or(AppError::NotFound)?;

    Ok(Json(event))
}

/// Update a single event by ID.
///
/// # Method
/// `PUT /api/event/{id}`
///
/// # Auth
/// Requires authentication
///
/// # Responses
/// - `200 OK` - JSON body `Event` containing updated event details
/// - `404 NOT_FOUND` - When event doesn't exist
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X PUT http://localhost:3001/api/event/123 \
///   -H "Content-Type: application/json" \
///   -d '{"event_name": "Updated Event Name"}'
/// ```
///
pub async fn api_update_event(
    Extension(user): Extension<AuthUser>,
    Path(event_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<UpdateEventRequest>,
) -> ApiResult<Json<Event>> {
    debug!(
        "HANDLER ->> /api/event/{} 'api_update_event' - User ID: {}",
        event_id, user.id
    );

    // Execute the update query with coalesced values
    let updated_event: EventRow = sqlx::query_as!(
        EventRow,
        r#"UPDATE events SET
            street_address = COALESCE($1, street_address),
            postal_code = COALESCE($2, postal_code),
            city = COALESCE($3, city),
            event_type = COALESCE($4, event_type),
            event_description = COALESCE($5, event_description),
            event_name = COALESCE($6, event_name)
        WHERE id = $7
        RETURNING id, street_address, postal_code, city, event_type, event_description, event_name"#,
        payload.street_address,
        payload.postal_code,
        payload.city,
        payload.event_type as Option<String>,
        payload.event_description,
        payload.event_name,
        event_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::from(e))?
    .ok_or(AppError::NotFound)?;

    Ok(Json(updated_event))
}

/// Create the event routes.
///
/// # Routes
/// - `GET /{id}` - Get individual event details (requires auth)
/// - `PUT /{id}` - Update individual event details (requires auth)
///
/// # Middleware
/// All routes require authentication.
///
pub fn event_routes() -> Router {
    Router::new()
        .route("/:id", get(api_get_event))
        .route("/:id", put(api_update_event))
        .route_layer(axum::middleware::from_fn(middleware_auth))
}