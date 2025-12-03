import React, { useState, useEffect } from "react";
import "../styles/ViewPageSidebar.css";
import userPfp from "../../public/user-pfp-temp.png";

interface ViewPageSidebarProps {
  onCreateEvent: () => void;
  onSearchEvents: () => void;
  onAddDay: () => void;
  onEditWithAI: () => void;
}

const ViewPageSidebar: React.FC<ViewPageSidebarProps> = ({
  onCreateEvent,
  onSearchEvents,
  onAddDay,
  onEditWithAI
}) => {
  const [plusMenuOpen, setPlusMenuOpen] = useState(false);

  // Close plus menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as HTMLElement;
      if (plusMenuOpen && !target.closest(".view-sidebar-menu-container")) {
        setPlusMenuOpen(false);
      }
    };

    if (plusMenuOpen) {
      document.addEventListener("mousedown", handleClickOutside);
      return () => {
        document.removeEventListener("mousedown", handleClickOutside);
      };
    }
  }, [plusMenuOpen]);

  return (
    <div className="view-page-sidebar">
      <div className="view-sidebar-top">
        {/* Journey Logo */}
        <a href="/home" className="view-sidebar-logo">
          <img src="/logo.png" alt="Journey" className="logo-icon" />
        </a>

        {/* AI Button */}
        <button
          className="view-sidebar-button ai-button"
          onClick={onEditWithAI}
          title="Edit with AI"
        >
          <img src="/robot-svgrepo-com.svg" alt="AI" className="robot-icon" />
          <span className="view-sidebar-tooltip">Edit with AI</span>
        </button>

        {/* Plus Button with Dropdown Menu */}
        <div className="view-sidebar-menu-container">
          <button
            className="view-sidebar-button plus-button"
            onClick={() => setPlusMenuOpen(!plusMenuOpen)}
            title="Add element"
          >
            <svg
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <line x1="12" y1="5" x2="12" y2="19"></line>
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
            <span className="view-sidebar-tooltip">Add element</span>
          </button>

          {plusMenuOpen && (
            <div className="view-sidebar-dropdown">
              <button
                className="view-sidebar-menu-item"
                onClick={() => {
                  onCreateEvent();
                  setPlusMenuOpen(false);
                }}
              >
                Create Event
              </button>
              <button
                className="view-sidebar-menu-item"
                onClick={() => {
                  onSearchEvents();
                  setPlusMenuOpen(false);
                }}
              >
                Search Events
              </button>
              <button
                className="view-sidebar-menu-item"
                onClick={() => {
                  onAddDay();
                  setPlusMenuOpen(false);
                }}
              >
                Add Day
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Profile Picture at Bottom */}
      <div className="view-sidebar-bottom">
        <a href="/account" className="view-sidebar-profile" title="Account">
          <img
            src={userPfp}
            alt="Profile"
            className="view-sidebar-profile-pic"
          />
          <span className="view-sidebar-tooltip profile-tooltip">Account</span>
        </a>
      </div>
    </div>
  );
};

export default ViewPageSidebar;
