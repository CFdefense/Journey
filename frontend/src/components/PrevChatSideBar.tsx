import { useState } from "react";
import "../styles/PrevChatSideBar.css";
import ContextWindow from "./ContextWindow";
import type { ChatSession } from "../models/home";

interface PrevChatSideBarProps {
  chats: ChatSession[] | null;
  activeChatId: number | null;
  onSelectChat: (id: number) => void;
  onNewChat: () => void;
  onToggleSidebar: () => void;
  sidebarVisible: boolean;
}

export default function PrevChatSideBar({
  chats,
  activeChatId,
  onSelectChat,
  onNewChat,
  onToggleSidebar,
  sidebarVisible
}: PrevChatSideBarProps) {
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    chatId: number;
  } | null>(null);

  const handleContextMenu = (e: React.MouseEvent, chatId: number) => {
    e.stopPropagation();
    const rect = (e.target as HTMLElement).getBoundingClientRect();
    setContextMenu({
      x: rect.right + 5,
      y: rect.top,
      chatId
    });
  };

  const handleDelete = () => {
    if (contextMenu) {
      console.log("delete chat", contextMenu.chatId);
      setContextMenu(null);
    }
  };

  return (
    <div className={`sidebar ${sidebarVisible ? "open" : "closed"}`}>
      <div className="sidebar-top">
        {sidebarVisible && <div className="sidebar-title">Chat History</div>}
        <button className="sidebar-toggle-btn" onClick={onToggleSidebar}>
          ☰
        </button>
      </div>

      {sidebarVisible && (
        <>
          <div className="new-chat-btn-wrapper">
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
                  <span className="chat-title">{chat.title}</span>
                  <button
                    className="chat-menu-btn"
                    onClick={(e) => handleContextMenu(e, chat.id)}
                  >
                    ...
                  </button>
                </li>
              ))
            )}
          </ul>
        </>
      )}

      {contextMenu && (
        <ContextWindow
          x={contextMenu.x}
          y={contextMenu.y}
          onClose={() => setContextMenu(null)}
          onDelete={handleDelete}
        />
      )}
    </div>
  );
}