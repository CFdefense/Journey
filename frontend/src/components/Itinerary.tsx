import React, { useState } from "react";
import "../styles/Itinerary.css"
import {
  DragDropContext,
  Droppable,
  Draggable,
  type DropResult,
} from "@hello-pangea/dnd";

interface EventItem {
  id: string;
  time: string;
  title: string;
  link?: string;
}

const initialEvents: EventItem[] = [ //array of events that will be put in the itinerary
  { id: "1", time: "08:00 AM", title: "wake" },
  { id: "2", time: "10:00 AM", title: "eat" },
  { id: "3", time: "12:30 PM", title: "lunch" },
  { id: "4", time: "02:00 PM", title: "walk" },
  { id: "5", time: "06:00 PM", title: "dinner" },
];

export const Itinerary: React.FC = () => {
  const [events, setEvents] = useState<EventItem[]>(initialEvents);

  const handleDragEnd = (result: DropResult) => {
    if (!result.destination) return;

    const updated = Array.from(events);
    const [moved] = updated.splice(result.source.index, 1);
    updated.splice(result.destination.index, 0, moved);
    setEvents(updated);
  };

  return (
    <div className="itinerary-container">
      <h2 className="itinerary-title">Your Itinerary</h2>
      <DragDropContext onDragEnd={handleDragEnd}>
        <Droppable droppableId="itinerary">
          {(provided) => (
            <div
              className="itinerary-list"
              {...provided.droppableProps}
              ref={provided.innerRef}
            >
              {events.map((event, index) => (
                <Draggable key={event.id} draggableId={event.id} index={index}>
                  {(provided, snapshot) => (
                    <div
                      className={`itinerary-item ${
                        snapshot.isDragging ? "dragging" : ""
                      }`}
                      ref={provided.innerRef}
                      {...provided.draggableProps}
                      {...provided.dragHandleProps}
                    >
                      <div className="itinerary-info">
                        <p className="itinerary-time">{event.time}</p>
                        <p className="itinerary-event">{event.title}</p>
                      </div>
                      <span className="drag-handle">⋮⋮</span>
                    </div>
                  )}
                </Draggable>
              ))}
              {provided.placeholder}
            </div>
          )}
        </Droppable>
      </DragDropContext>
    </div>
  );
};
