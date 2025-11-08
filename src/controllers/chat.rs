use axum::{
	Extension, Json,
	extract::Path,
	routing::{delete, get, post},
};
use chrono::NaiveDate;
use sqlx::PgPool;
use std::env;
use utoipa::OpenApi;

use crate::{
	agent::config::AgentType,
	controllers::{AxumRouter, itinerary::insert_event_list},
	error::{ApiResult, AppError},
	global::MESSAGE_PAGE_LEN,
	http_models::{
		chat_session::{ChatsResponse, NewChatResponse, RenameRequest},
		event::Event,
		itinerary::{EventDay, Itinerary},
		message::{
			Message, MessagePageRequest, MessagePageResponse, SendMessageRequest,
			SendMessageResponse, UpdateMessageRequest,
		},
	},
	middleware::{AuthUser, middleware_auth},
	sql_models::message::{ChatSessionRow, MessageRow},
	swagger::SecurityAddon,
};
use langchain_rust::{chain::Chain, prompt_args};

#[derive(OpenApi)]
#[openapi(
	paths(
		api_chats,
		api_new_chat,
		api_message_page,
		api_send_message,
		api_update_message,
		api_delete_chat,
		api_rename
	),
	modifiers(&SecurityAddon),
	security(("set-cookie"=[])),
    info(
    	title="Chat Routes",
    	description = "API endpoints dealing with chatting and the home page."
    ),
    tags((name="Chat"))
)]
pub struct ChatApiDoc;

