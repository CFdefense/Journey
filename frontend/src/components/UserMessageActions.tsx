// UserMessageActions.tsx
import "../styles/UserMessageActions.css";

export type UserMessageActionsParams = {
  messageId: number;
  onEdit: (messageId: number) => void;
  isAiResponding?: boolean;
};

export default function UserMessageActions({
  messageId,
  onEdit,
  isAiResponding = false
}: UserMessageActionsParams) {
  return (
    <button
      className="chat-edit-button"
      onClick={() => onEdit(messageId)}
      title={
        isAiResponding ? "Cannot edit while AI is responding" : "Edit message"
      }
      disabled={isAiResponding}
    >
      <svg
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
        aria-hidden="true"
      >
        {/* Simple pen icon */}
        <path d="M3 21l3.5-1 11-11a2.121 2.121 0 0 0-3-3l-11 11L3 21z" />
        <path d="M15 6l3 3" />
      </svg>
    </button>
  );
}
