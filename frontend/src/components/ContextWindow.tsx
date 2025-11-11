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
      <div className="context-window-header">
        <button className="context-window-menu-btn" onClick={onClose} aria-label="Close menu">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M3 6H21" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
            <path d="M3 12H21" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
            <path d="M3 18H21" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
        </button>
      </div>
      <button className="context-menu-item rename" onClick={onRename}>
        Rename
      </button>
      <button className="context-menu-item delete" onClick={onDelete}>
        Delete
      </button>
    </div>
  );
}