/// Sends message and latest itinerary in chat session to llm, and waits for response.
///
/// When the bot replies, it's message and itinerary are inserted into the db.
/// # Warning!
/// Assumes the user's message has already been inserted into the db.
async fn send_message_to_llm(
	text: &str,
	account_id: i32,
	chat_session_id: i32,
	itinerary_id: Option<i32>,
	pool: &PgPool,
	agent: &AgentType,
) -> ApiResult<Message> {
	// Give the LLM an itinerary for context
	let itinerary_id = match itinerary_id {
		Some(id) => Some(id), //use the provided itinerary
		None => {
			//use the latest itinerary from the chat session
			sqlx::query!(
				r#"
				SELECT m.itinerary_id
				FROM messages m
				INNER JOIN chat_sessions c
				ON m.chat_session_id=c.id
				WHERE
					c.account_id=$1 AND
					c.id=$2 AND
					m.itinerary_id IS NOT NULL
				ORDER BY m.timestamp DESC
				LIMIT 1;
				"#,
				account_id,
				chat_session_id
			)
			.fetch_optional(pool)
			.await
			.map_err(AppError::from)?
			.map(|record| record.itinerary_id.unwrap())
		}
	};
	let _context_itinerary = match itinerary_id {
		Some(id) => Some(
			crate::controllers::itinerary::api_get_itinerary(
				Extension(AuthUser { id: account_id }),
				axum::extract::Path(id),
				Extension(pool.clone()),
			)
			.await?,
		),
		None => None,
	};

	// Check if DEPLOY_LLM == "1", return dummy response if not
	let ai_text = match env::var("DEPLOY_LLM") {
		#[cfg(not(tarpaulin_include))]
		Ok(s) if s.as_str() == "1" => {
			// DEPLOY_LLM is set, call the actual AI agent
			let agent_guard = agent.lock().await;
			agent_guard
				.invoke(prompt_args! {
					"input" => text,
				})
				.await
				.map_err(|e| AppError::Internal(format!("AI agent error: {}", e)))?
		}
		_ => {
			// DEPLOY_LLM != "1", return dummy response
			format!(
				"This is a dummy response for testing. You said: \"{}\"",
				text
			)
		}
	};

	// Create dummy itinerary (always used when DEPLOY_LLM != "1")
	let mut ai_itinerary = Itinerary {
		id: 0,
		start_date: NaiveDate::parse_from_str("2025-11-05", "%Y-%m-%d").unwrap(),
		end_date: NaiveDate::parse_from_str("2025-11-06", "%Y-%m-%d").unwrap(),
		event_days: vec![
			EventDay {
				morning_events: vec![Event {
					id: 1,
					street_address: String::from("1114 Shannon Ln"),
					postal_code: 17013,
					city: String::from("Carlisle"),
					event_type: String::from("Hike"),
					event_description: String::from(
						"A beautiful stroll along a river in this cute small town.",
					),
					event_name: String::from("Family Walking Path"),
					user_created: false,
					account_id: None,
					hard_start: None,
					hard_end: None,
				}],
				noon_events: vec![Event {
					id: 2,
					street_address: String::from("35 Campus Court"),
					postal_code: 12601,
					city: String::from("Poughkeepsie"),
					event_type: String::from("Restaurant"),
					event_description: String::from(
						"Local Italian restaurant known for its authentic pasta and upscale dining.",
					),
					event_name: String::from("Cosimos"),
					user_created: false,
					account_id: None,
					hard_start: None,
					hard_end: None,
				}],
				afternoon_events: vec![Event {
					id: 3,
					street_address: String::from("200 E 42nd St"),
					postal_code: 10017,
					city: String::from("New York"),
					event_type: String::from("Museum"),
					event_description: String::from(
						"World famous art museum with a focus on modern works, including Starry Starry Night by VanGough.",
					),
					event_name: String::from("Museum of Modern Art- MoMA"),
					user_created: false,
					account_id: None,
					hard_start: None,
					hard_end: None,
				}],
				evening_events: vec![Event {
					id: 4,
					street_address: String::from("1 S Broad St"),
					postal_code: 19107,
					city: String::from("Philadelphia"),
					event_type: String::from("Concert"),
					event_description: String::from(
						"Music center which hosts local and national bands.",
					),
					event_name: String::from("Jazz night at Broad Street"),
					user_created: false,
					account_id: None,
					hard_start: None,
					hard_end: None,
				}],
				date: NaiveDate::parse_from_str("2025-11-05", "%Y-%m-%d").unwrap(),
			},
			EventDay {
				morning_events: vec![Event {
					id: 5,
					street_address: String::from("1 Citizens Bank Way"),
					postal_code: 19148,
					city: String::from("Philadelphia"),
					event_type: String::from("Sports"),
					event_description: String::from(
						"A Phillies baseball game is a must-do for locals and visitors alike.",
					),
					event_name: String::from("Phillies Baseball Game"),
					user_created: false,
					account_id: None,
					hard_start: None,
					hard_end: None,
				}],
				noon_events: vec![Event {
					id: 6,
					street_address: String::from("5250 S Park Dr"),
					postal_code: 60615,
					city: String::from("Chicago"),
					event_type: String::from("Festival"),
					event_description: String::from(
						"Annual music festival with the biggest names in pop and indie scenes.",
					),
					event_name: String::from("LollaPalooza"),
					user_created: false,
					account_id: None,
					hard_start: None,
					hard_end: None,
				}],
				afternoon_events: vec![Event {
					id: 7,
					street_address: String::from("1 Rue de la Seine"),
					postal_code: 0,
					city: String::from("Paris"),
					event_type: String::from("Museum"),
					event_description: String::from("Explore the beautiful landmark of Paris."),
					event_name: String::from("Eiffel Tower"),
					user_created: false,
					account_id: None,
					hard_start: None,
					hard_end: None,
				}],
				evening_events: vec![Event {
					id: 8,
					street_address: String::from("3 Rue de la Museu"),
					postal_code: 0,
					city: String::from("Paris"),
					event_type: String::from("Museum"),
					event_description: String::from(
						"Wander the halls of the world famous art museum.",
					),
					event_name: String::from("le Louvre"),
					user_created: false,
					account_id: None,
					hard_start: None,
					hard_end: None,
				}],
				date: NaiveDate::parse_from_str("2025-11-06", "%Y-%m-%d").unwrap(),
			},
		],
		chat_session_id: None,
		title: String::from("World Tour 11/5-15 2025"),
	};

	// Insert generated itinerary into db
	let inserted_itinerary_id = sqlx::query!(
		r#"
		INSERT INTO itineraries (account_id, is_public, start_date, end_date, chat_session_id, saved, title)
		VALUES ($1, FALSE, $2, $3, $4, TRUE, $5)
		RETURNING id;
		"#,
		account_id,
		ai_itinerary.start_date,
		ai_itinerary.end_date,
		chat_session_id,
		ai_itinerary.title
	)
	.fetch_one(pool)
	.await
	.map_err(AppError::from)?
	.id;

	ai_itinerary.id = inserted_itinerary_id;

	// Insert itinerary events
	insert_event_list(ai_itinerary, pool).await?;

	// Insert bot message with itinerary
	let record = sqlx::query!(
		r#"
		INSERT INTO messages (chat_session_id, itinerary_id, is_user, timestamp, text)
		VALUES ($1, $2, FALSE, NOW(), $3)
		RETURNING id, timestamp;
		"#,
		chat_session_id,
		inserted_itinerary_id,
		ai_text.clone()
	)
	.fetch_one(pool)
	.await
	.map_err(AppError::from)?;

	let (bot_message_id, timestamp) = (record.id, record.timestamp);

	Ok(Message {
		id: bot_message_id,
		is_user: false,
		timestamp,
		text: ai_text,
		itinerary_id: Some(inserted_itinerary_id),
	})
}

