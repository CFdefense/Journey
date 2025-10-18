export type Message = {
	id: number,
	/// Whether this message was sent by the user or the LLM
	is_user: boolean,
	timestamp: string,
	text: string,
	itinerary_id: number | null
}

export type ChatsResponse = {
    chat_sessions: number[];
};

export type MessagePageRequest = {
	chat_session_id: number,
	message_id: number | null
}

export type MessagePageResponse = {
	message_page: Message[],
	/// The id of the message that comes right before the first message in this.message_page
	prev_message_id: number | null
}

export type UpdateMessageRequest = {
	/// The id of the user's message you want to update
	message_id: number,
	new_text: string
}

export type SendMessageRequest = {
	chat_session_id: number,
	text: string
}

export type SendMessageResponse = {
	/// The id of the message you just sent
	user_message_id: number,
	bot_message: Message
}