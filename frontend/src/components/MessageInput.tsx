import { useState } from "react";

interface MessageInputProps {
  onSend: (text: string) => void;
}

export default function MessageInput({ onSend }: MessageInputProps) {
  const [input, setInput] = useState("");
  const [isSending, setIsSending] = useState(false);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (input.trim()) {
      setIsSending(true);
      onSend(input.trim());
      setInput("");
      // Reset animation state after animation completes
      setTimeout(() => setIsSending(false), 300);
    }
  };

  return (
    <form className={`chat-empty-search ${isSending ? "sending" : ""}`} onSubmit={handleSubmit}>
      <input
        type="text"
        value={input}
        onChange={(e) => setInput(e.target.value)}
        placeholder="Ask anything"
        className="chat-empty-input"
        autoFocus
      />
      <button type="submit" className="chat-empty-submit">
        <svg
          width="24"
          height="24"
          viewBox="0 0 20 20"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M10 4V14M10 4L6 8M10 4L14 8"
            stroke="white"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      </button>
    </form>
  );
}
