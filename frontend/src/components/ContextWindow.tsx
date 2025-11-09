import { useEffect, useRef } from "react";
import "../styles/ContextWindow.css";

interface ContextWindowProps {
  x: number;
  y: number;
  onClose: () => void;
  onDelete: () => void;
  onRename: () => void;
}

export default function ContextWindow({
  x,
  y,
  onClose,
  onDelete,
  onRename
}: ContextWindowProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        onClose();
      }
    };

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleEscape);

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleEscape);
    };
  }, [onClose]);

  return (
    <div
      ref={menuRef}
      className="context-window"
      style={{ top: `${y}px`, left: `${x}px` }}
    >
      <button className="context-menu-item rename" onClick={onRename}>
        Rename
      </button>
      <button className="context-menu-item delete" onClick={onDelete}>
        Delete
      </button>
    </div>
  );
}