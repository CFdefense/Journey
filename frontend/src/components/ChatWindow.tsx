import MessageInput from "./MessageInput";
import "../styles/ChatWindow.css";
import { Link } from "react-router-dom";

interface Message {
  id: number;
  text: string;
  sender: "user" | "bot";
  itinerary_id?: number | null; 
}

interface ChatWindowProps {
  messages: Message[];
  onSend: (text: string) => void;
  itineraryTitles: Record<number, string>; 
  onItinerarySelect: (itineraryId: number) => void;
}

export default function ChatWindow({ messages, onSend, itineraryTitles, onItinerarySelect }: ChatWindowProps) {
  return (
    <div className="chat-container">
      <div className="chat-messages">
        {messages.length === 0 ? (
          <p className="chat-placeholder"> </p>
        ) : (
          messages.map((msg) => (
            <div
              key={msg.id}
              className={`chat-message ${msg.sender === "user" ? "user" : "bot"}`}
            >
              <p>{msg.text}</p>
              
              {msg.sender === "bot" && msg.itinerary_id && (
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
          ))
        )}
      </div>

      <MessageInput onSend={onSend} />
    </div>
  );
}
