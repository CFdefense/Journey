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
            event_name={event.event_name}
            event_description={event.event_description}
            street_address={event.street_address}
            postal_code={event.postal_code}
            city={event.city}
            event_type={event.event_type}
            user_created={event.user_created}
            account_id={event.account_id}
            hard_start={event.hard_start}
            hard_end={event.hard_end}
            draggable
            onDragStart={(e) => onDragStart(e, event)}
          />
        ))}
      </div>
    </div>
  );
};

export default UnassignedEvents;
