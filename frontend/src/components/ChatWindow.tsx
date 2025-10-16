import MessageInput from "./MessageInput";
import "../styles/ChatWindow.css";

interface Message {
  id: number;
  text: string;
  sender: "user" | "bot";
}

// messages is the array of messages from the active chat
interface ChatWindowProps {
  messages: Message[];
  onSend: (text: string) => void;
}

export default function ChatWindow({ messages, onSend }: ChatWindowProps) {
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
              {msg.text}
            </div>
          ))
        )}
      </div>
      <MessageInput onSend={onSend} />
    </div>
  );
}
