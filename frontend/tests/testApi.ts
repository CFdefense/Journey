
//================================//
// GENERATED FILE - DO NOT MODIFY //
//================================//

const API_BASE_URL = "http://localhost:3001";
export const test_state = {
	dev_mode: true
};
import type { ApiResult } from "../src/helpers/global";
import { customFetch } from "./customFetch";


import type {
	CurrentResponse,
	LoginRequest,
	SignUpRequest,
	UpdateRequest
} from "../src/models/account";

/// Calls login
///
/// # Method
/// Calls `POST /api/account/login` through `apiLogin`.
///
/// # Returns
/// Status of login call.
/// * 200: successful login
/// * -1: fetch call threw an exception
///
/// # Exceptions
/// Never throws an exception
export async function apiLogin(
	payload: LoginRequest
): Promise<ApiResult<void>> {
	try {
		const response = await customFetch(`${API_BASE_URL}/api/account/login`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: test_state.dev_mode ? "include" : "same-origin",
			body: JSON.stringify(payload)
		});
		return { result: null, status: response.status };
	} catch (error) {
		console.error("Login API error: ", error);
		return { result: null, status: -1 };
	}
}

/// Calls signup
///
/// # Method
/// Sends a `POST /api/account/signup` request to create a new user account.
///
/// # Returns
/// Status of signup call.
/// * 200: successful signup
/// * -1: fetch call threw an exception
///
/// # Exceptions
/// Never throws an exception
export async function apiSignUp(
	payload: SignUpRequest
): Promise<ApiResult<void>> {
	try {
		const response = await customFetch(`${API_BASE_URL}/api/account/signup`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: test_state.dev_mode ? "include" : "same-origin",
			body: JSON.stringify(payload)
		});
		return { result: null, status: response.status };
	} catch (error) {
		console.error("Sign Up API error: ", error);
		return { result: null, status: -1 };
	}
}

/// Calls logout
///
/// # Method
/// Sends a `GET /api/account/logout` request set cookie as expired.
///
/// # Returns
/// Status of logout call.
/// * 200: successful logout
/// * -1: fetch call threw an exception
///
/// # Exceptions
/// Never throws an exception
export async function apiLogout(): Promise<ApiResult<void>> {
	try {
		const response = await customFetch(`${API_BASE_URL}/api/account/logout`, {
			method: "GET",
			credentials: test_state.dev_mode ? "include" : "same-origin"
		});
		return { result: null, status: response.status };
	} catch (error) {
		console.error("Logout Up API error: ", error);
		return { result: null, status: -1 };
	}
}

/// Calls validate
///
/// # Method
/// Sends a `GET /api/account/validate` request set cookie as expired.
///
/// # Returns
/// Whether session is currently valid.
///
/// # Exceptions
/// Never throws an exception
export async function apiValidate(): Promise<ApiResult<void>> {
	try {
		const response = await customFetch(`${API_BASE_URL}/api/account/validate`, {
			method: "GET",
			credentials: test_state.dev_mode ? "include" : "same-origin"
		});
		return { result: null, status: response.status };
	} catch (error) {
		console.error("Validate API error: ", error);
		return { result: null, status: -1 };
	}
}

/// Calls update account
///
/// # Method
/// Sends a `POST /api/account/update` request to update user account details.
///
/// # Returns updated account information if successful.
/// # Throws Error with message to be displayed.
export async function apiUpdateAccount(
	payload: UpdateRequest
): Promise<ApiResult<CurrentResponse>> {
	try {
		const response = await customFetch(`${API_BASE_URL}/api/account/update`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: "include",
			body: JSON.stringify(payload)
		});
		return {
			result: response.ok ? await response.json() : null,
			status: response.status
		};
	} catch (error) {
		console.error("Update Account API error: ", error);
		return { result: null, status: -1 };
	}
}

/// Calls current
///
/// # Method
/// Sends a `GET /api/account/current` request set cookie as expired.
///
/// # Returns
/// Non-sensitive account info and the status of the API call
/// * 200: successful fetch
/// * -1: fetch call threw an exception
///
/// # Exceptions
/// Never throws an exception
export async function apiCurrent(): Promise<ApiResult<CurrentResponse>> {
	// TODO: get account data from cache if it exists
	try {
		const response = await customFetch(`${API_BASE_URL}/api/account/current`, {
			method: "GET",
			credentials: test_state.dev_mode ? "include" : "same-origin"
		});
		return {
			result: response.ok ? await response.json() : null,
			status: response.status
		};
	} catch (error) {
		console.error("Current API error: ", error);
		return { result: null, status: -1 };
	}
}



