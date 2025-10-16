use axum::{extract::{Path, Query}, routing::{get, post}, Extension, Json, Router};
use sqlx::PgPool;
use crate::{error::ApiResult, http_models::{chat_session::ChatsResponse, message::{Message, MessagePageResponse, SaveMessageResponse, UpdateMessageRequest}}, middleware::{middleware_auth, AuthUser}};

pub async fn api_chats(
	Extension(user): Extension<AuthUser>,
    Extension(pool): Extension<PgPool>
) -> ApiResult<Json<ChatsResponse>> {
	todo!()
}

pub async fn api_message_page(
	Extension(user): Extension<AuthUser>,
    Path(chat_session_id): Path<i32>,
    message_id: Option<Query<i32>>,
    Extension(pool): Extension<PgPool>
) -> ApiResult<Json<MessagePageResponse>> {
	todo!()
}

pub async fn api_update_message(
	Extension(user): Extension<AuthUser>,
    Extension(pool): Extension<PgPool>,
    Json(UpdateMessageRequest {message_id, new_text}): Json<UpdateMessageRequest>
) -> ApiResult<Json<Message>> {
	todo!()
}

pub async fn api_send_message(
	Extension(user): Extension<AuthUser>,
    Extension(pool): Extension<PgPool>,
    Query(text): Query<String>,
) -> ApiResult<Json<SaveMessageResponse>> {
	todo!()
}

/// Create the chat routes with authentication middleware.
///
/// # Routes
/// - `GET /chats` - Get metadata for all the user's chat sessions (protected)
/// - `GET /messagePage/{chat_session_id}?message_id=[Option<i32>]` - Gets a page of messages in the session, ending with message_id or the latest message (protected)
/// - `POST /updateMessage` - Updates a user's message and waits for a bot reply (protected)
/// - `GET /sendMessage?text=[String]` - Sends a user's message and waits for a bot reply (protected)
///
/// # Middleware
/// All routes are protected by `middleware_auth` which validates the `auth-token` cookie.
///
pub fn chat_routes() -> Router {
    Router::new()
        // .route("/chats", get(api_chats))
        // .route("/messagePage", get(api_message_page))
        // .route("/updateMessage", post(api_update_message))
        // .route("/sendMessage", get(api_send_message))
        .route_layer(axum::middleware::from_fn(middleware_auth))
}