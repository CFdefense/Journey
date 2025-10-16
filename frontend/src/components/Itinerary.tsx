import React, { useState } from "react";
import EventCard from "./EventCard"; // import your new component
import "../styles/Itinerary.css";

interface Event {
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
    { time: "Evening", events: [] }
  ]);

  const [unassignedEvents, setUnassignedEvents] = useState<Event[]>([
    { id: "1", title: "Breakfast", desc: "Saxbys coffee and bagel" },
    { id: "2", title: "Capping Meeting", desc: "Make user stories" },
    { id: "3", title: "Lunch", desc: "Halal Shack" },
    { id: "4", title: "Work on itinerary", desc: "Grind time" }
  ]);

  const onDragStart = (e: React.DragEvent, event: Event) => {
    e.dataTransfer.setData("eventId", event.id); //grabs the event block (and its data based on id)
  };

  const onDrop = (e: React.DragEvent, timeIndex: number) => {
    const eventId = e.dataTransfer.getData("eventId");
    const draggedEvent =
      unassignedEvents.find((ev) => ev.id === eventId) || //checks the unassigned events or timeblocks for the event id
      timeBlocks.flatMap((tb) => tb.events).find((ev) => ev.id === eventId);

    if (!draggedEvent) return;

    // Remove from unassigned or other time blocks
    setUnassignedEvents((prev) => prev.filter((ev) => ev.id !== eventId));
    setTimeBlocks((prev) =>
      prev.map((tb, i) =>
        i === timeIndex
          ? {
              ...tb,
              events: [
                ...tb.events.filter((ev) => ev.id !== eventId),
                draggedEvent
              ]
            }
          : { ...tb, events: tb.events.filter((ev) => ev.id !== eventId) }
      )
    );
  };

  const onDragOver = (e: React.DragEvent) => {
    e.preventDefault();
  };

  return (
    <div className="itinerary-page">
      {/* 🔹 Unassigned Events Section */}
      <div className="unassigned-section">
        <h3>Unassigned Events</h3>
        <div className="unassigned-list">
          {unassignedEvents.map((event) => (
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

      {/* 🔹 Main Itinerary Section */}
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
                    onDragStart={(e) => onDragStart(e, event)}
                  />
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default Itinerary;
