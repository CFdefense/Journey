import { useState, useEffect } from "react";
import ChatWindow from "../components/ChatWindow";
import PrevChatSideBar from "../components/PrevChatSideBar";
import Itinerary from "../components/Itinerary";
import "../styles/Home.css";
import { FinishAccountPopup } from "../components/FinishAccountPopup";
import { apiCheckIfPreferencesPopulated } from "../api/account";
import { apiChats, apiMessages, apiNewChatId } from "../api/home";
import type { MessagePageRequest, MessagePageResponse } from "../models/chat";
import { apiItineraryDetails } from "../api/itinerary"; 
import { handleMessageSendExistingChat, handleMessageSendNewChat } from "../helpers/home";
import type { Message, ChatSession } from "../models/home";

export default function Home() {
  const [chats, setChats] = useState<ChatSession[]>([]);
  const [activeChatId, setActiveChatId] = useState<number | null>(null);
  const [showFinishPopup, setShowFinishPopup] = useState(false);
  const [itineraryTitles, setItineraryTitles] = useState<Record<number, string>>({});
  const [selectedItineraryId, setSelectedItineraryId] = useState<number | null>(null);

  const handleItinerarySelect = (itineraryId: number) => {
    setSelectedItineraryId(itineraryId);
  };

  //  Fetch user preferences 
  useEffect(() => {
    async function fetchPreferences() {
      try {
        const preferencesFilled = await apiCheckIfPreferencesPopulated();
        setShowFinishPopup(!preferencesFilled);
      } catch {
        setShowFinishPopup(true);
      }
    }
    fetchPreferences();
  }, []);

  //  Fetch chats and messages 
  useEffect(() => {
    async function fetchChatsAndMessages() {
      try {
        const chatData = await apiChats();

        const initialChats: ChatSession[] = chatData.chat_sessions.map((id, i) => ({
          id,
          title: `Chat ${i + 1}`,
          messages: [],
        }));

        const chatsWithMessages = await Promise.all(
          initialChats.map(async (chat) => {
            const payload: MessagePageRequest = {
              chat_session_id: chat.id,
              message_id: null,
            };

            const messagePage: MessagePageResponse = await apiMessages(payload);

            const messages: Message[] = messagePage.message_page.map((msg) => ({
              id: msg.id,
              text: msg.text,
              sender: msg.is_user ? "user" : "bot",
              itinerary_id: msg.itinerary_id, 
            }));

            // 
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

            return { ...chat, messages };
          })
        );

        setChats(chatsWithMessages);
      } catch (err) {
        console.error("Error fetching chats:", err);
      }
    }

    fetchChatsAndMessages();
  }, []);

  // clicking the button opens up a blank chatwindow, but a new chat is not created until a user sends in a message
  const createNewChatFromButton = async () => {
    try {
      const newChatId = await apiNewChatId();
      if (newChatId === -1) return;

      const blankChat: ChatSession = {
        id: newChatId,
        title: `Chat ${chats.length + 1}`,
        messages: [],
      };

      setChats((prev) => [...prev, blankChat]);
      setActiveChatId(newChatId);
    } catch (err) {
      console.error("Error creating chat:", err);
    }
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
          onNewChat={createNewChatFromButton}
        />

        <ChatWindow
          messages={activeChat?.messages || []}
          onSend={handleSendMessage}
          itineraryTitles={itineraryTitles}
          onItinerarySelect={handleItinerarySelect} // 👈 pass handler
        />
        
        <Itinerary />
      </div>
    </div>
  );
}
