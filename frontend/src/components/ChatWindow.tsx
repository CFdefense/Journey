// ChatWindow.tsx
import { useState, useEffect, useRef } from "react";
import MessageInput from "./MessageInput";
import "../styles/ChatWindow.css";
import type { Message } from "../models/chat";
import ChatMessage from "./ChatMessage";
import { apiMessages } from "../api/home";
import { useNavigate } from "react-router-dom";

interface ChatWindowProps {
  messages: Message[];
  onSend: (text: string) => void;
  onItinerarySelect: (itineraryId: number) => void;
  onEditMessage: (messageId: number, newText: string) => void;
  hasActiveChat?: boolean;
  chat_session_id: number;
  set_messages: (msgs: Message[], chat_id: number) => void;
  prevMsgId: number | null | undefined;
  setPrevMsgId: (id: number | null | undefined) => void;
  isAiResponding?: boolean;
}

const BASE_TEXT = "What are your ";
const ENDINGS = [
  "travel plans?",
  "adventure goals?",
  "journey ideas?",
  "vacation dreams?"
];

export default function ChatWindow({
  messages,
  onSend,
  onItinerarySelect,
  onEditMessage,
  hasActiveChat = false,
  chat_session_id,
  set_messages,
  prevMsgId,
  setPrevMsgId,
  isAiResponding = false
}: ChatWindowProps) {
  const [emptyStateInput, setEmptyStateInput] = useState("");
  const [displayedText, setDisplayedText] = useState("");
  const [isExpanding, setIsExpanding] = useState(false);
  const [isSwitching, setIsSwitching] = useState(false);
  const [isSendingEmpty, setIsSendingEmpty] = useState(false);
  // const [allMsgLoaded, setAllMsgLoaded] = useState(false);

  const animationTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const typeIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const deleteIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const prevMessagesRef = useRef<Message[]>([]);
  const prevMessagesLengthRef = useRef<number>(0);
  const hasMountedRef = useRef<boolean>(false);
  const mountTimeRef = useRef<number>(Date.now());
  const switchingTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const expandingTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const animationStartedRef = useRef<boolean>(false);
  const shouldPreserveAnimationRef = useRef<boolean>(false);
  const initialFadeInDelayRef = useRef<NodeJS.Timeout | null>(null);

  const navigate = useNavigate();

  useEffect(() => {
    // Check if messages have actually changed
    const messagesChanged =
      prevMessagesLengthRef.current !== messages.length ||
      prevMessagesRef.current.length !== messages.length ||
      (messages.length > 0 &&
        prevMessagesRef.current.length > 0 &&
        prevMessagesRef.current[0]?.id !== messages[0]?.id);

    // Helper function to create cleanup that preserves animation if needed
    const createCleanup = (
      fadeInDelay?: NodeJS.Timeout,
      isInitialMount: boolean = false
    ) => {
      return () => {
        // Don't clear initial fadeInDelay if we're preserving animation
        // (it will be cleared when typeText is called or when messages change)
        if (
          fadeInDelay &&
          isInitialMount &&
          shouldPreserveAnimationRef.current
        ) {
          // Preserve the initial timeout - don't clear it
          return;
        }
        if (fadeInDelay) {
          clearTimeout(fadeInDelay);
        }
        if (isInitialMount && fadeInDelay) {
          initialFadeInDelayRef.current = null;
        }
        // Only clear intervals if we shouldn't preserve the animation
        if (!shouldPreserveAnimationRef.current) {
          if (animationTimeoutRef.current) {
            clearTimeout(animationTimeoutRef.current);
          }
          if (typeIntervalRef.current) {
            clearInterval(typeIntervalRef.current);
          }
          if (deleteIntervalRef.current) {
            clearInterval(deleteIntervalRef.current);
          }
        }
        if (switchingTimeoutRef.current) {
          clearTimeout(switchingTimeoutRef.current);
        }
        if (expandingTimeoutRef.current) {
          clearTimeout(expandingTimeoutRef.current);
        }
      };
    };

    // Determine if we should preserve the animation (messages haven't changed and we're in empty state)
    const shouldPreserve =
      !messagesChanged && messages.length === 0 && animationStartedRef.current;

    // Track initial mount
    if (!hasMountedRef.current) {
      hasMountedRef.current = true;
      mountTimeRef.current = Date.now();
      prevMessagesRef.current = messages;
      prevMessagesLengthRef.current = messages.length;

      // If we have messages on initial mount, this is a reloaded chat session
      if (messages.length > 0) {
        setIsSwitching(true);
        shouldPreserveAnimationRef.current = false;
        if (switchingTimeoutRef.current) {
          clearTimeout(switchingTimeoutRef.current);
        }
        switchingTimeoutRef.current = setTimeout(
          () => setIsSwitching(false),
          500
        );
        return createCleanup();
      }
      // If empty on initial mount, start animation
      else {
        animationStartedRef.current = true;
        // Set preserve flag early so cleanup doesn't clear the timeout
        shouldPreserveAnimationRef.current = true;
        // Clear any existing timeout first
        if (initialFadeInDelayRef.current) {
          clearTimeout(initialFadeInDelayRef.current);
        }
        initialFadeInDelayRef.current = setTimeout(() => {
          typeText(0, true);
          initialFadeInDelayRef.current = null;
        }, 600);

        return createCleanup(initialFadeInDelayRef.current, true);
      }
    }

    // If messages haven't changed and we're in empty state with animation running, don't restart
    if (shouldPreserve) {
      // Animation is already running, don't restart it
      // Set ref so next cleanup knows to preserve
      shouldPreserveAnimationRef.current = true;
      return createCleanup();
    }

    // If we're in empty state but animation hasn't started yet (e.g., initial mount timeout was cleared),
    // start it now
    if (
      messages.length === 0 &&
      !animationStartedRef.current &&
      hasMountedRef.current &&
      prevMessagesLengthRef.current === 0
    ) {
      animationStartedRef.current = true;
      shouldPreserveAnimationRef.current = true; // Preserve so cleanup doesn't clear it
      if (initialFadeInDelayRef.current) {
        clearTimeout(initialFadeInDelayRef.current);
      }
      initialFadeInDelayRef.current = setTimeout(() => {
        typeText(0, true);
        initialFadeInDelayRef.current = null;
      }, 600);
      return createCleanup(initialFadeInDelayRef.current, true);
    }

    // Reset preserve flag since we're not preserving
    shouldPreserveAnimationRef.current = false;

    const wasEmpty = prevMessagesLengthRef.current === 0;
    const nowHasMessages = messages.length > 0;
    const timeSinceMount = Date.now() - mountTimeRef.current;

    // Detect if we're switching chats (messages changed but not just appended)
    // Compare first message ID to detect chat switches
    const prevFirstId = prevMessagesRef.current[0]?.id;
    const currentFirstId = messages[0]?.id;
    const isChatSwitch =
      messages.length > 0 &&
      prevMessagesRef.current.length > 0 &&
      (prevFirstId !== currentFirstId ||
        messages.length < prevMessagesLengthRef.current);

    // If switching between chats with messages
    if (isChatSwitch) {
      setIsSwitching(true);
      setIsExpanding(false);
      animationStartedRef.current = false;
      if (switchingTimeoutRef.current) {
        clearTimeout(switchingTimeoutRef.current);
      }
      switchingTimeoutRef.current = setTimeout(
        () => setIsSwitching(false),
        500
      );
    }
    // If messages appear within 500ms of mount, treat it as initial load (reloaded chat)
    // Otherwise, if transitioning from empty to having messages, it's an expansion
    else if (nowHasMessages && wasEmpty && hasMountedRef.current) {
      animationStartedRef.current = false;
      if (timeSinceMount < 500) {
        // Messages appeared very quickly after mount - this is a reloaded chat
        setIsSwitching(true);
        setIsExpanding(false);
        if (switchingTimeoutRef.current) {
          clearTimeout(switchingTimeoutRef.current);
        }
        switchingTimeoutRef.current = setTimeout(
          () => setIsSwitching(false),
          500
        );
      } else {
        // Messages appeared after some time - this is a new message in an empty chat
        setIsSwitching(false);
        setIsExpanding(true);
        if (expandingTimeoutRef.current) {
          clearTimeout(expandingTimeoutRef.current);
        }
        expandingTimeoutRef.current = setTimeout(
          () => setIsExpanding(false),
          800
        );
      }
    } else if (!wasEmpty && !isChatSwitch) {
      // If we already had messages and it's not a switch, just reset states
      setIsExpanding(false);
      setIsSwitching(false);
    }

    // Update refs
    prevMessagesRef.current = messages;
    prevMessagesLengthRef.current = messages.length;

    if (messages.length > 0) {
      // Reset when messages appear
      animationStartedRef.current = false;
      shouldPreserveAnimationRef.current = false;
      setDisplayedText("");
      if (animationTimeoutRef.current) {
        clearTimeout(animationTimeoutRef.current);
      }
      if (typeIntervalRef.current) {
        clearInterval(typeIntervalRef.current);
      }
      if (deleteIntervalRef.current) {
        clearInterval(deleteIntervalRef.current);
      }
      return createCleanup();
    }

    // Only start animation if messages actually changed to empty state
    // (transitioned from non-empty to empty, or initial mount with empty)
    if (messages.length === 0) {
      // If transitioning from non-empty to empty, start animation
      if (messagesChanged && !wasEmpty) {
        animationStartedRef.current = true;
        shouldPreserveAnimationRef.current = false; // Starting new animation, don't preserve yet
        const fadeInDelay = setTimeout(() => {
          typeText(0, true);
        }, 600);

        return createCleanup(fadeInDelay);
      }
      // If already empty and animation not started yet (initial mount case handled separately)
      else if (!animationStartedRef.current && hasMountedRef.current) {
        animationStartedRef.current = true;
        shouldPreserveAnimationRef.current = false; // Starting new animation, don't preserve yet
        const fadeInDelay = setTimeout(() => {
          typeText(0, true);
        }, 600);

        return createCleanup(fadeInDelay);
      }
      // If animation is already running and messages haven't changed, preserve it
      else if (animationStartedRef.current && !messagesChanged) {
        shouldPreserveAnimationRef.current = true;
        return createCleanup();
      }
    }

    // Default cleanup if we reach here
    shouldPreserveAnimationRef.current = false;
    return createCleanup();
  }, [messages]);

  const typeText = (
    endingIndex: number,
    startFromBeginning: boolean = false
  ) => {
    const currentEnding = ENDINGS[endingIndex];
    const fullText = BASE_TEXT + currentEnding;
    // On first load, type from the beginning. On subsequent cycles, start from BASE_TEXT
    let currentIndex = startFromBeginning ? 0 : BASE_TEXT.length;

    if (typeIntervalRef.current) {
      clearInterval(typeIntervalRef.current);
    }

    typeIntervalRef.current = setInterval(() => {
      if (currentIndex <= fullText.length) {
        setDisplayedText(fullText.slice(0, currentIndex));
        currentIndex++;
      } else {
        if (typeIntervalRef.current) {
          clearInterval(typeIntervalRef.current);
        }
        // Wait 3 seconds before deleting
        animationTimeoutRef.current = setTimeout(() => {
          deleteText(endingIndex);
        }, 3000);
      }
    }, 50); // Typing speed
  };

  const deleteText = (endingIndex: number) => {
    const currentEnding = ENDINGS[endingIndex];
    const fullText = BASE_TEXT + currentEnding;
    let currentIndex = fullText.length;

    if (deleteIntervalRef.current) {
      clearInterval(deleteIntervalRef.current);
    }

    deleteIntervalRef.current = setInterval(() => {
      if (currentIndex > BASE_TEXT.length) {
        setDisplayedText(fullText.slice(0, currentIndex - 1));
        currentIndex--;
      } else {
        if (deleteIntervalRef.current) {
          clearInterval(deleteIntervalRef.current);
        }
        // Move to next ending
        const nextIndex = (endingIndex + 1) % ENDINGS.length;
        // Start typing the new ending after a brief pause
        animationTimeoutRef.current = setTimeout(() => {
          typeText(nextIndex);
        }, 300);
      }
    }, 30); // Deleting speed (faster than typing)
  };

  const handleEmptyStateSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (emptyStateInput.trim() && !isAiResponding) {
      setIsSendingEmpty(true);
      onSend(emptyStateInput.trim());
      setEmptyStateInput("");
      // Reset animation state after animation completes
      setTimeout(() => setIsSendingEmpty(false), 600);
    }
  };

  const onChatMsgsWheel = async (e: React.WheelEvent) => {
    if (prevMsgId === undefined) {
      console.error("Unreachable statement");
      return;
    }
    if (e.deltaY >= 0 || prevMsgId === null) {
      return;
    }
    const chatMsgWindow = document.getElementById("chat-messages")!;
    if (chatMsgWindow.scrollTop !== 0) {
      return;
    }
    const oldScrollHeight = chatMsgWindow.scrollHeight;
    const page_result = await apiMessages({
      chat_session_id,
      message_id: prevMsgId
    });
    if (page_result.status === 401) {
      navigate("/login");
      return;
    }
    if (page_result.result === null || page_result.status !== 200) {
      alert("TODO: handle error - failed to load messages");
      return;
    }
    const res = page_result.result;
    setPrevMsgId(res.prev_message_id);
    if (res.message_page.length === 0) {
      return;
    }
    set_messages([...res.message_page, ...messages], chat_session_id);
    // use previous scroll position to preserve scroll state
    requestAnimationFrame(() => {
      const newScrollHeight = chatMsgWindow.scrollHeight;
      chatMsgWindow.scrollTop = newScrollHeight - oldScrollHeight;
    });
  };

  const showEmptyState = !hasActiveChat && messages.length === 0;
  const titleText = displayedText;

  return (
    <div
      className={`chat-container ${showEmptyState ? "chat-container-empty" : ""} ${isExpanding ? "expanding" : ""} ${isSwitching ? "switching" : ""}`}
    >
      {showEmptyState ? (
        <div className="chat-empty-state">
          <h1 className="chat-empty-title">
            {titleText}
            <span className="typing-cursor">|</span>
          </h1>
          <form
            className={`chat-empty-search ${isSendingEmpty ? "sending" : ""}`}
            onSubmit={handleEmptyStateSubmit}
          >
            <input
              type="text"
              value={emptyStateInput}
              onChange={(e) => setEmptyStateInput(e.target.value)}
              placeholder="Ask anything"
              className="chat-empty-input"
              autoFocus
            />
            <button
              type="submit"
              className="chat-empty-submit"
              disabled={isAiResponding}
            >
              <svg
                width="24"
                height="24"
                viewBox="0 0 20 20"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M10 4V14M10 4L6 8M10 4L14 8"
                  stroke="white"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
            </button>
          </form>
        </div>
      ) : (
        <>
          {/* Header Section */}
          <div className="chat-header">
            <img
              src="/ai-pic.png"
              alt="AI Assistant"
              className="chat-header-image"
            />
            <div className="chat-header-text">
              <div className="chat-header-title">Travel Assistant</div>
              <div className="chat-header-subtitle">
                Ready to help with your next adventure
              </div>
            </div>
          </div>

          <div
            id="chat-messages"
            className="chat-messages"
            onWheel={onChatMsgsWheel}
          >
            {messages.map((msg) => {
              return (
                <ChatMessage
                  key={msg.id}
                  message={msg}
                  onItinerarySelect={onItinerarySelect}
                  onEditMessage={onEditMessage}
                  isAiResponding={isAiResponding}
                />
              );
            })}
          </div>

          <MessageInput onSend={onSend} isAiResponding={isAiResponding} />
        </>
      )}
    </div>
  );
}
