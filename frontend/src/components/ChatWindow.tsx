// ChatWindow.tsx
import { useState, useEffect, useRef } from "react";
import MessageInput from "./MessageInput";
import "../styles/ChatWindow.css";
import type { Message } from "../models/chat";
import ChatMessage from "./ChatMessage";
import aiPic from "../assets/ai-pic.png";

interface ChatWindowProps {
  messages: Message[];
  onSend: (text: string) => void;
  onItinerarySelect: (itineraryId: number) => void;
  onEditMessage: (messageId: number, newText: string) => void;
  hasActiveChat?: boolean;
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
  hasActiveChat = false
}: ChatWindowProps) {
  const [emptyStateInput, setEmptyStateInput] = useState("");
  const [displayedText, setDisplayedText] = useState("");
  const [isExpanding, setIsExpanding] = useState(false);
  const [isInitialLoad, setIsInitialLoad] = useState(messages.length > 0);
  const [isSendingEmpty, setIsSendingEmpty] = useState(false);
  const animationTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const typeIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const deleteIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const prevMessagesLengthRef = useRef<number>(0);
  const hasMountedRef = useRef<boolean>(false);
  const mountTimeRef = useRef<number>(Date.now());
  const initialLoadTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const expandingTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    // Track initial mount
    if (!hasMountedRef.current) {
      hasMountedRef.current = true;
      mountTimeRef.current = Date.now();
      // If we have messages on initial mount, this is a reloaded chat session
      if (messages.length > 0) {
        setIsInitialLoad(true);
        // Remove initial load state after animation completes
        if (initialLoadTimeoutRef.current) {
          clearTimeout(initialLoadTimeoutRef.current);
        }
        initialLoadTimeoutRef.current = setTimeout(() => setIsInitialLoad(false), 900);
      }
    }

    const wasEmpty = prevMessagesLengthRef.current === 0;
    const nowHasMessages = messages.length > 0;
    const timeSinceMount = Date.now() - mountTimeRef.current;
    
    // If messages appear within 500ms of mount, treat it as initial load (reloaded chat)
    // Otherwise, if transitioning from empty to having messages, it's an expansion
    if (nowHasMessages && wasEmpty && hasMountedRef.current) {
      if (timeSinceMount < 500) {
        // Messages appeared very quickly after mount - this is a reloaded chat
        setIsInitialLoad(true);
        setIsExpanding(false);
        if (initialLoadTimeoutRef.current) {
          clearTimeout(initialLoadTimeoutRef.current);
        }
        initialLoadTimeoutRef.current = setTimeout(() => setIsInitialLoad(false), 900);
      } else {
        // Messages appeared after some time - this is a new message in an empty chat
        setIsInitialLoad(false);
        setIsExpanding(true);
        if (expandingTimeoutRef.current) {
          clearTimeout(expandingTimeoutRef.current);
        }
        expandingTimeoutRef.current = setTimeout(() => setIsExpanding(false), 800);
      }
    } else if (!wasEmpty) {
      // If we already had messages, we're switching chats, not expanding
      setIsExpanding(false);
      setIsInitialLoad(false);
    }

    if (messages.length > 0) {
      // Reset when messages appear
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
      prevMessagesLengthRef.current = messages.length;
      return;
    } else {
      prevMessagesLengthRef.current = 0;
    }

    // Wait for fade-in animation to complete (600ms) before starting typing
    // On first load, type from the beginning
    const fadeInDelay = setTimeout(() => {
      typeText(0, true);
    }, 600);

    return () => {
      clearTimeout(fadeInDelay);
      if (animationTimeoutRef.current) {
        clearTimeout(animationTimeoutRef.current);
      }
      if (typeIntervalRef.current) {
        clearInterval(typeIntervalRef.current);
      }
      if (deleteIntervalRef.current) {
        clearInterval(deleteIntervalRef.current);
      }
      if (initialLoadTimeoutRef.current) {
        clearTimeout(initialLoadTimeoutRef.current);
      }
      if (expandingTimeoutRef.current) {
        clearTimeout(expandingTimeoutRef.current);
      }
    };
  }, [messages.length]);

  const typeText = (endingIndex: number, startFromBeginning: boolean = false) => {
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
    if (emptyStateInput.trim()) {
      setIsSendingEmpty(true);
      onSend(emptyStateInput.trim());
      setEmptyStateInput("");
      // Reset animation state after animation completes
      setTimeout(() => setIsSendingEmpty(false), 600);
    }
  };

  const showEmptyState = !hasActiveChat && messages.length === 0;
  const titleText = displayedText;

  return (
    <div className={`chat-container ${showEmptyState ? "chat-container-empty" : ""} ${isExpanding ? "expanding" : ""} ${isInitialLoad ? "initial-load" : ""}`}>
      {showEmptyState ? (
        <div className="chat-empty-state">
          <h1 className="chat-empty-title">
            {titleText}
            <span className="typing-cursor">|</span>
          </h1>
          <form className={`chat-empty-search ${isSendingEmpty ? "sending" : ""}`} onSubmit={handleEmptyStateSubmit}>
            <input
              type="text"
              value={emptyStateInput}
              onChange={(e) => setEmptyStateInput(e.target.value)}
              placeholder="Ask anything"
              className="chat-empty-input"
              autoFocus
            />
            <button type="submit" className="chat-empty-submit">
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
            <img src={aiPic} alt="AI Assistant" className="chat-header-image" />
            <div className="chat-header-text">
              <div className="chat-header-title">Travel Assistant</div>
              <div className="chat-header-subtitle">
                Ready to help with your next adventure
              </div>
            </div>
          </div>

          <div className="chat-messages">
            {messages.map((msg) => {
              return (
                <ChatMessage
                  key={msg.id}
                  message={msg}
                  onItinerarySelect={onItinerarySelect}
                  onEditMessage={onEditMessage}
                />
              );
            })}
          </div>

          <MessageInput onSend={onSend} />
        </>
      )}
    </div>
  );
}
