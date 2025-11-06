import React, { useState } from "react";
import "../styles/EventCard.css";

interface EventCardProps {
  event_name: string;
  event_description?: string;
  draggable?: boolean;
  time?: string;
  street_address?: string;
  postal_code?: number;
  city?: string;
  event_type?: string;
  user_created?: boolean;
  account_id?: number | null;
  hard_start?: Date | null;
  hard_end?: Date | null;

  // Added handlers for drag logic
  onDragStart?: (e: React.DragEvent, eventData: any) => void;
  onDragEnd?: (e: React.DragEvent) => void;
}

const EventCard: React.FC<EventCardProps> = ({
  event_name,
  event_description,
  time,
  street_address,
  city,
  event_type,
  hard_start,
  hard_end,
  user_created,
  //account_id,
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
      onDragStart(e, { event_name, event_description, time });
    }
  };

  const handleDragEnd = (e: React.DragEvent) => {
    setIsDragging(false);
    if (onDragEnd) {
      onDragEnd(e);
    }
  };

  const formatDateTime = (date: Date | null | undefined) => {
    if (!date) return null;
    try {
      const dateObj = typeof date === "string" ? new Date(date) : date;
      return dateObj.toLocaleString();
    } catch {
      return null;
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
        <h3 className="event-title">{event_name}</h3>
        {(street_address || city) && (
          <p className="event-location">
            {street_address && city
              ? `${street_address}, ${city}`
              : street_address || city}
          </p>
        )}
        {event_type && <p className="event-type">{event_type}</p>}
      </div>

      {isOpen && (
        <div className="event-modal-overlay" onClick={closeModal}>
          <div className="event-modal" onClick={(e) => e.stopPropagation()}>
            <button className="close-button" onClick={closeModal}>
              ✕
            </button>
            <h2>{event_name}</h2>
            {event_description && <p>{event_description}</p>}
            {time && (
              <p>
                <strong>Time:</strong> {time}
              </p>
            )}
            {street_address && (
              <p>
                <strong>Address:</strong> {street_address}
              </p>
            )}
            {city && (
              <p>
                <strong>City:</strong> {city}
              </p>
            )}
            {hard_start && (
              <p>
                <strong>Start:</strong> {formatDateTime(hard_start)}
              </p>
            )}
            {hard_end && (
              <p>
                <strong>End:</strong> {formatDateTime(hard_end)}
              </p>
            )}
            {event_type && (
              <p>
                <strong>Type:</strong> {event_type}
              </p>
            )}
            {user_created && (
              <p>
                <strong>User Created:</strong> Yes
              </p>
            )}
          </div>
        </div>
      )}
    </>
  );
};

export default EventCard;
