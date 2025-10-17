use axum::{extract::{Path, Query}, routing::{get, post}, Extension, Json, Router};
use chrono::NaiveDateTime;
use sqlx::PgPool;
use crate::{error::ApiResult, http_models::{chat_session::{ChatSession, ChatsResponse}, message::{Message, MessagePageResponse, SendMessageResponse, UpdateMessageRequest}}, middleware::{middleware_auth, AuthUser}};

pub async fn api_chats(
	Extension(user): Extension<AuthUser>,
    Extension(pool): Extension<PgPool>
) -> ApiResult<Json<ChatsResponse>> {
	// TODO: delete temporary data and actually implement controller
	Ok(Json(ChatsResponse {
	    chat_sessions: vec![
			ChatSession {
				id: 1,
				account_id: user.id
			},
			ChatSession {
				id: 2,
				account_id: user.id
			},
			ChatSession {
				id: 3,
				account_id: user.id
			}
		]
	}))
}

pub async fn api_message_page(
	Extension(user): Extension<AuthUser>,
    Path(chat_session_id): Path<i32>,
    message_id: Option<Query<i32>>,
    Extension(pool): Extension<PgPool>
) -> ApiResult<Json<MessagePageResponse>> {
	// TODO: delete temporary data and actually implement controller
	Ok(Json(MessagePageResponse {
		message_page: vec![
			Message {
				id: 1,
				is_user: true,
				timestamp: NaiveDateTime::parse_from_str("2025-10-17 20:56:04", "%Y-%m-%d %H:%M:%S").unwrap(),
				text: String::from("Make me an itinerary"),
				itinerary_id: None
			},
			Message {
				id: 2,
				is_user: false,
				timestamp: NaiveDateTime::parse_from_str("2025-10-17 20:56:06", "%Y-%m-%d %H:%M:%S").unwrap(),
				text: String::from("No"),
				itinerary_id: None
			},
			Message {
				id: 3,
				is_user: true,
				timestamp: NaiveDateTime::parse_from_str("2025-10-17 20:56:10", "%Y-%m-%d %H:%M:%S").unwrap(),
				text: String::from("Please?"),
				itinerary_id: None
			},
			Message {
				id: 4,
				is_user: false,
				timestamp: NaiveDateTime::parse_from_str("2025-10-17 20:56:15", "%Y-%m-%d %H:%M:%S").unwrap(),
				text: String::from("Ok"),
				itinerary_id: Some(1)
			}
		],
		prev_message_id: None
	}))
}

pub async fn api_update_message(
	Extension(user): Extension<AuthUser>,
    Extension(pool): Extension<PgPool>,
    Json(UpdateMessageRequest {message_id, new_text}): Json<UpdateMessageRequest>
) -> ApiResult<Json<Message>> {
	// TODO: delete temporary data and actually implement controller
	Ok(Json(Message {
		id: 2,
		is_user: true,
		timestamp: NaiveDateTime::parse_from_str("2025-10-17 20:56:20", "%Y-%m-%d %H:%M:%S").unwrap(),
		text: new_text,
		itinerary_id: None
	}))
}

pub async fn api_send_message(
	Extension(user): Extension<AuthUser>,
    Extension(pool): Extension<PgPool>,
    Query(text): Query<String>,
) -> ApiResult<Json<SendMessageResponse>> {
	// TODO: delete temporary data and actually implement controller
	Ok(Json(SendMessageResponse {
		user_message_id: 1,
		bot_message: Message {
			id: 5,
			is_user: false,
			timestamp: NaiveDateTime::parse_from_str("2025-10-17 20:56:25", "%Y-%m-%d %H:%M:%S").unwrap(),
			text: String::from("Bot reply"),
			itinerary_id: None
		}
	}))
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
        .route("/chats", get(api_chats))
        .route("/messagePage", get(api_message_page))
        .route("/updateMessage", post(api_update_message))
        .route("/sendMessage", get(api_send_message))
        .route_layer(axum::middleware::from_fn(middleware_auth))
}