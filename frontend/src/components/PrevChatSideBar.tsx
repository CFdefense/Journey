import "../styles/PrevChatSideBar.css";
import type { ChatSession } from "../models/home";

interface PrevChatSideBarProps {
  chats: ChatSession[] | null;
  activeChatId: number | null;
  onSelectChat: (id: number) => void;
  onNewChat: () => void;
  onToggleSidebar: () => void;
}

export default function PrevChatSideBar({
  chats,
  activeChatId,
  onSelectChat,
  onNewChat,
  onToggleSidebar
}: PrevChatSideBarProps) {
   // Filter chats that have at least one message (only show these in sidebar)
  //const visibleChats = chats.filter((chat) => chat.messages && chat.messages.length > 0);
  return (
    <div className="sidebar">
      <div className="sidebar-header">
        <button className="toggle-btn" onClick={onToggleSidebar}>
          ✕
        </button>
        <button className="new-chat-btn" onClick={onNewChat}>
          + New Chat
        </button>
      </div>

      <ul className="chat-list">
        {chats === null || chats.length === 0 ? (
          <p className="empty">No previous chats yet</p>
        ) : (
          chats.map((chat) => (
            <li
              key={chat.id}
              className={chat.id === activeChatId ? "active" : ""}
              onClick={() => onSelectChat(chat.id)}
            >
              {chat.title}
            </li>
          ))
        )}
      </ul>
    </div>
  );
}
