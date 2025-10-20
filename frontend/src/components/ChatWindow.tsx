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
}

export default function ChatWindow({ messages, onSend, itineraryTitles }: ChatWindowProps) {
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
                  <Link to={`/itinerary/${msg.itinerary_id}`} className="itinerary-link">
                    Itinerary: {itineraryTitles[msg.itinerary_id] || "No Itinerary (change later)"} 
                    {/* Will display this fallback text only if the database does not have an itinerary title for this message */} 
                  </Link>
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
