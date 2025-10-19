import { useState, useEffect } from "react";
import ChatWindow from "../components/ChatWindow";
import PrevChatSideBar from "../components/PrevChatSideBar";
import Itinerary from "../components/Itinerary";
import "../styles/Home.css";
import { FinishAccountPopup } from "../components/FinishAccountPopup";
import { apiCheckIfPreferencesPopulated } from "../api/account";
import { apiChats, apiMessages, apiSendMessage } from "../api/home";
import type { MessagePageRequest, MessagePageResponse, SendMessageRequest, SendMessageResponse } from "../models/chat";


interface Message {
  id: number;
  text: string;
  sender: "user" | "bot";
}

interface ChatSession {
  id: number;
  title: string;
  messages: Message[];
}

export default function Home() {
  //array of chat sessions. each chat session has an id, title, and list of messages
  const [chats, setChats] = useState<ChatSession[]>([]);
  // current chat in window 
  const [activeChatId, setActiveChatId] = useState<number | null>(null);
  // the finish account creation 
  const [showFinishPopup, setShowFinishPopup] = useState(false);
  // which itinerary is shown in chat
  const [displayedItinerary, setDisplayedItinerary] = useState(-1);

    // check if preferences are filled out for a user
    useEffect(() => {
      async function fetchPreferences() {
        try {
          const preferencesFilled = await apiCheckIfPreferencesPopulated();
          console.log("Preferences filled:", preferencesFilled);
          setShowFinishPopup(!preferencesFilled); // show only if false
        } catch (err) {
          console.error("Error calling apiCheckIfPreferencesPopulated:", err);
          setShowFinishPopup(true); // any auth error just show the popup 
        }
      }

      fetchPreferences();
    }, []);

    useEffect(() => {
      async function fetchChatsAndMessages() {
      try {
        const chatData = await apiChats();
        console.log("Fetched chats:", chatData);

        // get the chat id, set a title, and initialize empty array for messages later
        const initialChats: ChatSession[] = chatData.chat_sessions.map((id, i) => ({
          id,
          title: `Chat ${i + 1}`,
          messages: [],
        }));

        // fetch all of the chats in parallel
        const chatsWithMessages = await Promise.all(
        initialChats.map(async (chat) => {
          const payload: MessagePageRequest = {
            chat_session_id: chat.id,
            message_id: null, 
          };

          const messagePage: MessagePageResponse = await apiMessages(payload);

          console.log("Message page for chat ID", chat.id, ":", messagePage); // log message page and chatId for debugging for now


          // convert the backend message into the format the front end is expecting
          const messages: Message[] = messagePage.message_page.map((msg) => ({
            id: msg.id,
            text: msg.text,
            sender: msg.is_user ? "user" : "bot",
          }));

          return {
            ...chat, // call all properties of chat into a new object (id, text, sender)
            messages, // adds the messages to the previously blank section of initialChats
          };
        })
      );

      setChats(chatsWithMessages);

      } catch (err) {
        console.error("Error fetching chats:", err);
      }
    }

  fetchChatsAndMessages();
}, []);

  // Create a new blank chat
  const handleNewChat = () => {
    const newChat: ChatSession = {
      id: Date.now(),
      title: `Chat ${chats.length + 1}`,
      messages: [],
    };
    setChats((prev) => [...prev, newChat]);
    setActiveChatId(newChat.id);
  };

  // Handle sending a message
  const handleSendMessage = async (text: string) => {
    if (!text.trim()) return;

    // If there are no chats yet, create a new chat first
    if (chats.length === 0) {
      const newChatId = Date.now(); // temporary ID for new chat

      const userMessage: Message = {
        id: newChatId,
        text,
        sender: "user",
      };

      // Add the new chat locally first
      const newChat: ChatSession = {
        id: newChatId,
        title: `Chat 1`,
        messages: [userMessage], // only the user message for now
      };
      setChats([newChat]);
      setActiveChatId(newChatId);

      try {
        const payload: SendMessageRequest = {
          chat_session_id: newChatId,
          text,
        };
        
      
        const response: SendMessageResponse = await apiSendMessage(payload);

        // Convert backend bot message to frontend format
        const botMessage: Message = {
          id: response.bot_message.id,
          text: response.bot_message.text,
          sender: response.bot_message.is_user ? "user" : "bot",
        };

        // Update chat to include bot message
        setChats((prevChats) =>
          prevChats.map((chat) =>
            chat.id === newChatId
              ? { ...chat, messages: [...chat.messages, botMessage] }
              : chat
          )
        );
      } catch (err) {
        console.error("Error sending message:", err);
      }

      return;
    }

  // If chat exists, append messages to active chat
  if (activeChatId === null) return;

  const userMessage: Message = {
    id: Date.now(),
    text,
    sender: "user",
  };

  // Optimistically add the user message. Avoids waiting for it to load from back end, it will never be different. 
  setChats((prevChats) =>
    prevChats.map((chat) =>
      chat.id === activeChatId
        ? { ...chat, messages: [...chat.messages, userMessage] }
        : chat
    )
  );

  try {
    const payload: SendMessageRequest = {
      chat_session_id: activeChatId,
      text,
    };
    const response: SendMessageResponse = await apiSendMessage(payload);

    const botMessage: Message = {
      id: response.bot_message.id,
      text: response.bot_message.text,
      sender: response.bot_message.is_user ? "user" : "bot",
    };

    // Append the bot message after API returns
    setChats((prevChats) =>
      prevChats.map((chat) =>
        chat.id === activeChatId
          ? { ...chat, messages: [...chat.messages, botMessage] }
          : chat
      )
    );

  } catch (err) {
    console.error("Error sending message:", err);
  }
};

  const activeChat = chats.find((c) => c.id === activeChatId) || null;

  return (
    <div className="home-page">
    <h1>Where do you plan to explore?</h1>
    <div className="home-layout">
      {showFinishPopup && <FinishAccountPopup />}
      <PrevChatSideBar
        chats={chats}
        activeChatId={activeChatId}
        onSelectChat={setActiveChatId}
        onNewChat={handleNewChat}
      />
      <ChatWindow messages={activeChat?.messages || []} onSend={handleSendMessage} />
      <Itinerary />
    </div>
  </div>
  );
}
