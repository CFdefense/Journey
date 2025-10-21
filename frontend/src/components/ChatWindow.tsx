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
  return (
    <div className="chat-container">
      <div className="chat-messages">
        {messages.length === 0 ? (
          <p className="chat-placeholder"> </p>
        ) : (
          messages.map((msg) => (
            <div
              key={msg.id}
              className={`chat-message ${msg.is_user ? "user" : "bot"}`}
            >
              <p>{msg.text}</p>
              
              {!msg.is_user && msg.itinerary_id && (
                <div className="itinerary-info">
                    <button className="display-itinerary-button" onClick={() => onItinerarySelect(msg.itinerary_id!)}>
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
