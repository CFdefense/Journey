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
  onDragEnd
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [isDragging, setIsDragging] = useState(false);

  const openModal = () => {
    if (!isDragging) setIsOpen(true);
  };

  const closeModal = () => setIsOpen(false);

  const handleDragStart = (e: React.DragEvent) => {
    setIsDragging(true);
    if (onDragStart) {
      onDragStart(e, { title, desc, time });
    }
  };

  const handleDragEnd = (e: React.DragEvent) => {
    setIsDragging(false);
    if (onDragEnd) {
      onDragEnd(e);
    }
  };

  return (
    <>
      <div
        className={`event-card ${draggable ? "draggable" : ""}`}
        draggable={draggable}
        onDragStart={handleDragStart}
        onDragEnd={handleDragEnd}
        onClick={openModal}
      >
        <h3 className="event-title">{title}</h3>
      </div>

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
