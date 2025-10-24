import React, { useEffect, useState } from "react";
import EventCard from "./EventCard";
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

interface DayItinerary {
  date: string;
  timeBlocks: TimeBlock[];
}

interface ItineraryProps {
  days?: DayItinerary[];
}

const Itinerary: React.FC<ItineraryProps> = ({ days }) => {
  const [selectedDayIndex, setSelectedDayIndex] = useState(0);
  const [timeBlocks, setTimeBlocks] = useState<TimeBlock[]>([]);
  const [editMode, setEditMode] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);

  useEffect(() => {
    if (days && days.length > 0) {
      setTimeBlocks(days[selectedDayIndex].timeBlocks);
    }
  }, [days, selectedDayIndex]);

  const onDrop = (e: React.DragEvent, timeIndex: number) => {
    const eventId = e.dataTransfer.getData("eventId");
    const title = e.dataTransfer.getData("eventTitle");
    const desc = e.dataTransfer.getData("eventDesc");

    if (!eventId || !title) return;

    const draggedEvent: Event = { id: eventId, title, desc };

    setTimeBlocks((prev) =>
      prev.map((tb, i) =>
        i === timeIndex
          ? { ...tb, events: [...tb.events, draggedEvent] }
          : tb
      )
    );
  };

  const onDragOver = (e: React.DragEvent) => {
    if (editMode) e.preventDefault();
  }

  const handleSave = async () => {
    setEditMode(false);
    setMenuOpen(false);
    
    console.log("Saving updated itinerary:", {
      day: days?.[selectedDayIndex].date,
      updatedTimeBlocks: timeBlocks,
    });
  };

  if (!days || days.length === 0) {
    return <div className="itinerary-section">No itinerary data available</div>;
  }

  return (
    <div className="itinerary-section">
      {/* Header Row */}
      <div className="itinerary-header">
        <h3>Itinerary</h3>

        <div className="menu-wrapper">
          <button
            className="menu-button"
            onClick={() => setMenuOpen((prev) => !prev)}
          >
            ⋯
          </button>

          {menuOpen && (
            <div className="menu-dropdown">
              {!editMode && (
                <button
                  className="menu-item"
                  onClick={() => {
                    setEditMode(true);
                    setMenuOpen(false);
                  }}
                >
                  Edit
                </button>
              )}
              {editMode && (
                <button className="menu-item" onClick={handleSave}>
                  Save
                </button>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Day Tabs */}
      <div className="day-tabs">
        {days?.map((day, index) => (
          <button
            key={day.date}
            className={`day-tab ${index === selectedDayIndex ? "active" : ""}`}
            onClick={() => setSelectedDayIndex(index)}
          >
            Day {index + 1} ({day.date})
          </button>
        ))}
      </div>

      {/* Time Blocks */}
      <div className="itinerary-list">
        {timeBlocks.map((block, index) => (
          <div
            key={block.time}
            className={`time-block ${editMode ? "editable" : ""}`}
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
                  time={block.time}
                  draggable={editMode}
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