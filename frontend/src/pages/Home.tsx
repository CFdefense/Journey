import { useState, useEffect } from "react";
import ChatWindow from "../components/ChatWindow";
import PrevChatSideBar from "../components/PrevChatSideBar";
import Itinerary from "../components/Itinerary";
import "../styles/Home.css";
import { FinishAccountPopup } from "../components/FinishAccountPopup";
import { apiChats, apiMessages } from "../api/home";
import type { MessagePageRequest, MessagePageResponse } from "../models/chat";
import { apiItineraryDetails } from "../api/itinerary"; 
import { apiCheckIfPreferencesPopulated } from "../api/account";
import { handleMessageSendExistingChat, handleMessageSendNewChat, createNewChat } from "../helpers/home";
import type { ChatSession } from "../models/home";
import type { Message } from "../models/chat";

export default function Home() {
  const [chats, setChats] = useState<ChatSession[]>([]);
  const [activeChatId, setActiveChatId] = useState<number | null>(null);
  const [showFinishPopup, setShowFinishPopup] = useState(false);
  const [itineraryTitles, setItineraryTitles] = useState<Record<number, string>>({});
  const [selectedItineraryId, setSelectedItineraryId] = useState<number | null>(null);

  void selectedItineraryId;
 // this is to prevent a build error. the value will be used later but not in this pr
  // TODO, build this out and move to helper/home.ts
  const handleItinerarySelect = (itineraryId: number) => {
    setSelectedItineraryId(itineraryId);
  };

  //fetch userPreferences
  useEffect(() => { 
    async function fetchPreferences() { 
      try { 
        const preferencesFilled = await apiCheckIfPreferencesPopulated(); 
        setShowFinishPopup(!preferencesFilled); 
      } catch { 
        setShowFinishPopup(true); 
      } 
    } fetchPreferences(); 
  }, []);


  // fetch all of the chatSessions from the db
  useEffect(() => {
  async function fetchChats() {
    try {
      // get the list of chat session ids
      const chatData = await apiChats();

      const chatList: ChatSession[] = chatData.chat_sessions.map((id, i) => ({
        id,
        title: `Chat ${i + 1}`,
        messages: [], // message loading handled at fetchMessagesForActiveChat
      }));

      setChats(chatList);

    } catch (err) {
      console.error("Error fetching chats:", err);
    }
  }

  fetchChats();
}, []);

// fetch message pages only for the current active chat
useEffect(() => {
  async function fetchMessagesForActiveChat() {
    if (activeChatId === null) return;

    try {
      const payload: MessagePageRequest = {
        chat_session_id: activeChatId,
        message_id: null,
      };

      const messagePage: MessagePageResponse = await apiMessages(payload);
      const messages: Message[] = messagePage.message_page;

      // get the itinerary titles for only messages in this chat session
      for (const msg of messages) {
        if (msg.itinerary_id && !itineraryTitles[msg.itinerary_id]) {
          apiItineraryDetails(msg.itinerary_id).then((it) => {
            setItineraryTitles((prev) => ({
              ...prev,
              [msg.itinerary_id!]: it.title,
            }));
          });
        }
      }

      // update messages for the selected chat
      setChats((prevChats) =>
        prevChats.map((chat) =>
          chat.id === activeChatId ? { ...chat, messages } : chat
        )
      );
    } catch (err) {
      console.error(`Error fetching messages for chat ${activeChatId}:`, err);
    }
  }

  fetchMessagesForActiveChat();
}, [activeChatId]);


  // creates a new chat
  const handleNewChat = async () => {
    await createNewChat(chats, setChats, setActiveChatId);
  };

  // utlizes functions in helper/home to deal with sending a chat properly
  const handleSendMessage = async (text: string) => {
    if (!text.trim()) return;
    const isNewChat = chats.length === 0 || activeChatId === null;
    if (isNewChat) {
      await handleMessageSendNewChat(text, chats, setChats, setActiveChatId, setItineraryTitles);
    } else {
      await handleMessageSendExistingChat(text, activeChatId, setChats, setItineraryTitles);
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

        <ChatWindow
          messages={activeChat?.messages || []}
          onSend={handleSendMessage}
          itineraryTitles={itineraryTitles}
          onItinerarySelect={handleItinerarySelect} 
        />
        
        <Itinerary />
      </div>
    </div>
  );
}
