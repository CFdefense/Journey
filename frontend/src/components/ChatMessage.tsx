// ChatMessage.tsx
import { useEffect, useState, useRef } from "react";
import type { Message } from "../models/chat";
import type { Itinerary } from "../models/itinerary";
import { apiItineraryDetails } from "../api/itinerary";
import aiPic from "../assets/ai-pic.png";
import UserMessageActions from "./UserMessageActions";
import "../styles/ChatMessage.css";

export type ChatMessageParams = {
  message: Message;
  onItinerarySelect: (itineraryId: number) => void;
  onEditMessage: (messageId: number, newText: string) => void;
};

export default function ChatMessage({
  message,
  onItinerarySelect,
  onEditMessage
}: ChatMessageParams) {
  const [itinerary, setItinerary] = useState<Itinerary | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [editText, setEditText] = useState(message.text);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const editContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    async function getItinerary() {
      if (message.itinerary_id === null) {
        return;
      }
      const itineraryResult = await apiItineraryDetails(message.itinerary_id);
      // TODO: 401 -> navigate to /login

      if (itineraryResult.result === null || itineraryResult.status !== 200) {
        return; // TODO: handle and display error
      }

      setItinerary(itineraryResult.result);
    }

    getItinerary();
  }, [message.itinerary_id]);

  // Constrain edit container to not overlap input bar
  useEffect(() => {
    if (isEditing && editContainerRef.current) {
      const updateMaxHeight = () => {
        const container = editContainerRef.current;
        if (!container) return;

        const rect = container.getBoundingClientRect();
        const inputBar = document.querySelector(
          ".message-input"
        ) as HTMLElement;
        if (!inputBar) return;

        const inputRect = inputBar.getBoundingClientRect();
        const availableHeight = inputRect.top - rect.top - 20; // 20px margin
        const maxHeight = Math.min(400, Math.max(150, availableHeight));
        container.style.maxHeight = `${maxHeight}px`;
      };

      updateMaxHeight();
      window.addEventListener("resize", updateMaxHeight);
      window.addEventListener("scroll", updateMaxHeight, true);

      return () => {
        window.removeEventListener("resize", updateMaxHeight);
        window.removeEventListener("scroll", updateMaxHeight, true);
      };
    }
  }, [isEditing]);

  const handleEdit = () => {
    setIsEditing(true);
    setEditText(message.text);
  };

  const handleCancelEdit = () => {
    setIsEditing(false);
    setEditText(message.text);
  };

  const handleSaveEdit = () => {
    const trimmedText = editText.trim();
    if (trimmedText === "") return;

    onEditMessage(message.id, trimmedText);
    setIsEditing(false);
  };

  const handleTextareaChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setEditText(e.target.value);
  };

  const formattedTimestamp = new Date(message.timestamp).toLocaleString(
    "en-US",
    {
      month: "2-digit",
      day: "2-digit",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
      hour12: true
    }
  );

  return (
    <div className={`chat-message-wrapper ${message.is_user ? "user" : "bot"}`}>
      {!message.is_user && (
        <img src={aiPic} alt="AI Assistant" className="message-avatar" />
      )}

      <div className={`chat-message-content ${isEditing ? "editing" : ""}`}>
        <div className={`chat-message ${message.is_user ? "user" : "bot"}`}>
          <div className="message-text">
            {isEditing ? (
              <div className="edit-message-container" ref={editContainerRef}>
                <textarea
                  ref={textareaRef}
                  className="edit-message-input"
                  value={editText}
                  onChange={handleTextareaChange}
                  autoFocus
                />
                <div className="edit-message-actions">
                  <button
                    className="edit-cancel-btn"
                    onClick={handleCancelEdit}
                  >
                    Cancel
                  </button>
                  <button className="edit-save-btn" onClick={handleSaveEdit}>
                    Save
                  </button>
                </div>
              </div>
            ) : (
              <p>{message.text}</p>
            )}
          </div>

          {!message.is_user && message.itinerary_id && (
            <div className="itinerary-info">
              <button
                className="display-itinerary-button"
                onClick={() => {
                  console.log("Selected itinerary ID:", message.itinerary_id);
                  onItinerarySelect(message.itinerary_id!);
                }}
              >
                Itinerary: {itinerary?.title ?? "No title"}
              </button>
            </div>
          )}
        </div>

        <div className="timestamp-and-actions">
          <span className="timestamp">{formattedTimestamp}</span>
          {message.is_user && !isEditing && (
            <UserMessageActions messageId={message.id} onEdit={handleEdit} />
          )}
        </div>
      </div>
    </div>
  );
}
