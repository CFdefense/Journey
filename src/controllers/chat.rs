use axum::{routing::{get, post}, Extension, Json, Router};
use chrono::NaiveDate;
use sqlx::PgPool;
use crate::{controllers::itinerary::insert_event_list, error::{ApiResult, AppError}, global::MESSAGE_PAGE_LEN, http_models::{chat_session::{ChatsResponse, NewChatResponse}, event::Event, itinerary::{EventDay, Itinerary}, message::{Message, MessagePageRequest, MessagePageResponse, SendMessageRequest, SendMessageResponse, UpdateMessageRequest}}, middleware::{middleware_auth, AuthUser}, sql_models::message::MessageRow};

/// Sends message and latest itinerary in chat session to llm, and waits for response.
///
/// When the bot replies, it's message and itinerary are inserted into the db.
/// # Warning!
/// Assumes the user's message has already been inserted into the db.
async fn send_message_to_llm(text: &str, account_id: i32, chat_session_id: i32, pool: &PgPool) -> ApiResult<Message> {
	// TODO: this is where the LLM call will be.
	// It should generate an itinerary, insert it into the db, and give some text.
	// Fow now we just make a temporary message.
	let ai_text = "Bot reply";
	let ai_itinerary = Itinerary {
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
					event_description: String::from("A beautiful stroll along a river in this cute small town."),
					event_name: String::from("Family Walking Path")
				}],
				noon_events: vec![Event {
					id: 2,
					street_address: String::from("35 Campus Court"),
					postal_code: 12601,
					city: String::from("Poughkeepsie"),
					event_type: String::from("Restaurant"),
					event_description: String::from("Local Italian restaurant known for its authentic pasta and upscale dining."),
					event_name: String::from("Cosimos")
				}],
				afternoon_events: vec![Event {
					id: 3,
					street_address: String::from("200 E 42nd St"),
					postal_code: 10017,
					city: String::from("New York"),
					event_type: String::from("Museum"),
					event_description: String::from("World famous art museum with a focus on modern works, including Starry Starry Night by VanGough."),
					event_name: String::from("Museum of Modern Art- MoMA")
				}],
				evening_events: vec![Event {
					id: 4,
					street_address: String::from("1 S Broad St"),
					postal_code: 19107,
					city: String::from("Philadelphia"),
					event_type: String::from("Concert"),
					event_description: String::from("Music center which hosts local and national bands."),
					event_name: String::from("Jazz night at Broad Street")
				}],
				date: NaiveDate::parse_from_str("2025-11-05", "%Y-%m-%d").unwrap()
			},
			EventDay {
				morning_events: vec![Event {
					id: 5,
					street_address: String::from("1 Citizens Bank Way"),
					postal_code: 19148,
					city: String::from("Philadelphia"),
					event_type: String::from("Sports"),
					event_description: String::from("A Phillies baseball game is a must-do for locals and visitors alike."),
					event_name: String::from("Phillies Baseball Game")
				}],
				noon_events: vec![Event {
					id: 6,
					street_address: String::from("5250 S Park Dr"),
					postal_code: 60615,
					city: String::from("Chicago"),
					event_type: String::from("Festival"),
					event_description: String::from("Annual music festival with the biggest names in pop and indie scenes."),
					event_name: String::from("LollaPalooza")
				}],
				afternoon_events: vec![Event {
					id: 7,
					street_address: String::from("1 Rue de la Seine"),
					postal_code: 0,
					city: String::from("Paris"),
					event_type: String::from("Museum"),
					event_description: String::from("Explore the beautiful landmark of Paris."),
					event_name: String::from("Eiffel Tower")
				}],
				evening_events: vec![Event {
					id: 8,
					street_address: String::from("3 Rue de la Museu"),
					postal_code: 0,
					city: String::from("Paris"),
					event_type: String::from("Museum"),
					event_description: String::from("Wander the halls of the world famous art museum."),
					event_name: String::from("le Louvre")
				}],
				date: NaiveDate::parse_from_str("2025-11-06", "%Y-%m-%d").unwrap()
			}
		],
	    chat_session_id: None,
		title: String::from("World Tour 11/5-15 2025")
	};

	// insert generated itinerary into db
	let itinerary_id = Some(sqlx::query!(
		r#"
		INSERT INTO itineraries (account_id, is_public, start_date, end_date, chat_session_id, saved)
		VALUES ($1, FALSE, $2, $3, $4, TRUE)
		RETURNING id;
		"#,
		account_id,
		ai_itinerary.start_date,
		ai_itinerary.end_date,
		chat_session_id
	)
	.fetch_one(pool)
	.await
	.map_err(|e| AppError::from(e))?
	.id);

	insert_event_list(ai_itinerary, pool).await?;

	// insert bot message into db
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
	.fetch_one(pool)
	.await
	.map_err(|e| AppError::from(e))?;

	let (bot_message_id, timestamp) = (record.id, record.timestamp);

	Ok(Message {
		id: bot_message_id,
		is_user: false,
		timestamp,
		text: String::from(ai_text),
		itinerary_id,
	})
}

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
		WHERE m.id=$1 AND c.account_id=$2 AND m.is_user=TRUE;
		"#,
		message_id,
		user.id
	)
	.fetch_optional(&pool)
	.await
	.map_err(|e| AppError::from(e))?
	.ok_or(AppError::NotFound)?;

	// delete future messages
	sqlx::query!(
		r#"
		DELETE FROM messages
		WHERE timestamp > (
			SELECT timestamp
			FROM messages
			WHERE id=$1
		);
		"#,
		message_id
	)
	.execute(&pool)
	.await
	.map_err(|e| AppError::from(e))?;

	// update user message
	let chat_session_id = sqlx::query!(
		r#"
		UPDATE messages
		SET text=$1, timestamp=NOW()
		WHERE is_user=TRUE AND id=$2
		RETURNING chat_session_id;
		"#,
		new_text,
		message_id
	)
	.fetch_one(&pool)
	.await
	.map_err(|e| AppError::from(e))?
	.chat_session_id;

	// call llm and insert bot response into db
	let bot_message = send_message_to_llm(new_text.as_str(), user.id, chat_session_id, &pool).await?;

	Ok(Json(bot_message))
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
		SELECT id FROM chat_sessions
		WHERE id=$1 AND account_id=$2;
		"#,
		chat_session_id,
		user.id
	)
	.fetch_optional(&pool)
	.await
	.map_err(|e| AppError::from(e))?
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
	.map_err(|e| AppError::from(e))?
	.id;

	// call llm and insert bot response into db
	let bot_message = send_message_to_llm(text.as_str(), user.id, chat_session_id, &pool).await?;

	Ok(Json(SendMessageResponse {
		user_message_id,
		bot_message
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
			// make a new chat session
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