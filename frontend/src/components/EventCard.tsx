import React, { useState, useEffect } from "react";
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

  onDragStart?: (e: React.DragEvent) => void;
}

const EventCard: React.FC<EventCardProps> = ({title, desc, time, address, draggable = false, onDragStart}) => {
  const [isOpen, setIsOpen] = useState(false);

  const openModal = () => setIsOpen(true);
  const closeModal = () => setIsOpen(false);

   return (
    <>
      {/* Normal card view */}
      <div className="event-card" onClick={openModal}>
        <h3 className="event-title">{title}</h3>
      </div>

      {/* Pop-out modal */}
      {isOpen && (
        <div className="event-modal-overlay" onClick={closeModal}>
          <div className="event-modal" onClick={(e) => e.stopPropagation()}>
            <button className="close-button" onClick={closeModal}>
              ✕
            </button>
            <h2>{title}</h2>
            <p>{desc}</p>
            {time && <p><strong>Date:</strong> {time}</p>}
            {address && <p><strong>Location:</strong> {address}</p>}
          </div>
        </div>
      )}
    </>
  );
};

export default EventCard;