/// Fetch all the chat session ids belonging to the user to made the request
///
/// # Method
/// `GET /api/chat/chats`
///
/// # Responses
/// - `200 OK` - [ChatsResponse] - list of chat session ids
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X GET http://localhost:3001/api/chat/chats
///   -H "Content-Type: application/json"
/// ```
#[utoipa::path(
	get,
	path="/chats",
	summary="Fetch user's chat session IDs",
	description="Fetches a list of all chat session IDs belonging to the user.",
	responses(
		(
			status=200,
			description="Successfully retrieved chat sessions",
			body=ChatsResponse,
			content_type="application/json",
			example=json!({
				"chat_sessions": [
					{
						"id": 5,
						"title": "Berlin, Germany"
					},
					{
						"id": 17,
						"title": "Shanghai, China"
					},
					{
						"id": 41,
						"title": "Miami, Florida, USA"
					}
				]
			})
		),
		(status=400, description="Bad Request"),
		(status=401, description="User has an invalid cookie/no cookie"),
		(status=405, description="Method Not Allowed - Must be GET"),
		(status=408, description="Request Timed Out"),
		(status=500, description="Internal Server Error")
	),
	security(("set-cookie"=[])),
	tag="Chat"
)]
pub async fn api_chats(
	Extension(user): Extension<AuthUser>,
	Extension(pool): Extension<PgPool>,
) -> ApiResult<Json<ChatsResponse>> {
	Ok(Json(ChatsResponse {
		chat_sessions: sqlx::query_as!(
			ChatSessionRow,
			r#"
			SELECT id, title from chat_sessions
			WHERE account_id=$1;
			"#,
			user.id
		)
		.fetch_all(&pool)
		.await
		.map_err(AppError::from)?,
	}))
}

