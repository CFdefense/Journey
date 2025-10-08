import { useState } from "react";
import MessageInput from "../components/MessageInput";
import "../styles/ChatWindow.css";

interface Message {
  id: number;
  text: string;
  sender: "user" | "bot";
}

export default function ChatWindow() {
  const [messages, setMessages] = useState<Message[]>([]);

  const handleSend = (text: string) => {
    if (!text.trim()) return;

    const newMessage: Message = {
      id: Date.now(),
      text,
      sender: "user",
    };

    setMessages((prev) => [...prev, newMessage]);
  };

  return (
    <div className="chat-container">
      <div className="chat-messages">
        {messages.map((msg) => (
          <div
            key={msg.id}
            className={`chat-message ${msg.sender === "user" ? "user" : "bot"}`}
          >
            {msg.text}
          </div>
        ))}
      </div>
      <MessageInput onSend={handleSend} />
    </div>
  );
}
