import React from "react";
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

//
const EventCard: React.FC<EventCardProps> = ({
  title,
  desc,
  draggable = false,
  onDragStart
}) => {
  return (
    <div className="event-card" draggable={draggable} onDragStart={onDragStart}>
      <div className="event-title">{title}</div>
      {desc && <div className="event-desc">{desc}</div>}
    </div>
  );
};

export default EventCard;