/// Get a page of messages from this chat session belonging to the user who made the request
///
/// # Method
/// `POST /api/chat/messagePage`
///
/// # Request Body
/// - [MessagePageRequest]
///
/// # Responses
/// - `200 OK` - with body: [MessagePageResponse]
/// - `400 BAD_REQUEST` - Request payload contains invalid data (public error)
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// Fetch latest massages
/// ```bash
/// curl -X POST http://localhost:3001/api/chat/messagePage
///   -H "Content-Type: application/json"
///   -d '{
///         "chat_session_id": 3
///       }'
/// ```
/// Fetch messages ending with specific message
/// ```bash
/// curl -X POST http://localhost:3001/api/chat/messagePage
///   -H "Content-Type: application/json"
///   -d '{
///         "chat_session_id": 3,
///         "message_id": 6
///       }'
/// ```
#[utoipa::path(
	post,
	path="/messagePage",
	summary="Fetch a page of messages from a chat session",
	description="If no message id is provided, this fetches the latest messages from the chat session. If a message id is provided, that message and messages preceeding it will be fetched.",
	request_body(
		content=MessagePageRequest,
		content_type="application/json",
		description="Message id may be omitted to get the latest messages",
		examples(
			("Latest Messages"=(
				summary="Fetch the latest messages from a chat session",
				value=json!({
					"chat_session_id": 4
				})
			)),
			("Specific Messages"=(
				summary="Fetch a specific page of messages from a chat session",
				value=json!({
					"chat_session_id": 4,
					"message_id": 4
				})
			))
		)
	),
	responses(
		(
			status=200,
			description="Messages retrieved successfully",
			body=MessagePageResponse,
			content_type="application/json",
			examples(
				("Latest Messages"=(
					summary="The latest messages from a chat session",
					value=json!({
						"message_page": [
							{"id": 6, "is_user": true, "timestamp": "2025-10-14 11-34-19", "text": "User message"},
							{"id": 10, "is_user": false, "timestamp": "2025-10-14 11-34-24", "text": "Bot reply", "itinerary_id": 2},
							{"id": 12, "is_user": true, "timestamp": "2025-10-14 11-34-42", "text": "User message"},
							{"id": 22, "is_user": false, "timestamp": "2025-10-14 11-34-56", "text": "Bot reply", "itinerary_id": 5},
							{"id": 26, "is_user": true, "timestamp": "2025-10-14 11-35-10", "text": "User message"},
							{"id": 33, "is_user": false, "timestamp": "2025-10-14 11-35-19", "text": "Bot reply", "itinerary_id": 9},
							{"id": 39, "is_user": true, "timestamp": "2025-10-14 11-35-31", "text": "User message"},
							{"id": 44, "is_user": false, "timestamp": "2025-10-14 11-35-54", "text": "Bot reply", "itinerary_id": 14},
							{"id": 61, "is_user": true, "timestamp": "2025-10-14 11-36-24", "text": "User message"},
							{"id": 72, "is_user": false, "timestamp": "2025-10-14 11-36-29", "text": "Bot reply", "itinerary_id": 27}
						],
						"prev_message_id": 4
					})
				)),
				("Specific Messages"=(
					summary="A specific page of messages from a chat session",
					value=json!({
						"message_page": [
							{"id": 1, "is_user": true, "timestamp": "2025-10-14 11-33-21", "text": "User message"},
							{"id": 2, "is_user": false, "timestamp": "2025-10-14 11-33-35", "text": "Bot reply", "itinerary_id": 1},
							{"id": 3, "is_user": true, "timestamp": "2025-10-14 11-33-45", "text": "User message"},
							{"id": 4, "is_user": false, "timestamp": "2025-10-14 11-34-01", "text": "Bot reply", "itinerary_id": 1},
						],
						"prev_message_id": null
					})
				))
			)
		),
		(status=400, description="Bad Request"),
		(status=401, description="User has an invalid cookie/no cookie"),
		(status=405, description="Method Not Allowed - Must be POST"),
		(status=408, description="Request Timed Out"),
		(status=500, description="Internal Server Error")
	),
	security(("set-cookie"=[])),
	tag="Chat"
)]
pub async fn api_message_page(
	Extension(user): Extension<AuthUser>,
	Extension(pool): Extension<PgPool>,
	Json(MessagePageRequest {
		chat_session_id,
		message_id,
	}): Json<MessagePageRequest>,
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
	.map_err(AppError::from)?
	.into_iter()
	.rev()
	.map(|msg_row| Message {
		id: msg_row.id,
		is_user: msg_row.is_user,
		timestamp: msg_row.timestamp,
		text: msg_row.text,
		itinerary_id: msg_row.itinerary_id,
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
		prev_message_id,
	}))
}

