import { useState, useRef, useEffect } from "react";
import "../styles/PrevChatSideBar.css";
import ContextWindow from "./ContextWindow";
import type { ChatSession } from "../models/home";
import { apiDeleteChat, apiRenameChat } from "../api/home";
import { ACTIVE_CHAT_SESSION } from "../pages/Home";

interface PrevChatSideBarProps {
  chats: ChatSession[] | null;
  activeChatId: number | null;
  onSelectChat: (id: number) => void;
  onNewChat: () => void;
  onToggleSidebar: () => void;
  onDeleteChat: (id: number) => void;
  onRenameChat: (id: number, newTitle: string) => void;
  sidebarVisible: boolean;
}

export default function PrevChatSideBar({
  chats,
  activeChatId,
  onSelectChat,
  onNewChat,
  onToggleSidebar,
  onDeleteChat,
  onRenameChat,
  sidebarVisible
}: PrevChatSideBarProps) {
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    chatId: number;
  } | null>(null);
  const [editingChatId, setEditingChatId] = useState<number | null>(null);
  const [editingTitle, setEditingTitle] = useState<string>("");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (editingChatId !== null && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [editingChatId]);

  const handleContextMenu = (e: React.MouseEvent, chatId: number) => {
    e.stopPropagation();
    const rect = (e.target as HTMLElement).getBoundingClientRect();
    setContextMenu({
      x: rect.left + 5,
      y: rect.bottom + 7,
      chatId
    });
  };

  const handleDelete = async () => {
    if (contextMenu) {
      const chatIdToDelete = contextMenu.chatId;
      const response = await apiDeleteChat(chatIdToDelete);

      if (response.status === 200) {
        // deleting an active chat causes us to just start a new chat
        if (chatIdToDelete === activeChatId) {
          onNewChat();
        }
        // need to tell Home.tsx what to delete, so the chat list can update properly
        onDeleteChat(chatIdToDelete);
      } else {
        console.error("Failed to delete chat:", response.status);
      }

      setContextMenu(null);
    }
  };

  const handleRename = () => {
    if (contextMenu) {
      const chatIdToRename = contextMenu.chatId;
      const chat = chats?.find((c) => c.id === chatIdToRename);
      
      if (!chat) {
        setContextMenu(null);
        return;
      }

      setEditingChatId(chatIdToRename);
      setEditingTitle(chat.title);
      setContextMenu(null);
    }
  };

  const handleTitleSubmit = async (chatId: number) => {
    const trimmedTitle = editingTitle.trim();
    
    if (trimmedTitle === "") {
      // Don't allow empty titles, revert to original
      setEditingChatId(null);
      setEditingTitle("");
      return;
    }

    const chat = chats?.find((c) => c.id === chatId);
    if (chat && trimmedTitle !== chat.title) {
      const response = await apiRenameChat({
        id: chatId,
        new_title: trimmedTitle
      });

      if (response.status === 200) {
        onRenameChat(chatId, trimmedTitle);
      } else {
        console.error("Failed to rename chat:", response.status);
      }
    }

    setEditingChatId(null);
    setEditingTitle("");
  };

  const handleTitleKeyDown = (e: React.KeyboardEvent, chatId: number) => {
    if (e.key === "Enter") {
      handleTitleSubmit(chatId);
    } else if (e.key === "Escape") {
      setEditingChatId(null);
      setEditingTitle("");
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
                  onClick={() => {
                    if (editingChatId !== chat.id) {
                      onSelectChat(chat.id);
                      sessionStorage.setItem(
                        ACTIVE_CHAT_SESSION,
                        chat.id.toString()
                      );
                    }
                  }}
                >
                  {editingChatId === chat.id ? (
                    <input
                      ref={inputRef}
                      type="text"
                      className="chat-title-input"
                      value={editingTitle}
                      onChange={(e) => setEditingTitle(e.target.value)}
                      onBlur={() => handleTitleSubmit(chat.id)}
                      onKeyDown={(e) => handleTitleKeyDown(e, chat.id)}
                      onClick={(e) => e.stopPropagation()}
                    />
                  ) : (
                    <span className="chat-title">{chat.title}</span>
                  )}
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
          onRename={handleRename}
        />
      )}
    </div>
  );
}