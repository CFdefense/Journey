import { useState, useEffect } from "react";
import ChatWindow from "../components/ChatWindow";
import PrevChatSideBar from "../components/PrevChatSideBar";
import Itinerary from "../components/Itinerary";
import "../styles/Home.css";
import { FinishAccountPopup } from "../components/FinishAccountPopup";
import { apiCheckIfPreferencesPopulated } from "../api/account";
import { apiChats } from "../api/home";
import { apiMessages } from "../api/home";
import type { MessagePageRequest } from "../models/chat";
import type { MessagePageResponse } from "../models/chat";


 

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
  //TODO use this to determine which itinerary to show
  const [activeChatId, setActiveChatId] = useState<number | null>(null);

  const [showFinishPopup, setShowFinishPopup] = useState(false);

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
const handleSendMessage = (text: string) => {
  if (!text.trim()) return;

  // If there are no chats yet, create a new chat including this first message
  // When the user makes a chat for the first time, they do not have to click, new chat, they just have to start chatting
  if (chats.length === 0) {
    const userMessage: Message = {
      id: Date.now(),
      text,
      sender: "user",
    };

    const botMessage: Message = {
      id: Date.now() + 1,
      text: "bot reply",
      sender: "bot",
    };

    const newChat: ChatSession = {
      id: Date.now(),
      title: `Chat 1`,
      messages: [userMessage, botMessage], // include the first message
    };

    setChats([newChat]);
    setActiveChatId(newChat.id);
    return; // message already added, exit
  }

  // There are existing chats, add to the active chat
  if (activeChatId === null) return;

  setChats((prevChats) =>
    prevChats.map((chat) => {
      if (chat.id !== activeChatId) return chat;

      const userMessage: Message = {
        id: Date.now(),
        text,
        sender: "user",
      };

      const botMessage: Message = {
        id: Date.now() + 1,
        text: "bot reply",
        sender: "bot",
      };

      return {
        ...chat,
        messages: [...chat.messages, userMessage, botMessage],
      };
    })
  );
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