/// Update an existing message with new text, and get a message back from the LLM
///
/// # Method
/// `POST /api/chat/updateMessage`
///
/// # Request Body
/// - [UpdateMessageRequest]
///
/// # Responses
/// - `200 OK` - with body: [Message] - message from LLM
/// - `400 BAD_REQUEST` - Request payload contains invalid data (public error)
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
/// - `404 NOT_FOUND` - The provided message id does not belong to the user or does not exist (public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X POST http://localhost:3001/api/chat/updateMessage
///   -H "Content-Type: application/json"
///   -d '{
///         "message_id": 3,
///         "new_text": "Updated message",
///         "itinerary_id": 7
///       }'
/// ```
#[utoipa::path(
	post,
	path="/updateMessage",
	summary="Update the text of a message and wait for a reply from the LLM",
	description="Updating a message deletes all proceeding messages, updates the text of the given message, and returns a response from the LLM.",
	request_body(
		content=UpdateMessageRequest,
		content_type="application/json",
		description="Itinerary id is optional and is used to give context to the LLM.",
		example=json!({
			"message_id": 41,
			"new_text": "Updated message content",
			"itinerary_id": 17
		})
	),
	responses(
		(
			status=200,
			description="Message updated, and LLM replied successfully",
			body=Message,
			content_type="application/json",
			example=json!({
				"id": 43,
				"is_user": false,
				"timestamp": "2025-10-14 11-38-52",
				"text": "Bot reply",
				"itinerary_id": 19
			})
		),
		(status=400, description="Bad Request"),
		(status=401, description="User has an invalid cookie/no cookie"),
		(status=404, description="Message not found in this chat session for this user"),
		(status=405, description="Method Not Allowed - Must be POST"),
		(status=408, description="Request Timed Out"),
		(status=500, description="Internal Server Error")
	),
	security(("set-cookie"=[])),
	tag="Chat"
)]
pub async fn api_update_message(
	Extension(user): Extension<AuthUser>,
	Extension(pool): Extension<PgPool>,
	Extension(agent): Extension<AgentType>,
	Json(UpdateMessageRequest {
		message_id,
		new_text,
		itinerary_id,
	}): Json<UpdateMessageRequest>,
) -> ApiResult<Json<Message>> {
	if new_text.is_empty() {
		return Err(AppError::BadRequest(String::from("Text cannot be empty")));
	}

	// Get the message and verify ownership in one query
	let message_info = sqlx::query!(
		r#"
		SELECT m.chat_session_id, m.timestamp
		FROM messages m
		INNER JOIN chat_sessions c ON m.chat_session_id = c.id
		WHERE m.id = $1 AND c.account_id = $2 AND m.is_user = TRUE;
		"#,
		message_id,
		user.id
	)
	.fetch_optional(&pool)
	.await
	.map_err(AppError::from)?
	.ok_or(AppError::NotFound)?;

	let chat_session_id = message_info.chat_session_id;
	let message_timestamp = message_info.timestamp;
	
	let to_delete = sqlx::query!(
		r#"
		SELECT id, chat_session_id, timestamp, text
		FROM messages
		WHERE chat_session_id = $1
		  AND timestamp > $2
		  AND id != $3;
		"#,
		chat_session_id,
		message_timestamp,
		message_id
	)
	.fetch_all(&pool)
	.await
	.map_err(AppError::from)?;
	
	eprintln!("About to delete {} messages:", to_delete.len());
	for msg in &to_delete {
		eprintln!("  - ID: {}, Chat: {}, Timestamp: {:?}, Text: {}", 
			msg.id, msg.chat_session_id, msg.timestamp, msg.text);
	}

	// Delete future messages in this chat session only
	let delete_result = sqlx::query!(
		r#"
		DELETE FROM messages
		WHERE chat_session_id = $1
		  AND timestamp > $2
		  AND id != $3;
		"#,
		chat_session_id,
		message_timestamp,
		message_id
	)
	.execute(&pool)
	.await
	.map_err(AppError::from)?;
	
	eprintln!("Deleted {} messages", delete_result.rows_affected());
	eprintln!("=========================");

	// Update the user message
	sqlx::query!(
		r#"
		UPDATE messages
		SET text = $1, timestamp = NOW()
		WHERE id = $2;
		"#,
		new_text,
		message_id
	)
	.execute(&pool)
	.await
	.map_err(AppError::from)?;

	// Call LLM and insert bot response
	let bot_message = send_message_to_llm(
		new_text.as_str(),
		user.id,
		chat_session_id,
		itinerary_id,
		&pool,
		&agent,
	)
	.await?;

	Ok(Json(bot_message))
}

