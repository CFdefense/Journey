import MessageInput from "./MessageInput";
import "../styles/ChatWindow.css";
import type { Message } from "../models/chat";
import ChatMessage from "./ChatMessage";
import aiPic from "../assets/ai-pic.png";

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
  return (
    <div className="chat-container">
      {/* Header Section */}
      <div className="chat-header">
        <img src={aiPic} alt="AI Assistant" className="chat-header-image" />
        <div className="chat-header-text">
          <div className="chat-header-title">Travel Assistant</div>
          <div className="chat-header-subtitle">Ready to help with your next adventure</div>
        </div>
      </div>

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