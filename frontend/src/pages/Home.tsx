import { useState, useEffect } from "react";
import ChatWindow from "../components/ChatWindow";
import PrevChatSideBar from "../components/PrevChatSideBar";
import Itinerary from "../components/Itinerary";
import "../styles/Home.css";
import { FinishAccountPopup } from "../components/FinishAccountPopup";
import { apiCheckIfPreferencesPopulated } from "../api/account";
import { apiChats, apiMessages, apiNewChatId } from "../api/home";
import type { MessagePageRequest, MessagePageResponse, SendMessageRequest, SendMessageResponse } from "../models/chat";
import { apiItineraryDetails } from "../api/itinerary"; 
import { handleMessageSendExistingChat, handleMessageSendNewChat, } from "../helpers/home";
import type { Message, ChatSession } from "../models/home";


export default function Home() {
  //array of chat sessions. each chat session has an id, title, and list of messages
  const [chats, setChats] = useState<ChatSession[]>([]);
  // current chat in window 
  const [activeChatId, setActiveChatId] = useState<number | null>(null);
  // the finish account creation 
  const [showFinishPopup, setShowFinishPopup] = useState(false);
  // which itinerary is shown in chat
  const [displayedItinerary, setDisplayedItinerary] = useState(3); // testing a known itinerary

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

useEffect(() => {
  if (displayedItinerary === -1) return;
  async function fetchItinerary() {
    try {
      const itinerary = await apiItineraryDetails(displayedItinerary);
      console.log("Fetched itinerary:", itinerary);
    } catch (err) {
      console.error("Error fetching itinerary:", err);
    }
  }

  fetchItinerary();
}, [displayedItinerary]);



// Create a new blank chat from the button and make it have a chatSessionId on the backend too
const createNewChatFromButton = async () => {
  try {
    const newChatId = await apiNewChatId();

    console.log(apiChats()); 

    if (newChatId === -1) {
      console.error("Failed to create new chat session");
      return;
    }

    const newChat: ChatSession = {
      id: newChatId,
      title: `Chat ${chats.length + 1 || 1}`,
      messages: [],
    };

    setChats((prev) => [...prev, newChat]);
    setActiveChatId(newChatId);
  } catch (err) {
    console.error("Error creating new chat:", err);
  }
};

  // this logic is handled in /helpers/home.ts
  const handleSendMessage = async (text: string) => {
    if (!text.trim()) return;

    // Determine which chat we are sending to
    // If no chats exist, or no chat is selected, create a new chat
    const isNewChat = chats.length === 0 || activeChatId === null;
    console.log(isNewChat)
    
    if (isNewChat) {
      await handleMessageSendNewChat(text, chats, setChats, setActiveChatId);
    } else {
      await handleMessageSendExistingChat(text, activeChatId, setChats);
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
      <ChatWindow messages={activeChat?.messages || []} onSend={handleSendMessage} />
      <Itinerary />
    </div>
  </div>
  );
}
