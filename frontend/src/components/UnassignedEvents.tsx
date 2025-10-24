// src/components/UnassignedEvents.tsx
import React from "react";
import EventCard from "./EventCard";
import "../styles/Itinerary.css";

export interface Event {
  id: string;
  title: string;
  desc?: string;
}

interface UnassignedEventsProps {
  events: Event[];
  onDragStart: (e: React.DragEvent, event: Event) => void;
}

const UnassignedEvents: React.FC<UnassignedEventsProps> = ({
  events,
  onDragStart
}) => {
  return (
    <div className="unassigned-section">
      <h3>Unassigned Events</h3>
      <div className="unassigned-list">
        {events.map((event) => (
          <EventCard
            key={event.id}
            title={event.title}
            desc={event.desc}
            draggable
            onDragStart={(e) => onDragStart(e, event)}
          />
        ))}
      </div>
    </div>
  );
};

export default UnassignedEvents;
