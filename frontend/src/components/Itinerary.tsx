// src/components/Itinerary.tsx
import React, { useState } from "react";
import EventCard from "./EventCard";
import "../styles/Itinerary.css";

export interface Event {
  id: string;
  title: string;
  desc?: string;
}

interface TimeBlock {
  time: string;
  events: Event[];
}

const Itinerary: React.FC = () => {
  const [timeBlocks, setTimeBlocks] = useState<TimeBlock[]>([
    { time: "Morning", events: [] },
    { time: "Afternoon", events: [] },
    { time: "Evening", events: [] },
  ]);

  const onDragOver = (e: React.DragEvent) => e.preventDefault();

  const onDrop = (e: React.DragEvent, timeIndex: number) => {
    const eventId = e.dataTransfer.getData("eventId");
    const eventTitle = e.dataTransfer.getData("eventTitle");
    const eventDesc = e.dataTransfer.getData("eventDesc");

    if (!eventId) return;

    const draggedEvent: Event = { id: eventId, title: eventTitle, desc: eventDesc };

    setTimeBlocks((prev) =>
      prev.map((tb, i) =>
        i === timeIndex
          ? { ...tb, events: [...tb.events.filter((ev) => ev.id !== eventId), draggedEvent] }
          : { ...tb, events: tb.events.filter((ev) => ev.id !== eventId) }
      )
    );
  };

  return (
    <div className="itinerary-section">
      <h3>Itinerary</h3>
      <div className="itinerary-list">
        {timeBlocks.map((block, index) => (
          <div
            key={block.time}
            className="time-block"
            onDrop={(e) => onDrop(e, index)}
            onDragOver={onDragOver}
          >
            <div className="time-label">{block.time}</div>
            <div className="events-area">
              {block.events.map((event) => (
                <EventCard
                  key={event.id}
                  title={event.title}
                  desc={event.desc}
                  draggable
                  onDragStart={(e) => {
                    e.dataTransfer.setData("eventId", event.id);
                    e.dataTransfer.setData("eventTitle", event.title);
                    e.dataTransfer.setData("eventDesc", event.desc || "");
                  }}
                />
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default Itinerary;
