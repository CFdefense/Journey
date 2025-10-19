import "../styles/PrevChatSideBar.css";

interface ChatSession {
  id: number;
  title: string;
  messages?: { id: number; text: string; sender: string }[];
}

interface PrevChatSideBarProps {
  chats: ChatSession[];
  activeChatId: number | null;
  onSelectChat: (id: number) => void;
  onNewChat: () => void;
}

export default function PrevChatSideBar({
  chats,
  activeChatId,
  onSelectChat,
  onNewChat,
}: PrevChatSideBarProps) {
  // Filter chats that have at least one message (only show these in sidebar)
  const visibleChats = chats.filter((chat) => chat.messages && chat.messages.length > 0);

  return (
    <div className="sidebar">
      <button className="new-chat-btn" onClick={onNewChat}>
        + New Chat
      </button>

      <ul className="chat-list">
        {visibleChats.length === 0 ? (
          <p className="empty">No previous chats yet</p>
        ) : (
          visibleChats.map((chat) => (
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
