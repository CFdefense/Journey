import type { ChatSession, Message } from "../models/home";
import type { SendMessageRequest, SendMessageResponse } from "../models/chat";
import { apiSendMessage } from "../api/home";

/**
 * Handles sending a message in an existing chat session.
 * Freezes chatId so messages always go to the correct chat.
 */
export async function handleMessageSendExistingChat(
  text: string,
  chatId: number, // explicitly pass the chat ID
  setChats: React.Dispatch<React.SetStateAction<ChatSession[]>>
) {
  const targetChatId = chatId; // freeze chat ID

  const userMessage: Message = {
    id: Date.now(),
    text,
    sender: "user",
  };

  // Optimistically add the user message
  setChats((prevChats) =>
    prevChats.map((chat) =>
      chat.id === targetChatId
        ? { ...chat, messages: [...chat.messages, userMessage] }
        : chat
    )
  );

  try {
    const payload: SendMessageRequest = {
      chat_session_id: targetChatId,
      text,
    };

    const response: SendMessageResponse = await apiSendMessage(payload);

    const botMessage: Message = {
      id: response.bot_message.id,
      text: response.bot_message.text,
      sender: response.bot_message.is_user ? "user" : "bot",
    };

    // Append bot message to the correct chat
    setChats((prevChats) =>
      prevChats.map((chat) =>
        chat.id === targetChatId
          ? { ...chat, messages: [...chat.messages, botMessage] }
          : chat
      )
    );
  } catch (err) {
    console.error("Error sending message:", err);
  }
}

/**
 * Handles sending the very first message when the user has no chats yet.
 * Freezes the new chat ID so messages don't leak between chats.
 */
export async function handleMessageSendNewChat(
  text: string,
  chats: ChatSession[],
  setChats: React.Dispatch<React.SetStateAction<ChatSession[]>>,
  setActiveChatId: React.Dispatch<React.SetStateAction<number | null>>
) {
  const newChatId = Date.now();
  const targetChatId = newChatId; // freeze ID for API call

  const userMessage: Message = {
    id: targetChatId,
    text,
    sender: "user",
  };

  // Create new chat locally
  const newChat: ChatSession = {
    id: targetChatId,
    title: `Chat ${chats.length + 1 || 1}`,
    messages: [userMessage],
  };

  setChats([newChat]);
  setActiveChatId(targetChatId);

  try {
    const payload: SendMessageRequest = {
      chat_session_id: targetChatId,
      text,
    };

    const response: SendMessageResponse = await apiSendMessage(payload);

    const botMessage: Message = {
      id: response.bot_message.id,
      text: response.bot_message.text,
      sender: response.bot_message.is_user ? "user" : "bot",
    };

    // Update the new chat with bot message
    setChats((prevChats) =>
      prevChats.map((chat) =>
        chat.id === targetChatId
          ? { ...chat, messages: [...chat.messages, botMessage] }
          : chat
      )
    );
  } catch (err) {
    console.error("Error sending first message:", err);
  }
}
