import { useState } from "react";
import ChatWindow from "../components/ChatWindow";
import PrevChatSideBar from "../components/PrevChatSideBar";
import Itinerary from "../components/Itinerary";
import "../styles/Home.css";

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
