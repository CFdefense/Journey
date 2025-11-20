import { useState, useRef, useEffect, useContext, type Context } from "react";
import { Link, useNavigate } from "react-router-dom";
import "../styles/PrevChatSideBar.css";
import ContextWindow from "./ContextWindow";
import type { ChatSession } from "../models/home";
import { apiDeleteChat, apiRenameChat } from "../api/home";
import { ACTIVE_CHAT_SESSION } from "../pages/Home";
import userPfp from "../assets/user-pfp-temp.png";
import { GlobalContext } from "../helpers/global";
import type { GlobalState } from "./GlobalProvider";
import { apiLogout } from "../api/account";

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
  const navigate = useNavigate();
  const { setAuthorized } = useContext<GlobalState>(
    GlobalContext as Context<GlobalState>
  );

  useEffect(() => {
    if (editingChatId !== null && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [editingChatId]);

  const handleContextMenu = (e: React.MouseEvent, chatId: number) => {
    e.stopPropagation();
    // Get the button element, not the SVG child
    const buttonElement = e.currentTarget as HTMLElement;
    const rect = buttonElement.getBoundingClientRect();
    const contextWindowWidth = 140; // Match the width in ContextWindow.css
    // Center the context window relative to the button
    const centeredX = rect.left + rect.width / 2 - contextWindowWidth / 2;
    setContextMenu({
      x: centeredX,
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

  const handleLogout = async () => {
    const { status } = await apiLogout();
    if (status !== 200) {
      console.error("Logout failed with status", status);
    }
    setAuthorized(false);
    sessionStorage.removeItem(ACTIVE_CHAT_SESSION);
    navigate("/login");
  };

  return (
    <div className={`sidebar ${sidebarVisible ? "open" : "closed"}`}>
      <div className="sidebar-actions">
        <div className="sidebar-header-top">
          <Link
            to="/"
            className={`action-btn menu-toggle-btn logo-link ${sidebarVisible ? "visible" : "hidden"}`}
            aria-label="Go to home"
            title="Home"
          >
            <img
              src="/placeholder-logo.png"
              alt="Journey Logo"
            <img
              src="/logo.png"
              alt="Journey Logo"
              className="sidebar-logo"
              onError={(e) => {
                const target = e.target as HTMLImageElement;
                const fallback = target.nextElementSibling as HTMLElement;
                if (
                  fallback &&
                  fallback.classList.contains("sidebar-logo-fallback")
                ) {
                  target.style.display = "none";
                  fallback.style.display = "flex";
                if (
                  fallback &&
                  fallback.classList.contains("sidebar-logo-fallback")
                ) {
                  target.style.display = "none";
                  fallback.style.display = "flex";
                }
              }}
            />
            <div className="sidebar-logo-fallback">J</div>
            <div className="sidebar-logo-fallback">
              <img src="/logo.png" alt="Journey Logo" />
            </div>
          </Link>
          <button
            className={`action-btn menu-toggle-btn hamburger-btn ${sidebarVisible ? "hidden" : "visible"}`}
            onClick={onToggleSidebar}
            aria-label="Toggle menu"
            title="Menu"
          >
            <span className="action-icon" aria-hidden="true">
              <svg
                viewBox="0 0 24 24"
                width="18"
                height="18"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M3 6H21"
                  stroke="#0b1220"
                  strokeWidth="2"
                  strokeLinecap="round"
                />
                <path
                  d="M3 12H21"
                  stroke="#0b1220"
                  strokeWidth="2"
                  strokeLinecap="round"
                />
                <path
                  d="M3 18H21"
                  stroke="#0b1220"
                  strokeWidth="2"
                  strokeLinecap="round"
                />
              </svg>
            </span>
          </button>
          <button
            className="action-btn menu-close-btn"
            onClick={onToggleSidebar}
            aria-label="Close menu"
            title="Close menu"
          >
            <svg
              viewBox="0 0 24 24"
              width="18"
              height="18"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M3 6H21"
                stroke="#0b1220"
                strokeWidth="2"
                strokeLinecap="round"
              />
              <path
                d="M3 12H21"
                stroke="#0b1220"
                strokeWidth="2"
                strokeLinecap="round"
              />
              <path
                d="M3 18H21"
                stroke="#0b1220"
                strokeWidth="2"
                strokeLinecap="round"
              />
            </svg>
          </button>
        </div>

        <button
          className={`action-btn primary ${sidebarVisible ? "expanded" : "icon-only"}`}
          onClick={onNewChat}
          aria-label="New chat"
          title="New chat"
        >
          <span className="action-icon" aria-hidden="true">
            <svg
              viewBox="0 0 24 24"
              width="18"
              height="18"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M12 5V19"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
              />
              <path
                d="M5 12H19"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
              />
            </svg>
          </span>
          <span className="action-label">New chat</span>
        </button>

        <button
          className={`action-btn ${sidebarVisible ? "expanded" : "icon-only"}`}
          aria-label="Search chats"
          title="Search chats"
        >
          <span className="action-icon" aria-hidden="true">
            <svg
              viewBox="0 0 24 24"
              width="18"
              height="18"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <circle
                cx="11"
                cy="11"
                r="6"
                stroke="currentColor"
                strokeWidth="2"
              />
              <path
                d="M16.5 16.5L20 20"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
              />
            </svg>
          </span>
          <span className="action-label">Search chats</span>
        </button>
      </div>

      <ul className="chat-list">
        {chats === null || chats.length === 0 ? (
          <p className="empty">No previous chats yet</p>
        ) : (
          chats.map((chat, index) => (
            <li
              key={chat.id}
              className={chat.id === activeChatId ? "active" : ""}
              style={{
                transitionDelay:
                  chat.id === activeChatId ? "0ms" : `${450 + index * 150}ms`
              }}
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
                aria-label="More options"
                title="More options"
                onClick={(e) => handleContextMenu(e, chat.id)}
              >
                <svg
                  viewBox="0 0 24 24"
                  width="18"
                  height="18"
                  xmlns="http://www.w3.org/2000/svg"
                  fill="currentColor"
                  aria-hidden="true"
                >
                  <circle cx="12" cy="5" r="2" />
                  <circle cx="12" cy="12" r="2" />
                  <circle cx="12" cy="19" r="2" />
                </svg>
              </button>
            </li>
          ))
        )}
      </ul>

      <div className="sidebar-bottom">
        <Link to="/account" className="sidebar-profile-link">
          <img
            src={userPfp}
            alt="User profile"
            className="sidebar-profile-pic"
          />
        </Link>
        <button
          className="sidebar-logout-btn"
          onClick={handleLogout}
          title="Logout"
        >
          <svg
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              d="M9 21H5C4.46957 21 3.96086 20.7893 3.58579 20.4142C3.21071 20.0391 3 19.5304 3 19V5C3 4.46957 3.21071 3.96086 3.58579 3.58579C3.96086 3.21071 4.46957 3 5 3H9"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
            <path
              d="M16 17L21 12L16 7"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
            <path
              d="M21 12H9"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </button>
      </div>

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
