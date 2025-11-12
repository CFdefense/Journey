// Home.tsx
import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import ChatWindow from "../components/ChatWindow";
import PrevChatSideBar from "../components/PrevChatSideBar";
import ItinerarySideBar from "../components/ItinerarySideBar";
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
  UpdateMessageRequest,
  Message
} from "../models/chat";
import type { ChatSession } from "../models/home";
import { apiCurrent } from "../api/account";
import { fetchItinerary, convertToApiFormat } from "../helpers/itinerary";
import type { DayItinerary } from "../models/itinerary";
import { apiItineraryDetails, apiSaveItineraryChanges } from "../api/itinerary";

export const ACTIVE_CHAT_SESSION: string = "activeChatSession";

export default function Home() {
  const navigate = useNavigate();
  const [chats, setChats] = useState<ChatSession[] | null>(null);
  const [activeChatId, setActiveChatId] = useState<number | null>(null);
  const [showFinishPopup, setShowFinishPopup] = useState(false);
  const [selectedItineraryId, setSelectedItineraryId] = useState<number | null>(
    null
  );
  const [sidebarVisible, setSidebarVisible] = useState(false);
  const [itinerarySidebarVisible, setItinerarySidebarVisible] = useState(false);
  const [itineraryData, setItineraryData] = useState<DayItinerary[] | null>(
    null
  );
  const [itineraryTitle, setItineraryTitle] = useState<string>("");
  const [itineraryStartDate, setItineraryStartDate] = useState<string>("");
  const [itineraryEndDate, setItineraryEndDate] = useState<string>("");
  const [initialStateProcessed, setInitialStateProcessed] = useState(false);

  // Flag to track if we came from ViewItinerary - needs to be state to trigger useEffect
  const [cameFromViewItinerary, setCameFromViewItinerary] = useState(false);
  // Track the initial chat ID to know when it actually changes
  const initialChatIdRef = useRef<number | null>(null);

  // Handle navigation state from ViewItinerary
  useEffect(() => {
    if (location.state && !initialStateProcessed) {
      const { selectedItineraryId, chatSessionId, openItinerarySidebar } =
        location.state;
      console.log("Navigation state - itinerary ID:", selectedItineraryId);

      // Set the flag if we have an itinerary ID from navigation
      if (selectedItineraryId !== undefined && selectedItineraryId !== null) {
        setCameFromViewItinerary(true);
      }

      if (chatSessionId !== undefined && chatSessionId !== null) {
        initialChatIdRef.current = chatSessionId;
        setActiveChatId(chatSessionId);
        sessionStorage.setItem(ACTIVE_CHAT_SESSION, chatSessionId.toString());
      }

      if (openItinerarySidebar !== undefined) {
        setItinerarySidebarVisible(openItinerarySidebar);
      }

      // Set itinerary ID last and trigger a load
      if (selectedItineraryId !== undefined && selectedItineraryId !== null) {
        setSelectedItineraryId(selectedItineraryId);
        // Manually load itinerary data since we're setting the ID in initial state
        loadItineraryData(selectedItineraryId);
      }

      setInitialStateProcessed(true);

      // Clear navigation state after processing
      navigate(location.pathname, { replace: true, state: {} });
    }
  }, [location.state, initialStateProcessed, navigate, location.pathname]);

  useEffect(() => {
    async function fetchAccount() {
      if (showFinishPopup) {
        return;
      }
      apiCurrent()
        .then((currentResult) => {
          const account = currentResult.result;
          if (account === null || currentResult.status !== 200) {
            console.error(
              "API call to /api/account/current failed with status: ",
              currentResult.status
            );
            navigate("/login");
            return;
          }
          setShowFinishPopup(
            account.budget_preference === null ||
              account.disabilities === null ||
              account.food_allergies === null ||
              account.risk_preference === null
          );
        })
        .catch((err) => {
          console.error("Failed to fetch account:", err);
          navigate("/login");
        });
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

      // get MRU chat from session storage only if not already set by navigation state
      if (chatSessionId === null) {
        const prevActiveChat: string | null =
          sessionStorage.getItem(ACTIVE_CHAT_SESSION);
        if (prevActiveChat !== null) {
          const id = +prevActiveChat;
          if (tempChats.find((chat) => chat.id === id) !== undefined) {
            chatSessionId = id;
            setActiveChatId(chatSessionId);
          }
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
    if (selectedItineraryId !== null) {
      loadItineraryData(selectedItineraryId);
    }
  }, [selectedItineraryId]);

  // Helper function to load itinerary data
  const loadItineraryData = async (itineraryId: number) => {
    try {
      const data = await fetchItinerary(itineraryId);
      setItineraryData(data);

      const apiResponse = await apiItineraryDetails(itineraryId);

      if (apiResponse.result) {
        setItineraryTitle(apiResponse.result.title);
        setItineraryStartDate(apiResponse.result.start_date);
        setItineraryEndDate(apiResponse.result.end_date);
      }

      setItinerarySidebarVisible(true); // whenever itinerary data is loaded successfully, make sure the side bar opens
    } catch (error) {
      console.error("Error loading itinerary:", error);
      setItineraryData(null);
      setItineraryTitle("");
      setItineraryStartDate("");
      setItineraryEndDate("");
    }
  };

  // Clear itinerary data when active chat changes (but not when coming from ViewItinerary)
  useEffect(() => {
    // Don't clear on initial mount when activeChatId is null
    if (activeChatId === null && !initialStateProcessed) {
      return;
    }

    // If we came from ViewItinerary and this is the initial chat ID, don't clear
    if (cameFromViewItinerary && activeChatId === initialChatIdRef.current) {
      // Reset the flag so subsequent chat changes will clear itinerary data
      setCameFromViewItinerary(false);
      return;
    }

    // For normal chat switches, clear itinerary data
    setSelectedItineraryId(null);
    setItineraryData(null);
    setItineraryTitle("");
    setItineraryStartDate("");
    setItineraryEndDate("");
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
    let isNewChat = false;

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
      isNewChat = true;
    }

    // If we came from ViewItinerary and created a new chat with an itinerary in view,
    // associate the itinerary with this new chat session
    if (
      cameFromViewItinerary &&
      isNewChat &&
      selectedItineraryId !== null &&
      itineraryData !== null
    ) {
      try {
        const apiPayload = convertToApiFormat(
          itineraryData,
          selectedItineraryId,
          itineraryStartDate,
          itineraryEndDate,
          itineraryTitle,
          currChatId
        );

        await apiSaveItineraryChanges(apiPayload);
      } catch (error) {
        console.error("Failed to associate itinerary with new chat:", error);
        // Don't block the message send if this fails
      }
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
            key={activeChatId ?? 'empty'}
            messages={activeChat?.messages ?? []}
            onSend={handleSendMessage}
            onItinerarySelect={handleItinerarySelect}
            onEditMessage={handleEditMessage}
            hasActiveChat={activeChatId !== null}
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
