/*
 * src/controllers/itinerary.rs
 *
 * File for Itinerary Controller API Endpoints
 *
 * Purpose:
 *   Serve Itinerary Related API Requests
 *
 * Include:
 *   api_saved_itineraries  - GET /api/itinerary/saved -> returns user's saved itineraries
 */

use axum::{Extension, Json, Router, routing::{get, post}};
use sqlx::PgPool;
use tracing::info;

use crate::error::{ApiResult, AppError};
use crate::middleware::{AuthUser, middleware_auth};
use crate::models::itinerary::*;

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
/// - `200 OK` - JSON body `{ "itineraries": [Itinerary] }` containing user's saved itineraries
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X GET http://localhost:3000/api/itinerary/saved
///   -H "Cookie: auth-token=..."
/// ```
///
pub async fn api_saved_itineraries(
    Extension(user): Extension<AuthUser>,
    Extension(pool): Extension<PgPool>,
) -> ApiResult<Json<SavedResponse>> {
    info!(
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

/// Create the itinerary routes with authentication middleware.
///
/// # Routes
/// - `GET /saved` - Get user's saved itineraries (protected)
///
/// # Middleware
/// All routes are protected by `middleware_auth` which validates the `auth-token` cookie.
///
pub fn itinerary_routes() -> Router {
    Router::new()
        .route("/saved", get(api_saved_itineraries))
        .route_layer(axum::middleware::from_fn(middleware_auth))
}