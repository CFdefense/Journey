// src/components/UnassignedEvents.tsx
import React from "react";
import EventCard from "./EventCard";
import type { Event } from "../models/itinerary";
import "../styles/Itinerary.css";

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
            event={event}
            draggable
            onDragStart={(e) => onDragStart(e, event)}
            localDays={[]}
            setLocalDays={function (): void {
              throw new Error("Function not implemented.");
            }}
            unassignedEvents={[]}
            setUnassignedEvents={function (): void {
              throw new Error("Function not implemented.");
            }}
          />
        ))}
      </div>
    </div>
  );
};

export default UnassignedEvents;