/// Send a new message, and get a message back from the LLM
///
/// # Method
/// `POST /api/chat/sendMessage`
///
/// # Request Body
/// - [SendMessageRequest]
///
/// # Responses
/// - `200 OK` - with body: [SendMessageResponse] - contains message from LLM
/// - `400 BAD_REQUEST` - Request payload contains invalid data (public error)
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
/// - `404 NOT_FOUND` - The provided chat session id does not belong to the user or does not exist (public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X POST http://localhost:3001/api/chat/sendMessage
///   -H "Content-Type: application/json"
///   -d '{
///         "chat_session_id": 6,
///         "text": "New message",
///         "itinerary_id": 7
///       }'
/// ```
#[utoipa::path(
	post,
	path="/sendMessage",
	summary="Send a message and wait for a reply from the LLM",
	description="Ask the LLM to generate an itinerary and it should respond with one.",
	request_body(
		content=SendMessageRequest,
		content_type="application/json",
		description="Itinerary id is optional and is used to give context to the LLM.",
		example=json!({
			"chat_session_id": 12,
			"text": "Make an itinerary",
			"itinerary_id": 13
		})
	),
	responses(
		(
			status=200,
			description="Message sent, and LLM replied successfully",
			body=SendMessageResponse,
			content_type="application/json",
			example=json!({
				"user_message_id": 52,
				"bot_message": {
					"id": 53,
					"is_user": false,
					"timestamp": "2025-10-14 11-39-10",
					"text": "Bot reply",
					"itinerary_id": 14
				}
			})
		),
		(status=400, description="Bad Request"),
		(status=401, description="User has an invalid cookie/no cookie"),
		(status=404, description="Chat session not found for this user"),
		(status=405, description="Method Not Allowed - Must be POST"),
		(status=408, description="Request Timed Out"),
		(status=500, description="Internal Server Error")
	),
	security(("set-cookie"=[])),
	tag="Chat"
)]
pub async fn api_send_message(
	Extension(user): Extension<AuthUser>,
	Extension(pool): Extension<PgPool>,
	Extension(agent): Extension<AgentType>,
	Json(SendMessageRequest {
		chat_session_id,
		text,
		itinerary_id,
	}): Json<SendMessageRequest>,
) -> ApiResult<Json<SendMessageResponse>> {
	if text.is_empty() {
		return Err(AppError::BadRequest(String::from("Text cannot be empty")));
	}

	// verify the given chat session belongs to this user
	sqlx::query!(
		r#"
		SELECT id FROM chat_sessions
		WHERE id=$1 AND account_id=$2;
		"#,
		chat_session_id,
		user.id
	)
	.fetch_optional(&pool)
	.await
	.map_err(AppError::from)?
	.ok_or(AppError::NotFound)?;

	// insert user message into db
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
	.map_err(AppError::from)?
	.id;

	// call llm and insert bot response into db
	let bot_message = send_message_to_llm(
		text.as_str(),
		user.id,
		chat_session_id,
		itinerary_id,
		&pool,
		&agent,
	)
	.await?;

	Ok(Json(SendMessageResponse {
		user_message_id,
		bot_message,
	}))
}

