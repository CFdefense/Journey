import "../styles/PrevChatSideBar.css";

interface ChatSession {
  id: number;
  title: string;
}

interface PrevChatSideBarProps {
  chats: ChatSession[];
  activeChatId: number | null;
  onSelectChat: (id: number) => void;
  onNewChat: () => void;
}

export default function PrevChatSideBar({
  chats, // array of all chat sessions (which each have messages in them)
  activeChatId,
  onSelectChat,
  onNewChat,
}: PrevChatSideBarProps) {
  return (
    <div className="sidebar">
      <button className="new-chat-btn" onClick={onNewChat}>
        + New Chat
      </button>
      <ul className="chat-list">
        {chats.length === 0 ? (
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
