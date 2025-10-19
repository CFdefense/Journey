use axum::{routing::{get, post}, Extension, Json, Router};
use sqlx::PgPool;
use crate::{error::{ApiResult, AppError}, global::MESSAGE_PAGE_LEN, http_models::{chat_session::{ChatsResponse, NewChatResponse}, message::{Message, MessagePageRequest, MessagePageResponse, SendMessageRequest, SendMessageResponse, UpdateMessageRequest}}, middleware::{middleware_auth, AuthUser}, sql_models::message::MessageRow};

pub async fn api_chats(
	Extension(user): Extension<AuthUser>,
    Extension(pool): Extension<PgPool>
) -> ApiResult<Json<ChatsResponse>> {
	Ok(Json(ChatsResponse { chat_sessions:
		sqlx::query!(
			r#"
			SELECT id from chat_sessions
			WHERE account_id=$1;
			"#,
			user.id
		)
		.fetch_all(&pool)
		.await
		.map_err(|e| AppError::from(e))?
		.into_iter()
		.map(|record| record.id)
		.collect()
	}))
}

pub async fn api_message_page(
	Extension(user): Extension<AuthUser>,
    Extension(pool): Extension<PgPool>,
    Json(MessagePageRequest {chat_session_id, message_id}): Json<MessagePageRequest>
) -> ApiResult<Json<MessagePageResponse>> {
	let mut message_page: Vec<Message> = sqlx::query_as!(
		MessageRow,
		r#"
		SELECT
			m.id,
			m.chat_session_id,
			m.itinerary_id,
			m.is_user,
			m.timestamp,
			m.text
		FROM messages m
		INNER JOIN chat_sessions c
		ON m.chat_session_id=c.id
		WHERE
			c.id=$1 AND
			c.account_id=$2 AND
			(
				$3::int IS NULL OR
				m.timestamp <= (SELECT timestamp FROM messages WHERE id=$3)
			)
		ORDER BY m.timestamp DESC
		LIMIT $4 + 1;
		"#,
		chat_session_id,
		user.id,
		message_id,
		MESSAGE_PAGE_LEN
	)
	.fetch_all(&pool)
	.await
	.map_err(|e| AppError::from(e))?
	.into_iter()
	.rev()
	.map(|msg_row| Message {
		id: msg_row.id,
		is_user: msg_row.is_user,
		timestamp: msg_row.timestamp,
		text: msg_row.text,
		itinerary_id: msg_row.itinerary_id
	})
	.collect();

	let prev_message_id = if message_page.len() == MESSAGE_PAGE_LEN as usize + 1 {
		// there might be a better way to do this, but it should work, and it's only O(MESSAGE_PAGE_LEN) time complexity
		Some(message_page.remove(0).id)
	} else {
		None
	};

	Ok(Json(MessagePageResponse {
		message_page,
		prev_message_id
	}))
}

pub async fn api_update_message(
	Extension(user): Extension<AuthUser>,
    Extension(pool): Extension<PgPool>,
    Json(UpdateMessageRequest {message_id, new_text}): Json<UpdateMessageRequest>
) -> ApiResult<Json<Message>> {
	if new_text.is_empty() {return Err(AppError::BadRequest(String::from("Text cannot be empty")))}

	// verify the message id belongs to this user
	sqlx::query!(
		r#"
		SELECT m.id
		FROM messages m
		INNER JOIN chat_sessions c
		ON m.chat_session_id=c.id
		WHERE m.id=$1 AND c.account_id=$2;
		"#,
		message_id,
		user.id
	)
	.fetch_optional(&pool)
	.await
	.map_err(|e| AppError::from(e))?
	.ok_or(AppError::NotFound)?;

	let Some(timestamp) = sqlx::query!(
		r#"
		UPDATE messages
		SET text=$1, timestamp=NOW()
		WHERE is_user=TRUE AND id=$2
		RETURNING timestamp;
		"#,
		new_text,
		message_id
	)
	.fetch_optional(&pool)
	.await
	.map_err(|e| AppError::from(e))?
	.map(|record| record.timestamp) else {
		// We could first check to see if the message exists, and give a 404 if not, then do 400 if is_user is false,
		// but this is easier for now.
		return Err(AppError::BadRequest(String::from("Invalid message id")))
	};
	// TODO: delete temporary data and actually implement controller
	Ok(Json(Message {
		id: message_id,
		is_user: true,
		timestamp,
		text: new_text,
		itinerary_id: None
	}))
}