/// Get an empty chat session id belonging to this user, or create one if one doesn't exist
///
/// # Method
/// `GET /api/chat/newChat`
///
/// # Responses
/// - `200 OK` - with body: [NewChatResponse]
/// - `400 BAD_REQUEST` - Request payload contains invalid data (public error)
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X POST http://localhost:3001/api/chat/sendMessage
///   -H "Content-Type: application/json"
///   -d '{
///         "chat_session_id": 6,
///         "text": "New message",
///         "itinerary_id": 7
///       }'
/// ```
#[utoipa::path(
	get,
	path="/newChat",
	summary="Get the chat session id for an empty chat",
	description="Creates a new empty chat session for this user if one doesn't already exist, and returns its chat session id.",
	responses(
		(
			status=200,
			description="New chat session retrieved successfully",
			body=NewChatResponse,
			content_type="application/json",
			example=json!({
				"chat_session_id": 13
			})
		),
		(status=400, description="Bad Request"),
		(status=401, description="User has an invalid cookie/no cookie"),
		(status=405, description="Method Not Allowed - Must be GET"),
		(status=408, description="Request Timed Out"),
		(status=500, description="Internal Server Error")
	),
	security(("set-cookie"=[])),
	tag="Chat"
)]
pub async fn api_new_chat(
	Extension(user): Extension<AuthUser>,
	Extension(pool): Extension<PgPool>,
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
	.map_err(AppError::from)?;

	let chat_session_id = match chat_sessions.first() {
		Some(record) => record.id,
		None => {
			// make a new chat session
			sqlx::query!(
				r#"
				INSERT INTO chat_sessions (account_id, title)
				VALUES ($1, 'New Chat')
				RETURNING id
				"#,
				user.id
			)
			.fetch_one(&pool)
			.await
			.map_err(AppError::from)?
			.id
		}
	};

	Ok(Json(NewChatResponse { chat_session_id }))
}

/// Delete the chat session with the given ID
///
/// # Method
/// `DELETE /api/chat/:id`
///
/// # Responses
/// - `200 OK` - chat session and associated messages and unsaved itineraries successfully deleted
/// - `400 BAD_REQUEST` - Request payload contains invalid data (public error)
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
/// - `404 NOT_FOUND` - The provided chat session id does not belong to the user or does not exist (public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X DELETE http://localhost:3001/api/chat/7
///   -H "Content-Type: application/json"
/// ```
#[utoipa::path(
	delete,
	path="/{id}",
	summary="Delete the given chat session",
	description="Deletes a chat session and its associated messages and unsaved, private itineraries if it belongs to the user making the request.",
	responses(
		(status=200, description="Chat session and associated messages and unsaved, private itineraries deleted successfully"),
		(status=400, description="Bad Request"),
		(status=401, description="User has an invalid cookie/no cookie"),
		(status=404, description="Chat session not found for this user"),
		(status=405, description="Method Not Allowed - Must be DELETE"),
		(status=408, description="Request Timed Out"),
		(status=500, description="Internal Server Error")
	),
	security(("set-cookie"=[])),
	tag="Chat"
)]
pub async fn api_delete_chat(
	Extension(user): Extension<AuthUser>,
	Extension(pool): Extension<PgPool>,
	Path(chat_session_id): Path<i32>,
) -> ApiResult<()> {
	// itineraries do not cascade, so we delete manually
	sqlx::query!(
		r#"
		DELETE FROM itineraries
		WHERE
			chat_session_id=$1 AND
			account_id=$2 AND
			is_public=FALSE AND
			saved=FALSE;
		"#,
		chat_session_id,
		user.id
	)
	.fetch_optional(&pool)
	.await
	.map_err(AppError::from)?;

	// messages will cascade
	sqlx::query!(
		r#"
		DELETE FROM chat_sessions
		WHERE id=$1 AND account_id=$2
		RETURNING id;
		"#,
		chat_session_id,
		user.id
	)
	.fetch_optional(&pool)
	.await
	.map_err(AppError::from)?
	.ok_or(AppError::NotFound)?;

	Ok(())
}

