import { useState, useEffect } from "react";
import ChatWindow from "../components/ChatWindow";
import PrevChatSideBar from "../components/PrevChatSideBar";
import Itinerary from "../components/Itinerary";
import Navbar from "../components/Navbar";
import "../styles/Home.css";
import { FinishAccountPopup } from "../components/FinishAccountPopup";
import {
  apiChats,
  apiMessages,
  apiNewChatId,
  apiSendMessage
} from "../api/home";
import type { MessagePageRequest, SendMessageRequest } from "../models/chat";
import type { ChatSession } from "../models/home";
import type { Message } from "../models/chat";
import { apiCurrent } from "../api/account";

export default function Home() {
  const [chats, setChats] = useState<ChatSession[] | null>(null);
  const [activeChatId, setActiveChatId] = useState<number | null>(null);
  const [showFinishPopup, setShowFinishPopup] = useState(false);
  const [selectedItineraryId, setSelectedItineraryId] = useState<number | null>(
    null
  );
  const [sidebarVisible, setSidebarVisible] = useState(true);
  const [firstName, setFirstName] = useState<string>("");

  useEffect(() => {
    async function fetchAccount() {
      if (showFinishPopup) {
        return;
      }
      const currentResult = await apiCurrent();
      // TODO 401 -> navigate to /login

      const account = currentResult.result;
      if (account === null || currentResult.status !== 200) {
        console.error(
          "API call to /api/account/current failed with status: ",
          currentResult.status
        );
        setShowFinishPopup(false);
        return; // TODO handle and display error
      }

      // Set the first name
      setFirstName(account.first_name || "");

      // check if any preferences were not yet filled out
      setShowFinishPopup(
        account.budget_preference === null ||
          account.disabilities === null ||
          account.food_allergies === null ||
          account.risk_preference === null
      );
    }

    async function fetchChats() {
      // get all chat session ids
      const chatsResult = await apiChats();
      // TODO: 401 -> navigate to /logout
      if (chatsResult.result === null || chatsResult.status !== 200) {
        return; // TODO handle and display error
      }

      const tempChats = chatsResult.result.chat_sessions.map((chat) => ({
        id: chat.id,
        title: chat.title,
        messages: [] // message loading handled at fetchMessagesForActiveChat
      }));

      if (activeChatId === null) {
        setChats(tempChats);
        return;
      }
      // get latest message page for this chat session
      const payload: MessagePageRequest = {
        chat_session_id: activeChatId,
        message_id: null
      };
      const messagePageResult = await apiMessages(payload);
      // TODO: 401 -> navigate to /logout

      if (
        messagePageResult.result === null ||
        messagePageResult.status !== 200
      ) {
        return; // TODO handle and display error
      }

      // TODO: use state for prev_message_id so you can fetch the next page when you scroll up

      const messages = messagePageResult.result.message_page;
      setChats(
        tempChats.map((c) =>
          c.id === activeChatId
            ? { ...c, messages: [...c.messages, ...messages] }
            : c
        )
      );
    }

    fetchAccount();
    fetchChats();
  }, [showFinishPopup, activeChatId]);

  const handleItinerarySelect = (itineraryId: number) => {
    setSelectedItineraryId(itineraryId);
  };

  const handleNewChat = async () => {
    // don't allow spamming new chats
    // instead, create the new chat once a message has been sent in it
    setActiveChatId(null);
  };

  const handleSendMessage = async (txt: string) => {
    const text = txt.trim();
    if (text === "") return;

    const userMessage: Message = {
      id: -1, //temporary id until the server gives us the real id
      text,
      is_user: true,
      timestamp: new Date().toISOString(),
      itinerary_id: null
    };

    let currChatId = activeChatId;

    // create a new chat if there is no active chat
    if (activeChatId === null) {
      const newChatResult = await apiNewChatId();
      // TODO: 401 -> navigate to /login

      if (newChatResult.result === null || newChatResult.status !== 200) {
        return; // TODO: handle and display error
      }

      const newChat: ChatSession = {
        id: newChatResult.result,
        messages: [],
        title: "New Chat"
      };

      setChats((prevChats) => [...(prevChats ?? []), newChat]);
      setActiveChatId(newChat.id);
      currChatId = newChat.id;
    }

    // The request might take some time, so we display the user message now,
    // then display the bot reply after we get it.
    setChats((prevChats) =>
      (prevChats ?? []).map((c) =>
        c.id === currChatId!
          ? { ...c, messages: [...c.messages, userMessage] }
          : c
      )
    );

    const payload: SendMessageRequest = {
      chat_session_id: currChatId!,
      text,
      itinerary_id: selectedItineraryId
    };

    const sendResult = await apiSendMessage(payload);
    // TODO: 401 -> navigate to /login
    if (sendResult.result === null || sendResult.status !== 200) {
      return; // TODO: handle and display error
    }

    // Thanks, React, for making this the convention for updating state
    setChats((prevChats) =>
      (prevChats ?? []).map((c) =>
        c.id === currChatId!
          ? {
              ...c,
              messages: c.messages
                .map((m) =>
                  m.id === -1
                    ? { ...m, id: sendResult.result!.user_message_id }
                    : m
                )
                .concat([sendResult.result!.bot_message])
            }
          : c
      )
    );
  };

  const handleToggleSidebar = () => {
    setSidebarVisible((prev) => !prev);
  };

  const activeChat = chats?.find((c) => c.id === activeChatId) ?? null;

  return (
    <div className="home-page">
      <Navbar page="home" firstName={firstName} />
      <div className={`home-layout ${sidebarVisible ? "with-sidebar" : "no-sidebar"}`}>
        {showFinishPopup && <FinishAccountPopup />}

        <PrevChatSideBar
          chats={chats}
          activeChatId={activeChatId}
          onSelectChat={setActiveChatId}
          onNewChat={handleNewChat}
          onToggleSidebar={handleToggleSidebar}
          sidebarVisible={sidebarVisible}
        />

        <div className="chat-window-wrapper">
          <ChatWindow
            messages={activeChat?.messages ?? []}
            onSend={handleSendMessage}
            onItinerarySelect={handleItinerarySelect}
          />
        </div>

        <div className="itinerary-wrapper">
          <Itinerary />
        </div>
      </div>
    </div>
  );
}