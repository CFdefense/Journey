// UserMessageActions.tsx
import "../styles/UserMessageActions.css";

export type UserMessageActionsParams = {
  messageId: number;
  onEdit: (messageId: number) => void;
};

export default function UserMessageActions({
  messageId,
  onEdit
}: UserMessageActionsParams) {
  return (
    <button
      className="edit-button"
      onClick={() => onEdit(messageId)}
      title="Edit message"
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
      >
        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
      </svg>
      Edit
    </button>
  );
}
