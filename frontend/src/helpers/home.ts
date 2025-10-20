import type { ChatSession, Message } from "../models/home";
import type { SendMessageRequest, SendMessageResponse } from "../models/chat";
import { apiSendMessage, apiNewChatId } from "../api/home";
import { apiItineraryDetails } from "../api/itinerary"; 


/**
 * Handles sending a message in an existing chat session.
 */
export async function handleMessageSendExistingChat(
  text: string,
  chatId: number,
  setChats: React.Dispatch<React.SetStateAction<ChatSession[]>>,
  setItineraryTitles: React.Dispatch<React.SetStateAction<Record<number, string>>>

) {
  const userMessage: Message = {
    id: Date.now(), 
    text,
    sender: "user",
    itinerary_id: null, 
  };

  // Optimistically add the user message
  setChats((prevChats) =>
    prevChats.map((chat) =>
      chat.id === chatId
        ? { ...chat, messages: [...chat.messages, userMessage] }
        : chat
    )
  );

  try {
    const payload: SendMessageRequest = {
      chat_session_id: chatId,
      text,
    };

    const response: SendMessageResponse = await apiSendMessage(payload);
    console.log(response.bot_message.itinerary_id)

    const botMessage: Message = {
      id: response.bot_message.id,
      text: response.bot_message.text,
      sender: response.bot_message.is_user ? "user" : "bot",
      itinerary_id: response.bot_message.itinerary_id ?? null, 
    };

    // get the itinerary information before displaying the messages
    if (botMessage.itinerary_id) {
      const itinerary = await apiItineraryDetails(botMessage.itinerary_id);
      setItineraryTitles((prev) => ({
        ...prev,
        [botMessage.itinerary_id!]: itinerary.title,
      }));
    }

    // Append bot message to the correct chat
    setChats((prevChats) =>
      prevChats.map((chat) =>
        chat.id === chatId
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
 */
export async function handleMessageSendNewChat(
  text: string,
  chats: ChatSession[],
  setChats: React.Dispatch<React.SetStateAction<ChatSession[]>>,
  setActiveChatId: React.Dispatch<React.SetStateAction<number | null>>,
  setItineraryTitles: React.Dispatch<React.SetStateAction<Record<number, string>>>

) {
  // get the chat session id from the backend
  const newChatId = await apiNewChatId();
  console.log("New chat session ID:", newChatId);

  if (newChatId === -1) {
    console.error("Failed to create new chat session");
    return;
  }

  const userMessage: Message = {
    id: Date.now(),
    text,
    sender: "user",
    itinerary_id: null, 
  };

  // Create a new chat locally
  const newChat: ChatSession = {
    id: newChatId,
    title: `Chat ${chats.length + 1 || 1}`,
    messages: [userMessage],
  };

  // Add to chat list and make it active
  setChats((prev) => [...prev, newChat]);
  setActiveChatId(newChatId);

  try {
    const payload: SendMessageRequest = {
      chat_session_id: newChatId,
      text,
    };

    const response: SendMessageResponse = await apiSendMessage(payload);
    console.log(response.bot_message.itinerary_id)

    const botMessage: Message = {
      id: response.bot_message.id,
      text: response.bot_message.text,
      sender: response.bot_message.is_user ? "user" : "bot",
      itinerary_id: response.bot_message.itinerary_id ?? null, 
    };

    // get the itinerary information before displaying the messages
    if (botMessage.itinerary_id) {
      const itinerary = await apiItineraryDetails(botMessage.itinerary_id);
      setItineraryTitles((prev) => ({
        ...prev,
        [botMessage.itinerary_id!]: itinerary.title,
      }));
    }

    // Update the same chat with the bot response
    setChats((prevChats) =>
      prevChats.map((chat) =>
        chat.id === newChatId
          ? { ...chat, messages: [...chat.messages, botMessage] }
          : chat
      )
    );
  } catch (err) {
    console.error("Error sending first message:", err);
  }
}
