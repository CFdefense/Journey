use axum::{Extension, Json, Router, routing::{get, post}};
use sqlx::PgPool;
use tracing::info;

use crate::error::{ApiResult, AppError};
use crate::middleware::{AuthUser, middleware_auth};
use crate::models::itinerary::*;


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

pub fn itinerary_routes() -> Router {
    Router::new()
        .route("/saved", get(api_saved_itineraries))
        .route_layer(axum::middleware::from_fn(middleware_auth))
}