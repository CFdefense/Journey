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
  user_created?: boolean;
  account_id?: number;
  hard_start?: Date;
  hard_end?: Date;

  // Added handlers for drag logic
  onDragStart?: (e: React.DragEvent, eventData: any) => void;
  onDragEnd?: (e: React.DragEvent) => void;
}

const EventCard: React.FC<EventCardProps> = ({
  title,
  desc,
  time,
  address,
  city,
  type,
  user_created = false,
  account_id,
  hard_start,
  hard_end,
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
        {(address || city) && (
          <p className="event-location">
            {address && city ? `${address}, ${city}` : address || city}
          </p>
        )}
        {type && <p className="event-type">{type}</p>}
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
                <strong>Address:</strong> {address}
              </p>
            )}
            {city && (
              <p>
                <strong>City:</strong> {city}
              </p>
            )}
            {hard_start && (
              <p>
                <strong>Start:</strong> {hard_start.toISOString()}
              </p>
            )}
            {type && (
              <p>
                <strong>Type:</strong> {type}
              </p>
            )}
          </div>
        </div>
      )}
    </>
  );
};

export default EventCard;