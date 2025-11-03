
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
	SignUpRequest
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
export async function apiLogin(payload: LoginRequest): Promise<ApiResult<void>> {
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
export async function apiSignUp(payload: SignUpRequest): Promise<ApiResult<void>> {
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
            headers: {
                "Content-Type": "application/json"
            },
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
            headers: {
                "Content-Type": "application/json"
            },
            credentials: test_state.dev_mode ? "include" : "same-origin"
        });
        return { result: null, status: response.status };
    } catch (error) {
        console.error("Validate API error: ", error);
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
			headers: {
				"Content-Type": "application/json"
			},
			credentials: test_state.dev_mode ? "include" : "same-origin"
		});
		if (!response.ok) {
			return { result: null, status: response.status };
		}
		return { result: await response.json(), status: response.status };
	} catch (error) {
		console.error("Current API error: ", error);
		return { result: null, status: -1 };
	}
}



import type { ChatsResponse } from "../src/models/chat";
import type {
	MessagePageRequest,
	MessagePageResponse,
	SendMessageRequest,
	SendMessageResponse
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
			headers: {
				"Content-Type": "application/json"
			},
			credentials: test_state.dev_mode ? "include" : "same-origin"
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
		return { result: await response.json(), status: response.status };
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
		return { result: await response.json(), status: response.status };
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
			headers: {
				"Content-Type": "application/json"
			},
			credentials: test_state.dev_mode ? "include" : "same-origin"
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



import type { Itinerary } from "../src/models/itinerary";

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
				headers: {
					"Content-Type": "application/json"
				},
				credentials: test_state.dev_mode ? "include" : "same-origin"
			}
		);
		return { result: await response.json(), status: response.status };
	} catch (error) {
		console.error("apiItineraryDetails error:", error);
		return { result: null, status: -1 };
	}
}
