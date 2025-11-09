// Home.tsx
import { useState, useEffect } from "react";
import ChatWindow from "../components/ChatWindow";
import PrevChatSideBar from "../components/PrevChatSideBar";
import ItinerarySideBar from "../components/ItinerarySideBar";
import Navbar from "../components/Navbar";
import "../styles/Home.css";
import { FinishAccountPopup } from "../components/FinishAccountPopup";
import {
  apiChats,
  apiMessages,
  apiNewChatId,
  apiSendMessage,
  apiUpdateMessage
} from "../api/home";
import type {
  MessagePageRequest,
  SendMessageRequest,
  UpdateMessageRequest
} from "../models/chat";
import type { ChatSession } from "../models/home";
import type { Message } from "../models/chat";
import { apiCurrent } from "../api/account";
import { fetchItinerary } from "../helpers/itinerary";
import type { DayItinerary } from "../helpers/itinerary";
import { apiItineraryDetails } from "../api/itinerary";

export const ACTIVE_CHAT_SESSION: string = "activeChatSession";

export default function Home() {
  const [chats, setChats] = useState<ChatSession[] | null>(null);
  const [activeChatId, setActiveChatId] = useState<number | null>(null);
  const [showFinishPopup, setShowFinishPopup] = useState(false);
  const [selectedItineraryId, setSelectedItineraryId] = useState<number | null>(
    null
  );
  const [sidebarVisible, setSidebarVisible] = useState(true);
  const [itinerarySidebarVisible, setItinerarySidebarVisible] = useState(false);
  const [firstName, setFirstName] = useState<string>("");
  const [itineraryData, setItineraryData] = useState<DayItinerary[] | null>(
    null
  );
  const [itineraryTitle, setItineraryTitle] = useState<string>("");

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

      let chatSessionId = activeChatId;

      // get MRU chat from session storage
      const prevActiveChat: string | null =
        sessionStorage.getItem(ACTIVE_CHAT_SESSION);
      if (prevActiveChat !== null) {
        const id = +prevActiveChat;
        if (tempChats.find((chat) => chat.id === id) !== undefined) {
          chatSessionId = id;
          setActiveChatId(chatSessionId);
        }
      }

      if (chatSessionId === null) {
        setChats(tempChats);
        return;
      }
      // get latest message page for this chat session
      const payload: MessagePageRequest = {
        chat_session_id: chatSessionId,
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
          c.id === chatSessionId
            ? { ...c, messages: [...c.messages, ...messages] }
            : c
        )
      );
    }

    fetchAccount();
    fetchChats();
  }, [showFinishPopup, activeChatId, setActiveChatId]);

  // Fetch itinerary data when selectedItineraryId changes
  useEffect(() => {
    async function loadItinerary() {
      const itineraryId = selectedItineraryId;

      // if no current itinerary is selected, do not try and populate it
      if (itineraryId === null) {
        return;
      }

      try {
        const data = await fetchItinerary(itineraryId);
        setItineraryData(data);

        const apiResponse = await apiItineraryDetails(itineraryId);

        if (apiResponse.result) {
          setItineraryTitle(apiResponse.result.title);
        }
      } catch (error) {
        console.error("Error loading itinerary:", error);
        setItineraryData(null);
        setItineraryTitle("");
      }
    }

    loadItinerary();
  }, [selectedItineraryId]);

  // whenever the active chat changes, clear all itinerary information on home page.
  useEffect(() => {
    setSelectedItineraryId(null);
    setItineraryData(null);
    setItineraryTitle("");
    setItinerarySidebarVisible(false);
  }, [activeChatId]);

  const handleItinerarySelect = (itineraryId: number) => {
    setSelectedItineraryId(itineraryId);
    setItinerarySidebarVisible(true); // when an itinerary is selected, make sure the itinerary side bar also opens
  };

  const handleNewChat = async () => {
    // don't allow spamming new chats
    // instead, create the new chat once a message has been sent in it
    sessionStorage.removeItem(ACTIVE_CHAT_SESSION);
    setActiveChatId(null);
    setItinerarySidebarVisible(false);
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
      sessionStorage.setItem(ACTIVE_CHAT_SESSION, newChat.id.toString());
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

    const botMessage = sendResult.result!.bot_message;

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

    if (botMessage.itinerary_id !== null) {
      setSelectedItineraryId(botMessage.itinerary_id);
      setItinerarySidebarVisible(true);
    }
  };

  const handleEditMessage = async (messageId: number, newText: string) => {
    if (activeChatId === null) return;

    const payload: UpdateMessageRequest = {
      message_id: messageId,
      new_text: newText,
      itinerary_id: selectedItineraryId
    };

    // Optimistically update the UI - update the message text
    setChats((prevChats) =>
      (prevChats ?? []).map((c) =>
        c.id === activeChatId
          ? {
              ...c,
              messages: c.messages.map((m) =>
                m.id === messageId
                  ? { ...m, text: newText, timestamp: new Date().toISOString() }
                  : m
              )
            }
          : c
      )
    );

    const updateResult = await apiUpdateMessage(payload);
    // TODO: 401 -> navigate to /login

    if (updateResult.result === null || updateResult.status !== 200) {
      // TODO: handle and display error, revert optimistic update
      return;
    }

    const botMessage = updateResult.result;

    // Remove all messages after the edited message, then add the new bot response
    setChats((prevChats) =>
      (prevChats ?? []).map((c) => {
        if (c.id !== activeChatId) return c;

        const editedMessageIndex = c.messages.findIndex(
          (m) => m.id === messageId
        );
        if (editedMessageIndex === -1) return c;

        // Keep messages up to and including the edited message, then add bot response
        const updatedMessages = c.messages.slice(0, editedMessageIndex + 1);
        return {
          ...c,
          messages: [...updatedMessages, botMessage]
        };
      })
    );

    if (botMessage.itinerary_id !== null) {
      setSelectedItineraryId(botMessage.itinerary_id);
      setItinerarySidebarVisible(true);
    }
  };

  const handleDeleteChat = (deletedChatId: number) => {
    // Remove the deleted chat from the chats list
    setChats((prevChats) => {
      if (!prevChats) return prevChats;
      return prevChats.filter((chat) => chat.id !== deletedChatId);
    });
  };

  const handleRenameChat = (chatId: number, newTitle: string) => {
    // Update the chat title in the chats list
    setChats((prevChats) => {
      if (!prevChats) return prevChats;
      return prevChats.map((chat) =>
        chat.id === chatId ? { ...chat, title: newTitle } : chat
      );
    });
  };

  const handleToggleSidebar = () => {
    setSidebarVisible((prev) => !prev);
  };

  const handleToggleItinerarySidebar = () => {
    setItinerarySidebarVisible((prev) => !prev);
  };

  const activeChat = chats?.find((c) => c.id === activeChatId) ?? null;

  return (
    <div className="home-page">
      <Navbar page="home" firstName={firstName} />
      <div
        className={`home-layout ${sidebarVisible ? "with-sidebar" : "no-sidebar"}`}
      >
        {showFinishPopup && <FinishAccountPopup />}

        <PrevChatSideBar
          chats={chats}
          activeChatId={activeChatId}
          onSelectChat={setActiveChatId}
          onNewChat={handleNewChat}
          onToggleSidebar={handleToggleSidebar}
          onDeleteChat={handleDeleteChat}
          onRenameChat={handleRenameChat}
          sidebarVisible={sidebarVisible}
        />

        <div className="chat-window-wrapper">
          <ChatWindow
            messages={activeChat?.messages ?? []}
            onSend={handleSendMessage}
            onItinerarySelect={handleItinerarySelect}
            onEditMessage={handleEditMessage}
          />
        </div>

        <ItinerarySideBar
          onToggleSidebar={handleToggleItinerarySidebar}
          sidebarVisible={itinerarySidebarVisible}
          itineraryData={itineraryData}
          selectedItineraryId={selectedItineraryId}
          itineraryTitle={itineraryTitle}
        />
      </div>
    </div>
  );
}