pub async fn api_send_message(
	Extension(user): Extension<AuthUser>,
    Extension(pool): Extension<PgPool>,
    Json(SendMessageRequest {chat_session_id, text}): Json<SendMessageRequest>
) -> ApiResult<Json<SendMessageResponse>> {
	if text.is_empty() {return Err(AppError::BadRequest(String::from("Text cannot be empty")))}

	// verify the given chat session belongs to this user
	sqlx::query!(
		r#"
		SELECT id from chat_sessions
		WHERE id=$1 AND account_id=$2;
		"#,
		user.id,
		chat_session_id
	)
	.fetch_optional(&pool)
	.await
	.map_err(|e| AppError::from(e))?
	.ok_or(AppError::NotFound)?;

	let user_message_id = sqlx::query!(
		r#"
		INSERT INTO messages (chat_session_id, itinerary_id, is_user, timestamp, text)
		VALUES ($1, NULL, TRUE, NOW(), $2)
		RETURNING id;
		"#,
		chat_session_id,
		text
	)
	.fetch_one(&pool)
	.await
	.map_err(|e| AppError::from(e))?
	.id;

	// TODO: this is where the LLM call will be.
	// It should generate an itinerary, insert it into the db, and give some text.
	// Fow now we just make a temporary message.
	let ai_text = "Bot reply";
	let itinerary_id: Option<i32> = None;

	let record = sqlx::query!(
		r#"
		INSERT INTO messages (chat_session_id, itinerary_id, is_user, timestamp, text)
		VALUES ($1, $2, FALSE, NOW(), $3)
		RETURNING id, timestamp;
		"#,
		chat_session_id,
		itinerary_id,
		ai_text
	)
	.fetch_one(&pool)
	.await
	.map_err(|e| AppError::from(e))?;

	let (bot_message_id, timestamp) = (record.id, record.timestamp);

	Ok(Json(SendMessageResponse {
		user_message_id,
		bot_message: Message {
			id: bot_message_id,
			is_user: false,
			timestamp: timestamp,
			text: String::from(ai_text),
			itinerary_id
		}
	}))
}

pub async fn api_new_chat(
	Extension(user): Extension<AuthUser>,
    Extension(pool): Extension<PgPool>
) -> ApiResult<Json<NewChatResponse>> {
	// check to see if there's already an empty chat session before making a new one
	let chat_sessions = sqlx::query!(
		r#"
		SELECT c.id
		FROM chat_sessions c
		WHERE
			c.account_id=$1
			AND NOT EXISTS (
				SELECT 1
				FROM messages m
				WHERE m.chat_session_id=c.id
			);
		"#,
		user.id
	)
	.fetch_all(&pool)
	.await
	.map_err(|e| AppError::from(e))?;

	let chat_session_id = match chat_sessions.first() {
		Some(record) => record.id,
		None => {
			sqlx::query!(
				r#"
				INSERT INTO chat_sessions (account_id)
				VALUES ($1)
				RETURNING id
				"#,
				user.id
			)
			.fetch_one(&pool)
			.await
			.map_err(|e| AppError::from(e))?
			.id
		}
	};

	Ok(Json(NewChatResponse { chat_session_id }))
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
        .route("/messagePage", post(api_message_page))
        .route("/updateMessage", post(api_update_message))
        .route("/sendMessage", post(api_send_message))
        .route("/newChat", get(api_new_chat))
        .route_layer(axum::middleware::from_fn(middleware_auth))
}