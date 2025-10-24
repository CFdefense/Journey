export type Message = {
	id: number;
	/// Whether this message was sent by the user or the LLM
	is_user: boolean;
	/// %Y-%m-%dT%H:%M:%S%.f
	timestamp: string;
	text: string;
	itinerary_id: number | null;
};

/// Row model for `chat_sessions` table
export type ChatSessionRow = {
	/// Primary key
	id: number;
	/// Name of chat for user context
	title: string;
};

export type ChatsResponse = {
	chat_sessions: ChatSessionRow[];
};

export type MessagePageRequest = {
	/// chat session to fetch page from
	chat_session_id: number;
	/// Possible message id to represent the end of the page
	/// * If Some, it will fetch this message and consecutive previous messages in chronological order
	/// * If None, it will fetch the latest consecutive messages from the chat session in chronological order
	message_id: number | null;
};

export type MessagePageResponse = {
	/// A page of messages guaranteed to be sorted in chronological order
	message_page: Message[];
	/// The id of the message that comes chronologically before the first message in message_page, if it exists
	prev_message_id: number | null;
};

export type UpdateMessageRequest = {
	/// ID of the message to update. This message must belong to a chat session which belongs to the user who made the request
	message_id: number;
	/// The text to replace the old content with
	new_text: string;
	/// A possible itinerary to give context to the LLM
	itinerary_id: number | null;
};

export type SendMessageRequest = {
	/// The chat session to send this message in. It must belong to the user making the request.
	chat_session_id: number;
	/// The content of the message
	text: string;
	/// A possible itinerary to give context to the LLM
	itinerary_id: number | null;
};

export type SendMessageResponse = {
	/// The newly-created id of the message you just sent
	user_message_id: number;
	/// The response message from the LLM
	bot_message: Message;
};

export type NewChatResponse = {
	/// this chat session is guaranteed to not have any messages in it
	chat_session_id: number;
};
