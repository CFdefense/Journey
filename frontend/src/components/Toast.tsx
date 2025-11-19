import { useState, useEffect, useCallback, useRef } from "react";
import "../styles/Toast.css";

export type ToastType = "success" | "error" | "info" | "warning";

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
  duration?: number;
}

interface ToastProps {
  toast: Toast;
  onClose: (id: string) => void;
  isExiting: boolean;
}

function ToastItem({ toast, onClose, isExiting }: ToastProps) {
  const duration = toast.duration ?? 2000;
  const timerRef = useRef<NodeJS.Timeout | null>(null);
  const onCloseRef = useRef(onClose);

  // Keep the onClose ref updated
  useEffect(() => {
    onCloseRef.current = onClose;
  }, [onClose]);

  useEffect(() => {
    if (duration > 0 && !isExiting) {
      // Clear any existing timer
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
      // Set new timer
      timerRef.current = setTimeout(() => {
        onCloseRef.current(toast.id);
      }, duration);
      return () => {
        if (timerRef.current) {
          clearTimeout(timerRef.current);
        }
      };
    } else {
      // Clear timer if exiting
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
            strokeLinecap="round"
            strokeLinejoin="round"
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
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>
        );
      case "warning":
        return (
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
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
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="16" x2="12" y2="12" />
            <line x1="12" y1="8" x2="12.01" y2="8" />
          </svg>
        );
    }
  };

  return (
    <div
      className={`toast toast--${toast.type} ${isExiting ? "toast-exiting" : ""}`}
    >
      <div className="toast-content">
        <div className="toast-icon">{getIcon()}</div>
        <p className="toast-message">{toast.message}</p>
        <button
          className="toast-close"
          onClick={() => onClose(toast.id)}
          aria-label="Close notification"
        >
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
    </div>
  );
}

// Toast manager state
let toastState: Toast[] = [];
let listeners: Array<(toasts: Toast[]) => void> = [];

function notifyListeners() {
  listeners.forEach((listener) => listener([...toastState]));
}

function addToast(
  message: string,
  type: ToastType = "info",
  duration?: number
) {
  const id = Math.random().toString(36).substring(2, 9) + Date.now().toString();
  const newToast: Toast = {
    id,
    message,
    type,
    duration
  };
  toastState = [...toastState, newToast];
  notifyListeners();
}

function removeToast(id: string) {
  // Remove immediately - the component will handle the exit animation
  toastState = toastState.filter((t) => t.id !== id);
  notifyListeners();
}

// Toast API
export const toast = {
  success: (message: string, duration?: number) =>
    addToast(message, "success", duration),
  error: (message: string, duration?: number) =>
    addToast(message, "error", duration),
  info: (message: string, duration?: number) =>
    addToast(message, "info", duration),
  warning: (message: string, duration?: number) =>
    addToast(message, "warning", duration),
  show: (message: string, type: ToastType = "info", duration?: number) =>
    addToast(message, type, duration)
};

// Main Toast Container Component
export default function Toast() {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [exitingIds, setExitingIds] = useState<Set<string>>(new Set());

  useEffect(() => {
    const listener = (newToasts: Toast[]) => {
      setToasts(newToasts);
      // Clean up exiting IDs for toasts that are no longer in state
      setExitingIds((prev) => {
        const currentIds = new Set(newToasts.map((t) => t.id));
        const newSet = new Set<string>();
        prev.forEach((id) => {
          if (currentIds.has(id)) {
            newSet.add(id);
          }
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
    // Mark as exiting first to trigger animation
    setExitingIds((prev) => new Set(prev).add(id));
    // Wait for animation to complete before removing
    setTimeout(() => {
      removeToast(id);
    }, 300); // Match animation duration in CSS
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
