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

    const userMessage: Message = {
		id: Date.now(),
		text,
		sender: "user",
 	};

	// here is where API will need to be called to get the official bot reply
	const botMessage: Message = {
		id: Date.now() + 1, // small offset to avoid duplicate keys
		text: "bot reply",
		sender: "bot",
	};

    setMessages((prev) => [...prev, userMessage, botMessage]);
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
