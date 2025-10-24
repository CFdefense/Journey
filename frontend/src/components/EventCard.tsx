import React, { useState } from "react";
import "../styles/EventCard.css";

interface EventCardProps {
  title: string;
  desc?: string;
  draggable?: boolean;
  time?: string;
  address?: string;
  postal_code?: number;
  city?: string;
  type?: string;

  // Added handlers for drag logic
  onDragStart?: (e: React.DragEvent, eventData: any) => void;
  onDragEnd?: (e: React.DragEvent) => void;
}

const EventCard: React.FC<EventCardProps> = ({
  title,
  desc,
  time,
  address,
  draggable = false,
  onDragStart,
  onDragEnd,
}) => {
  const [isOpen, setIsOpen] = useState(false);

  const openModal = () => {
    if (!draggable) setIsOpen(true); // Prevent click from opening modal while dragging
  };
  const closeModal = () => setIsOpen(false);

  return (
    <>
      {/* Card container */}
      <div
        className={`event-card ${draggable ? "draggable" : ""}`}
        draggable={draggable}
        onDragStart={(e) => onDragStart && onDragStart(e, { title, desc, time })}
        onDragEnd={onDragEnd}
        onClick={openModal}
      >
        <h3 className="event-title">{title}</h3>
      </div>

      {/* Modal display */}
      {isOpen && (
        <div className="event-modal-overlay" onClick={closeModal}>
          <div className="event-modal" onClick={(e) => e.stopPropagation()}>
            <button className="close-button" onClick={closeModal}>
              ✕
            </button>
            <h2>{title}</h2>
            {desc && <p>{desc}</p>}
            {time && (
              <p>
                <strong>Time:</strong> {time}
              </p>
            )}
            {address && (
              <p>
                <strong>Location:</strong> {address}
              </p>
            )}
          </div>
        </div>
      )}
    </>
  );
};

export default EventCard;