import type {
	MessagePageRequest,
	MessagePageResponse,
	SendMessageRequest,
	SendMessageResponse,
	ChatsResponse,
	Message,
	UpdateMessageRequest,
	RenameRequest
} from "../src/models/chat";

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
		const response = await customFetch(`${API_BASE_URL}/api/chat/chats`, {
			method: "GET",
			credentials: test_state.dev_mode ? "include" : "same-origin"
		});
		return {
			result: response.ok ? await response.json() : null,
			status: response.status
		};
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
		const response = await customFetch(`${API_BASE_URL}/api/chat/messagePage`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: test_state.dev_mode ? "include" : "same-origin",
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
		const response = await customFetch(`${API_BASE_URL}/api/chat/sendMessage`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: test_state.dev_mode ? "include" : "same-origin",
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
		const response = await customFetch(`${API_BASE_URL}/api/chat/newChat`, {
			method: "GET",
			credentials: test_state.dev_mode ? "include" : "same-origin"
		});
		return {
			result: response.ok
				? (await response.json()).chat_session_id
				: null,
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
export async function apiDeleteChat(payload: number): Promise<ApiResult<void>> {
	try {
		const response = await customFetch(`${API_BASE_URL}/api/chat/${payload}`, {
			method: "DELETE",
			credentials: test_state.dev_mode ? "include" : "same-origin"
		});
		return { result: null, status: response.status };
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
		const response = await customFetch(`${API_BASE_URL}/api/chat/rename`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: test_state.dev_mode ? "include" : "same-origin",
			body: JSON.stringify(payload)
		});
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
		const response = await customFetch(`${API_BASE_URL}/api/chat/updateMessage`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: test_state.dev_mode ? "include" : "same-origin",
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



import type {
	Itinerary,
	SavedItinerariesResponse,
	SaveResponse,
	SearchEventRequest,
	SearchEventResponse,
	UnsaveRequest,
	UserEventRequest,
	UserEventResponse
} from "../src/models/itinerary";

/// Calls itinerary details
///
/// # Method
/// Sends a `GET /api/itinerary/:itinerary_id` request to fetch the full details
/// of a specific itinerary for the currently authenticated user.
///
/// # Parameters
/// - `itinerary_id`: The numeric ID of the itinerary to retrieve.
///
/// # Returns
/// - On success: The `Itinerary` object returned by the backend.
///   Events will already be sorted in the order they were saved.
/// - On failure: A null itinerary with a non-200 status code.
///
/// # Exceptions
/// Never throws an exception
export async function apiItineraryDetails(
	itinerary_id: number
): Promise<ApiResult<Itinerary>> {
	// TODO: get itinerary from cache if it exists
	try {
		const response = await customFetch(`${API_BASE_URL}/api/itinerary/${itinerary_id}`,
			{
				method: "GET",
				credentials: test_state.dev_mode ? "include" : "same-origin"
			}
		);
		return {
			result: response.ok ? await response.json() : null,
			status: response.status
		};
	} catch (error) {
		console.error("apiItineraryDetails error:", error);
		return { result: null, status: -1 };
	}
}

/// Saves or updates an itinerary for the authenticated user
///
/// # Method
/// Sends a `POST /api/itinerary/save` request to insert a new itinerary
/// or update an existing one in the database.
///
/// # Parameters
/// - `payload`: The complete `Itinerary` object to save. If the itinerary ID
///   already exists for this user, it will be updated. Otherwise, a new
///   itinerary will be created.
///
/// # Returns
/// - On success: A `SaveResponse` object containing the ID of the saved itinerary.
/// - On failure: Throws an error with details about the failure.
///
/// # Exceptions
/// Never throws an exception
export async function apiSaveItineraryChanges(
	payload: Itinerary
): Promise<ApiResult<SaveResponse>> {
	try {
		const response = await customFetch(`${API_BASE_URL}/api/itinerary/save`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: test_state.dev_mode ? "include" : "same-origin",
			body: JSON.stringify(payload)
		});

		if (!response.ok) {
			return { result: null, status: response.status };
		}

		return { result: await response.json(), status: response.status };
	} catch (error) {
		console.error("apiSaveItineraryChanges error:", error);
		return { result: null, status: -1 };
	}
}

/// Unsaves an existing itinerary for the authenticated user
///
/// # Method
/// Sends a `POST /api/itinerary/unsave` request to set the saved field
/// to false for the specified itinerary.
///
/// # Parameters
/// - `payload`: The itinerary id to unsave. The itinerary must
///   belong to the authenticated user.
///
/// # Returns
/// - On success: status code of 200
/// - On failure: non-200 status code
///
/// # Exceptions
/// Never throws an exception
export async function apiUnsaveItinerary(
	payload: UnsaveRequest
): Promise<ApiResult<void>> {
	try {
		const response = await customFetch(`${API_BASE_URL}/api/itinerary/unsave`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: test_state.dev_mode ? "include" : "same-origin",
			body: JSON.stringify(payload)
		});
		return { result: null, status: response.status };
	} catch (error) {
		console.error("apiUnsaveItinerary error:", error);
		return { result: null, status: -1 };
	}
}

/// Inserts or updates a user-created event
///
/// # Method
/// Sends a `POST /api/itinerary/userEvent` request to insert or
/// update a user-created event in the db
///
/// # Body
/// - `UserEventRequest`: If the id is specified, it will try to update that event.
/// Otherwise it will insert a new one. Event name is always required.
///
/// # Returns
/// - On success: The `UserEventResponse` object returned by the backend.
/// - On failure: A non-200 status code.
///
/// # Exceptions
/// Never throws an exception
export async function apiUserEvent(
	userEvent: UserEventRequest
): Promise<ApiResult<UserEventResponse>> {
	try {
		const response = await customFetch(`${API_BASE_URL}/api/itinerary/userEvent`,
			{
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				credentials: test_state.dev_mode ? "include" : "same-origin",
				body: JSON.stringify(userEvent)
			}
		);
		return {
			result: response.ok ? await response.json() : null,
			status: response.status
		};
	} catch (error) {
		console.error("apiUserEvent error:", error);
		return { result: null, status: -1 };
	}
}

/// Searches the db for events based on the search filters
///
/// # Method
/// Sends a `POST /api/itinerary/searchEvent` request to fetch a list
/// of events that match the query parameters.
///
/// # Body
/// - `SearchEventRequest`: Optional search parameters to query with.
///
/// # Returns
/// - On success: The `SearchEventResponse` object returned by the backend.
/// - On failure: A non-200 status code.
///
/// # Exceptions
/// Never throws an exception
export async function apiSearchEvent(
	query: SearchEventRequest
): Promise<ApiResult<SearchEventResponse>> {
	// TODO: get events from cache if possible
	try {
		const response = await customFetch(`${API_BASE_URL}/api/itinerary/searchEvent`,
			{
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				credentials: test_state.dev_mode ? "include" : "same-origin",
				body: JSON.stringify(query)
			}
		);
		return {
			result: response.ok ? await response.json() : null,
			status: response.status
		};
	} catch (error) {
		console.error("apiSearchEvent error:", error);
		return { result: null, status: -1 };
	}
}

/// Deletes a user-created event from the db
///
/// # Method
/// Sends a `DELETE /api/itinerary/userEvent/{id}` request to delete the event
/// with the provided id.
///
/// # Returns
/// - On success: 200 status code.
/// - On failure: A non-200 status code.
///
/// # Exceptions
/// Never throws an exception
export async function apiDeleteUserEvent(id: number): Promise<ApiResult<void>> {
	//TODO remove event from cache
	try {
		const response = await customFetch(`${API_BASE_URL}/api/itinerary/userEvent/${id}`,
			{
				method: "DELETE",
				credentials: test_state.dev_mode ? "include" : "same-origin"
			}
		);
		return { result: null, status: response.status };
	} catch (error) {
		console.error("apiDeleteUserEvent error:", error);
		return { result: null, status: -1 };
	}
}

/// Sends a `GET /api/itinerary/saved` request to fetch all saved itineraries.
///
/// # Returns list of saved itineraries if successful.
/// # Throws Error with message to be displayed.
export async function apiGetSavedItineraries(): Promise<
	ApiResult<SavedItinerariesResponse>
> {
	try {
		const response = await customFetch(`${API_BASE_URL}/api/itinerary/saved`, {
			method: "GET",
			credentials: "include"
		});
		return {
			result: response.ok ? await response.json() : null,
			status: response.status
		};
	} catch (error) {
		console.error("Get Saved Itineraries API error: ", error);
		return { result: null, status: -1 };
	}
}
