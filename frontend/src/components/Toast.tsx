import { useState, useEffect, useCallback, useRef } from "react";
import "../styles/Toast.css";

export type ToastType =
  | "success"
  | "error"
  | "info"
  | "warning"
  | "account-warning";

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
  duration?: number;
  actionUrl?: string;
}

interface ToastProps {
  toast: Toast;
  onClose: (id: string) => void;
  isExiting: boolean;
}

function ToastItem({ toast, onClose, isExiting }: ToastProps) {
  // REMOVED: const navigate = useNavigate();
  const duration = toast.duration ?? 2000;
  const timerRef = useRef<NodeJS.Timeout | null>(null);
  const onCloseRef = useRef(onClose);

  useEffect(() => {
    onCloseRef.current = onClose;
  }, [onClose]);

  useEffect(() => {
    if (duration > 0 && !isExiting) {
      if (timerRef.current) clearTimeout(timerRef.current);

      timerRef.current = setTimeout(() => {
        onCloseRef.current(toast.id);
      }, duration);

      return () => {
        if (timerRef.current) clearTimeout(timerRef.current);
      };
    } else {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
        timerRef.current = null;
      }
    }
  }, [toast.id, duration, isExiting]);

  const getIcon = () => {
    switch (toast.type) {
      case "success":
        return (
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
            <polyline points="22 4 12 14.01 9 11.01" />
          </svg>
        );

      case "error":
        return (
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>
        );

      case "warning":
      case "account-warning":
        return (
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" />
            <line x1="12" y1="9" x2="12" y2="13" />
            <line x1="12" y1="17" x2="12.01" y2="17" />
          </svg>
        );

      default:
        return (
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="16" x2="12" y2="12" />
            <line x1="12" y1="8" x2="12.01" y2="8" />
          </svg>
        );
    }
  };

  const handleToastClick = () => {
    // Use window.location for navigation instead of useNavigate
    if (toast.actionUrl) {
      window.location.href = toast.actionUrl;
    }
  };

  return (
    <div
      className={`toast toast--${toast.type} ${isExiting ? "toast-exiting" : ""}`}
      onClick={handleToastClick}
      style={{ cursor: toast.actionUrl ? "pointer" : "default" }}
    >
      <div className="toast-content">
        <div className="toast-icon">{getIcon()}</div>
        <p className="toast-message">{toast.message}</p>
        <button
          className="toast-close"
          onClick={(e) => {
            e.stopPropagation();
            onClose(toast.id);
          }}
          aria-label="Close notification"
        >
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
    </div>
  );
}

// Toast Manager State
let toastState: Toast[] = [];
let listeners: Array<(toasts: Toast[]) => void> = [];

function notifyListeners() {
  listeners.forEach((listener) => listener([...toastState]));
}

function addToast(
  message: string,
  type: ToastType = "info",
  duration?: number,
  actionUrl?: string
) {
  const id = Math.random().toString(36).substring(2, 9) + Date.now();

  const newToast: Toast = {
    id,
    message,
    type,
    duration,
    actionUrl
  };

  toastState = [...toastState, newToast];
  notifyListeners();
}

function removeToast(id: string) {
  toastState = toastState.filter((t) => t.id !== id);
  notifyListeners();
}

// Public Toast API
export const toast = {
  success: (message: string, duration?: number) =>
    addToast(message, "success", duration),
  error: (message: string, duration?: number) =>
    addToast(message, "error", duration),
  info: (message: string, duration?: number) =>
    addToast(message, "info", duration),
  warning: (message: string, duration?: number) =>
    addToast(message, "warning", duration),

  /** Persistent account warning toast */
  accountWarning: (message: string, actionUrl: string) =>
    addToast(message, "account-warning", 0, actionUrl),

  show: (message: string, type: ToastType = "info", duration?: number) =>
    addToast(message, type, duration)
};

export default function Toast() {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [exitingIds, setExitingIds] = useState<Set<string>>(new Set());

  useEffect(() => {
    const listener = (newToasts: Toast[]) => {
      setToasts(newToasts);

      setExitingIds((prev) => {
        const currentIds = new Set(newToasts.map((t) => t.id));
        const newSet = new Set<string>();

        prev.forEach((id) => {
          if (currentIds.has(id)) newSet.add(id);
        });

        return newSet;
      });
    };

    listeners.push(listener);
    setToasts([...toastState]);

    return () => {
      listeners = listeners.filter((l) => l !== listener);
    };
  }, []);

  const handleClose = useCallback((id: string) => {
    setExitingIds((prev) => new Set(prev).add(id));

    setTimeout(() => {
      removeToast(id);
    }, 300);
  }, []);

  if (toasts.length === 0) return null;

  return (
    <div className="toast-container">
      {toasts.map((toast) => (
        <ToastItem
          key={toast.id}
          toast={toast}
          onClose={handleClose}
          isExiting={exitingIds.has(toast.id)}
        />
      ))}
    </div>
  );
}