/// Rename a chat session
///
/// # Method
/// `POST /api/chat/rename`
///
/// # Request Body
/// - [RenameRequest]
///
/// # Responses
/// - `200 OK`
/// - `400 BAD_REQUEST` - Request payload contains invalid data (public error)
/// - `401 UNAUTHORIZED` - When authentication fails (handled in middleware, public error)
/// - `404 NOT_FOUND` - The provided chat session id does not belong to the user or does not exist (public error)
/// - `500 INTERNAL_SERVER_ERROR` - Internal error (private)
///
/// # Examples
/// ```bash
/// curl -X POST http://localhost:3001/api/chat/rename
///   -H "Content-Type: application/json"
///   -d '{
///         "new_title": "Tokio, Japan (lmao)",
///         "id": 16
///       }'
/// ```
#[utoipa::path(
	post,
	path="/rename",
	summary="Rename a chat session",
	description="Renames a chat session that belongs to this user with the given ID to the given title.",
	request_body(
		content=RenameRequest,
		content_type="application/json",
		description="Chat session ID must belong to the user who sent the request. New Title must not be empty string.",
		example=json!({
			"new_title": "Tokio, Japan (lmao)",
			"id": 16
		})
	),
	responses(
		(status=200, description="Chat renamed successfully"),
		(status=400, description="Bad Request"),
		(status=401, description="User has an invalid cookie/no cookie"),
		(status=404, description="Chat session not found for this user"),
		(status=405, description="Method Not Allowed - Must be POST"),
		(status=408, description="Request Timed Out"),
		(status=500, description="Internal Server Error")
	),
	security(("set-cookie"=[])),
	tag="Chat"
)]
pub async fn api_rename(
	Extension(user): Extension<AuthUser>,
	Extension(pool): Extension<PgPool>,
	Json(RenameRequest { new_title, id }): Json<RenameRequest>,
) -> ApiResult<()> {
	// no empty titles
	if new_title.is_empty() {
		return Err(AppError::BadRequest(String::from(
			"New title must not be empty",
		)));
	}

	// verify chat session belongs to this user
	sqlx::query!(
		r#"SELECT id from chat_sessions WHERE id=$1 AND account_id=$2"#,
		id,
		user.id
	)
	.fetch_optional(&pool)
	.await
	.map_err(AppError::from)?
	.ok_or(AppError::NotFound)?;

	//change name
	sqlx::query!(
		r#"UPDATE chat_sessions SET title=$1 WHERE id=$2"#,
		new_title,
		id
	)
	.execute(&pool)
	.await
	.map_err(AppError::from)?;

	Ok(())
}

/// Create the chat routes with authentication middleware.
///
/// # Routes
/// - `GET /chats` - Get metadata for all the user's chat sessions (protected)
/// - `POST /messagePage` - Gets a page of messages in the session, ending with message_id or the latest message (protected)
/// - `POST /updateMessage` - Updates a user's message and waits for a bot reply (protected)
/// - `POST /sendMessage` - Sends a user's message and waits for a bot reply (protected)
/// - `GET /newChat` - Gets a chat session id for an empty chat (protected)
/// - `DELETE /:id` - Delete a chat session and associated messages (protected)
/// - `POST /rename` - Renames the title of a chat session (protected)
///
/// # Middleware
/// All routes are protected by `middleware_auth` which validates the `auth-token` cookie.
pub fn chat_routes() -> AxumRouter {
	AxumRouter::new()
		.route("/chats", get(api_chats))
		.route("/messagePage", post(api_message_page))
		.route("/updateMessage", post(api_update_message))
		.route("/sendMessage", post(api_send_message))
		.route("/newChat", get(api_new_chat))
		.route("/{id}", delete(api_delete_chat))
		.route("/rename", post(api_rename))
		.route_layer(axum::middleware::from_fn(middleware_auth))
}
