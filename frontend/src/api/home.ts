const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;
import type { ChatsResponse } from "../models/chat";
import type { MessagePageRequest } from "../models/chat";
import type { MessagePageResponse } from "../models/chat";



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




