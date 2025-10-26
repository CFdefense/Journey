import MessageInput from "./MessageInput";
import "../styles/ChatWindow.css";
import type { Message } from "../models/chat";
import ChatMessage from "./ChatMessage";

interface ChatWindowProps {
  messages: Message[];
  onSend: (text: string) => void;
  onItinerarySelect: (itineraryId: number) => void;
}

export default function ChatWindow({
  messages,
  onSend,
  onItinerarySelect
}: ChatWindowProps) {
  // returns the entire ChatWindow Component
  return (
    <div className="chat-container">
      <div className="chat-messages">
        {messages.length === 0 ? (
          <div className="chat-empty-state">
            <p>What are your travel plans?</p>
          </div>
        ) : (
          messages.map((msg, i) => {
            return (
              <ChatMessage
                key={i}
                message={msg}
                onItinerarySelect={onItinerarySelect}
              />
            );
          })
        )}
      </div>

      <MessageInput onSend={onSend} />
    </div>
  );
}