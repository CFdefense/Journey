const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;
import type { ChatsResponse } from "../models/chat";
import type { MessagePageRequest, MessagePageResponse, SendMessageRequest, SendMessageResponse } from "../models/chat";


/// Calls chats
///
/// # Method
/// Sends a `GET /api/chat/chats` request to fetch all chat sessions for the current user.
///
/// # Returns
/// - On success: `ChatsResponse` containing all existing chat sessions.
/// - On failure: An empty `ChatsResponse` with no chat sessions.
///
/// # Errors
/// - Logs a warning if the response is non-OK.
/// - Returns an empty list if there is a network or parsing error.
export async function apiChats(): Promise<ChatsResponse> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/chat/chats`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: import.meta.env.DEV ? "include" : "same-origin",
    });

    // TODO add the specific error handling when documentation on backend is updated
    if (!response.ok) {
      console.warn("Non-OK response from /api/chat/chats:", response.status);
      return { chat_sessions: [] }; // return and empty list if there are issues
    }

    const data = await response.json();

    return {
      chat_sessions: data.chat_sessions ?? [], // if data.chat_sessions is null or undefined, just return an empty array
    };
  } catch (error) {
    console.error("apiChats error:", error);
    return { chat_sessions: [] }; 
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
/// - On failure: Returns an empty `MessagePageResponse` with no messages.
///
/// # Errors
/// - Logs warnings for non-OK or network errors.
/// - Falls back to an empty message list on failure.
export async function apiMessages(payload: MessagePageRequest): Promise<MessagePageResponse> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/chat/messagePage`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: import.meta.env.DEV ? "include" : "same-origin",
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      console.warn("Non-OK response from /api/chat/messagePage:", response.status);
      // Return empty fallback on server error
      return {
        message_page: [],
        prev_message_id: null,
      };
    }

    const data = await response.json();

    return {
      message_page: data.message_page ?? [],
      prev_message_id: data.prev_message_id ?? null,
    };
  } catch (error) {
    console.error("apiMessages error:", error);
    // Return empty fallback on network or parsing errors
    return {
      message_page: [],
      prev_message_id: null,
    };
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
/// - On failure: Returns a fallback object containing an error message.
///
/// # Errors
/// - Logs warnings for non-OK or network failures.
/// - Returns placeholder error text in the bot message when the request fails.
export async function apiSendMessage(payload: SendMessageRequest): Promise<SendMessageResponse> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/chat/sendMessage`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: import.meta.env.DEV ? "include" : "same-origin",
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      console.warn("Non-OK response from /api/chat/sendMessage:", response.status);
      // Return placeholder fallback on server error
      return {
        user_message_id: -1, // no ids in the db will be negative
        bot_message: {
          id: -1,
          is_user: false,
          timestamp: new Date().toISOString(),
          text: "Error: could not send message",
          itinerary_id: null,
        },
      };
    }

    const data = await response.json();

    // Convert backend response to SendMessageResponse shape
    return {
      user_message_id: data.user_message_id ?? -1,
      bot_message: {
        id: data.bot_message?.id ?? -1,
        is_user: data.bot_message?.is_user ?? false,
        timestamp: data.bot_message?.timestamp ?? new Date().toISOString(),
        text: data.bot_message?.text ?? "",
        itinerary_id: data.bot_message?.itinerary_id ?? null,
      },
    };
  } catch (error) {
    console.error("apiSendMessage error:", error);
    // Return placeholder fallback on network or parsing error
    return {
      user_message_id: -1,
      bot_message: {
        id: -1,
        is_user: false,
        timestamp: new Date().toISOString(),
        text: "Error: network or parsing failure",
        itinerary_id: null,
      },
    };
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
/// - On failure: `-1` if the request fails or no chat session ID is returned.
///
/// # Errors
/// - Logs a warning for non-OK responses.
/// - Logs an error for network or parsing failures.
export async function apiNewChatId(): Promise<number> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/chat/newChat`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: import.meta.env.DEV ? "include" : "same-origin",
    });

    // TODO add the specific error handling when documentation on backend is updated
    if (!response.ok) {
      console.warn("Non-OK response from /api/chat/newChat:", response.status);
      return -1; // -1 if there is an error
    }

    const data = await response.json();
    
    return data.chat_session_id ?? -1;
    
  } catch (error) {
    console.error("apiChats error:", error);
    return -1; 
  }
}





