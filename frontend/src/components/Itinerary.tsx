import React, { useState } from "react";
import "../styles/Itinerary.css";

interface Event {
  id: string;
  title: string;
}

interface TimeBlock {
  time: string;
  events: Event[];
}

const Itinerary: React.FC = () => {
  const [timeBlocks, setTimeBlocks] = useState<TimeBlock[]>([
    { time: "08:00 AM", events: [] },
    { time: "09:00 AM", events: [] },
    { time: "10:00 AM", events: [] },
    { time: "11:00 AM", events: [] },
    { time: "12:00 PM", events: [] },
    { time: "01:00 PM", events: [] },
    { time: "02:00 PM", events: [] },
  ]);

  const [unassignedEvents, setUnassignedEvents] = useState<Event[]>([
    { id: "1", title: "Breakfast" },
    { id: "2", title: "Team Meeting" },
    { id: "3", title: "Lunch with Client" },
    { id: "4", title: "Gym" },
  ]);

  const onDragStart = (e: React.DragEvent, event: Event) => {
    e.dataTransfer.setData("eventId", event.id);
  };

  const onDrop = (e: React.DragEvent, timeIndex: number) => {
    const eventId = e.dataTransfer.getData("eventId");
    const draggedEvent =
      unassignedEvents.find((ev) => ev.id === eventId) ||
      timeBlocks.flatMap((tb) => tb.events).find((ev) => ev.id === eventId);


    if (!draggedEvent) return;

    // Remove from unassigned or old time block
    setUnassignedEvents((prev) => prev.filter((ev) => ev.id !== eventId));
    setTimeBlocks((prev) =>
      prev.map((tb, i) =>
        i === timeIndex
          ? { ...tb, events: [...tb.events, draggedEvent] }
          : { ...tb, events: tb.events.filter((ev) => ev.id !== eventId) }
      )
    );
  };

  const onDragOver = (e: React.DragEvent) => {
    e.preventDefault();
  };

  return (
    <div className="itinerary-container">
      <div className="unassigned-section">
        <h3>Unassigned Events</h3>
        <div className="unassigned-list">
          {unassignedEvents.map((event) => (
            <div
              key={event.id}
              className="event-card"
              draggable
              onDragStart={(e) => onDragStart(e, event)}
            >
              {event.title}
            </div>
          ))}
        </div>
      </div>

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
                  <div
                    key={event.id}
                    className="event-card"
                    draggable
                    onDragStart={(e) => onDragStart(e, event)}
                  >
                    {event.title}
                  </div>
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
