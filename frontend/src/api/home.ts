const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;
import type { ApiResult } from "../helpers/global";
import type {
	MessagePageRequest,
	MessagePageResponse,
	SendMessageRequest,
	SendMessageResponse,
	ChatsResponse,
	Message,
	UpdateMessageRequest,
	RenameRequest
} from "../models/chat";

/// Calls chats
///
/// # Method
/// Sends a `GET /api/chat/chats` request to fetch all chat sessions for the current user.
///
/// # Returns
/// - On success: `ChatsResponse` containing all existing chat sessions.
/// - On failure: A null `ChatsResponse` with a non-200 status code.
///
/// # Exceptions
/// Never throws an exception
export async function apiChats(): Promise<ApiResult<ChatsResponse>> {
	// TODO: get chats from cache if it exists
	try {
		const response = await fetch(`${API_BASE_URL}/api/chat/chats`, {
			method: "GET",
			credentials: import.meta.env.DEV ? "include" : "same-origin"
		});
		if (!response.ok) {
			return { result: null, status: response.status };
		}
		return { result: await response.json(), status: response.status };
	} catch (error) {
		console.error("apiChats error:", error);
		return { result: null, status: -1 };
	}
}

/// Calls messagePage
///
/// # Method
/// Sends a `POST /api/chat/messagePage` request to fetch a list of messages
/// from a specific chat session.
///
/// # Parameters
/// - `payload`: A `MessagePageRequest` object containing chat session ID.
///
/// # Returns
/// - On success: `MessagePageResponse` containing the list of messages for a page.
/// - On failure: A null `MessagePageResponse` with a non-200 status code.
///
/// # Exceptions
/// Never throws an exception
export async function apiMessages(
	payload: MessagePageRequest
): Promise<ApiResult<MessagePageResponse>> {
	// TODO: get messages from cache if it exists
	try {
		const response = await fetch(`${API_BASE_URL}/api/chat/messagePage`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: import.meta.env.DEV ? "include" : "same-origin",
			body: JSON.stringify(payload)
		});
		if (!response.ok) {
			return { result: null, status: response.status };
		}
		const pageRes: MessagePageResponse = await response.json();
		for (const msg of pageRes.message_page) {
			msg.timestamp += "Z";
		}
		return { result: pageRes, status: response.status };
	} catch (error) {
		console.error("apiMessages error:", error);
		return { result: null, status: -1 };
	}
}

/// Calls sendMessage
///
/// # Method
/// Sends a `POST /api/chat/sendMessage` request to send a new user message to the backend,
/// and receive an AI-generated bot response.
///
/// # Parameters
/// - `payload`: A `SendMessageRequest` object containing the chat session ID and message text.
///
/// # Returns
/// - On success: `SendMessageResponse` containing both the sent user message ID and bot response.
/// - On failure: Returns a null `SendMessageResponse` with a non-200 status code.
///
/// # Exceptions
/// Never throws an exception
export async function apiSendMessage(
	payload: SendMessageRequest
): Promise<ApiResult<SendMessageResponse>> {
	try {
		const response = await fetch(`${API_BASE_URL}/api/chat/sendMessage`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: import.meta.env.DEV ? "include" : "same-origin",
			body: JSON.stringify(payload)
		});
		if (!response.ok) {
			return { result: null, status: response.status };
		}
		const sendRes: SendMessageResponse = await response.json();
		sendRes.bot_message.timestamp += "Z";
		return { result: sendRes, status: response.status };
	} catch (error) {
		console.error("apiSendMessage error:", error);
		return { result: null, status: -1 };
	}
}

/// Calls newChat
///
/// # Method
/// Sends a `GET /api/chat/newChat` request to create a brand-new chat session
/// for the current authenticated user.
///
/// # Returns
/// - On success: The numeric ID of the newly created chat session.
/// - On failure: null with a non-200 status code
///
/// # Exceptions
/// Never throws an exception
export async function apiNewChatId(): Promise<ApiResult<number>> {
	try {
		const response = await fetch(`${API_BASE_URL}/api/chat/newChat`, {
			method: "GET",
			credentials: import.meta.env.DEV ? "include" : "same-origin"
		});
		if (!response.ok) {
			return { result: null, status: response.status };
		}
		return {
			result: (await response.json()).chat_session_id,
			status: response.status
		};
	} catch (error) {
		console.error("apiChats error:", error);
		return { result: null, status: -1 };
	}
}

/// Calls deleteChat
///
/// # Method
/// Sends a `DELETE /api/chat/:id` to delete a specific chat session
/// and its associated messages for the current authenticated user.
///
/// # Returns
/// - On success: The numeric ID of the deleted chat session.
/// - On failure: null with a non-200 status code
///
/// # Exceptions
/// Never throws an exception
export async function apiDeleteChat(
	payload: number
): Promise<ApiResult<number>> {
	try {
		const response = await fetch(`${API_BASE_URL}/api/chat/${payload}`, {
			method: "DELETE",
			credentials: import.meta.env.DEV ? "include" : "same-origin"
		});
		if (!response.ok) {
			return { result: null, status: response.status };
		}
		return { result: payload, status: response.status };
	} catch (error) {
		console.error("apiDeleteChat error:", error);
		return { result: null, status: -1 };
	}
}

/// Renames a chat title
///
/// # Method
/// Sends a `POST /api/chat/rename` request to rename that chat title
/// of a specific chat session.
///
/// # Parameters
/// - `payload`: A `RenameRequest` object containing chat session ID and new title.
///
/// # Returns
/// - On success: Just a 200
/// - On failure: Just a non-200 status code.
///
/// # Exceptions
/// Never throws an exception
export async function apiRenameChat(
	payload: RenameRequest
): Promise<ApiResult<void>> {
	// TODO: update chat title in cache
	try {
		const response = await fetch(`${API_BASE_URL}/api/chat/rename`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: import.meta.env.DEV ? "include" : "same-origin",
			body: JSON.stringify(payload)
		});
		if (!response.ok) {
			return { result: null, status: response.status };
		}
		return { result: null, status: response.status };
	} catch (error) {
		console.error("apiRenameChat error:", error);
		return { result: null, status: -1 };
	}
}

/// Updates an existing message with new text and receives a new AI response
///
/// # Method
/// Sends a `POST /api/chat/updateMessage` request to update a user message,
/// delete all subsequent messages, and receive a new AI-generated bot response.
///
/// # Parameters
/// - `payload`: An `UpdateMessageRequest` object containing:
///   - `message_id`: The ID of the message to update
///   - `new_text`: The updated message text
///   - `itinerary_id` (optional): Itinerary context for the LLM
///
/// # Returns
/// - On success: `Message` object containing the bot's response with status 200
/// - On failure: Returns null result with appropriate status code:
///
/// # Exceptions
/// Never throws an exception
export async function apiUpdateMessage(
	payload: UpdateMessageRequest
): Promise<ApiResult<Message>> {
	try {
		const response = await fetch(`${API_BASE_URL}/api/chat/updateMessage`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: import.meta.env.DEV ? "include" : "same-origin",
			body: JSON.stringify(payload)
		});
		if (!response.ok) {
			return { result: null, status: response.status };
		}
		const updateRes: Message = await response.json();
		updateRes.timestamp += "Z"; // Add this line to match other API functions
		return { result: updateRes, status: response.status };
	} catch (error) {
		console.error("apiUpdateMessage error:", error);
		return { result: null, status: -1 };
	}
}
