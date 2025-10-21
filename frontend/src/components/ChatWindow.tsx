import MessageInput from "./MessageInput";
import "../styles/ChatWindow.css";
import type { Message } from "../models/chat";

interface ChatWindowProps {
  messages: Message[];
  onSend: (text: string) => void;
  itineraryTitles: Record<number, string>;
  onItinerarySelect: (itineraryId: number) => void;
}

export default function ChatWindow({ messages, onSend, itineraryTitles, onItinerarySelect }: ChatWindowProps) {
  // returns the entire ChatWindow Component
  return (
    <div className="chat-container">
      <div className="chat-messages">
        {messages.length === 0 ? (
          <p className="chat-placeholder"> </p>
        ) : (
          messages.map((msg) => {
            // Format Timestamp nicely (MM/DD/YYYY, h:mm AM/PM)
            const formattedTimestamp = new Date(msg.timestamp.replace(" ", "T")).toLocaleString("en-US", {
              month: "2-digit",
              day: "2-digit",
              year: "numeric",
              hour: "2-digit",
              minute: "2-digit",
              hour12: true,
            });

            // returns each individual message within map
            return (
              <div
                key={msg.id}
                className={`chat-message ${msg.is_user ? "user" : "bot"}`}
              >
                <div className="message-text">
                  <p>{msg.text}</p>
                  <span className="timestamp">{formattedTimestamp}</span>
                </div>

                {!msg.is_user && msg.itinerary_id && (
                  <div className="itinerary-info">
                    <button
                      className="display-itinerary-button"
                      onClick={() => {
                        console.log("Selected itinerary ID:", msg.itinerary_id);
                        onItinerarySelect(msg.itinerary_id!);
                      }}
                    >
                      Itinerary: {itineraryTitles[msg.itinerary_id] || "No Itinerary (change later)"}
                      {/* Will display this fallback text only if the database does not have an itinerary title for this message */}
                    </button>
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>

      <MessageInput onSend={onSend} />
    </div>
  );
}
