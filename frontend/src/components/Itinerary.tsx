import React, { useEffect, useState } from "react";
import EventCard from "./EventCard";
import type { Event } from "../models/itinerary";
import "../styles/Itinerary.css";

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
  unassigned?: Event[];
  onUpdate?: (updatedDays: DayItinerary[]) => void;
  onSave?: (updatedDays: DayItinerary[]) => Promise<void>;
  editMode?: boolean;
  title?: string;
  compact?: boolean;
}

const Itinerary: React.FC<ItineraryProps> = ({
  days,
  unassigned,
  onUpdate,
  onSave,
  editMode,
  title,
  compact = false
}) => {
  const [selectedDayIndex, setSelectedDayIndex] = useState(0);
  const [localDays, setLocalDays] = useState<DayItinerary[]>(days || []);
  const [unassignedEvents, setUnassignedEvents] = useState<Event[]>([]);
  const [buttonsDisabled, setButtonsDisabled] = useState<boolean>(true);

  // Sync local state with props when days change
  useEffect(() => {
    if (days) {
      setLocalDays(days);
    }
  }, [days]);
  useEffect(() => {
    setUnassignedEvents(unassigned || []);
  }, [unassigned]);

  const onDragStart = (e: React.DragEvent, event: Event, timeIndex: number) => {
    e.dataTransfer.setData("eventId", event.id.toString());
    e.dataTransfer.setData("eventName", event.event_name);
    e.dataTransfer.setData("eventDescription", event.event_description || "");
    e.dataTransfer.setData("sourceTimeIndex", timeIndex.toString());
  };

  const onDrop = (e: React.DragEvent, targetTimeIndex: number) => {
    e.preventDefault();

    const eventIdStr = e.dataTransfer.getData("eventId");
    const eventName = e.dataTransfer.getData("eventName");
    const eventDescription = e.dataTransfer.getData("eventDescription");
    const sourceTimeIndexStr = e.dataTransfer.getData("sourceTimeIndex");

    if (!eventIdStr || !eventName) return;

    const eventId = parseInt(eventIdStr);
    const sourceTimeIndex = sourceTimeIndexStr
      ? parseInt(sourceTimeIndexStr)
      : -1;

    // Create a copy of localDays
    const updatedDays = JSON.parse(JSON.stringify(localDays)) as DayItinerary[];
    const currentDay = updatedDays[selectedDayIndex];
    let unassigned_events = JSON.parse(
      JSON.stringify(unassignedEvents)
    ) as Event[];

    // Remove event from source time block if it exists
    if (sourceTimeIndex >= 0) {
      currentDay.timeBlocks[sourceTimeIndex].events = currentDay.timeBlocks[
        sourceTimeIndex
      ].events.filter((e) => e.id !== eventId);
    } else {
      unassigned_events = unassigned_events.filter((e) => e.id !== eventId);
    }

    // Find the full event object from the source
    let draggedEvent: Event | undefined =
      sourceTimeIndex >= 0
        ? localDays[selectedDayIndex].timeBlocks[sourceTimeIndex].events.find(
            (e) => e.id === eventId
          )
        : unassignedEvents.find((e) => e.id === eventId);

    if (!draggedEvent) {
      // Fallback if we can't find the full event
      draggedEvent = {
        id: eventId,
        event_name: eventName,
        event_description: eventDescription,
        street_address: "",
        postal_code: 0,
        city: "",
        event_type: "",
        user_created: false,
        account_id: null,
        hard_start: null,
        hard_end: null
      };
    }

    // Add event to target time block if not already there
    if (targetTimeIndex >= 0) {
      const targetBlock = currentDay.timeBlocks[targetTimeIndex];
      if (!targetBlock.events.some((e) => e.id === eventId)) {
        targetBlock.events.push(draggedEvent);
      }
    } else if (!unassigned_events.some((e) => e.id === eventId)) {
      unassigned_events.push(draggedEvent);
    }

    // Update local state immediately for UI responsiveness
    setLocalDays(updatedDays);
    setUnassignedEvents(unassigned_events);
    setButtonsDisabled(false);
  };

  const onDragOver = (e: React.DragEvent) => {
    if (editMode) e.preventDefault();
  };

  const handleSave = async () => {
    console.log("Saving updated itinerary:", {
      day: localDays[selectedDayIndex].date,
      updatedTimeBlocks: localDays[selectedDayIndex].timeBlocks
    });

    // Update parent state
    if (onUpdate) {
      onUpdate(localDays);
    }

    // Call parent's save function (which handles API call)
    if (onSave) {
      try {
        await onSave(localDays);
      } catch (error) {
        console.error("Save failed:", error);
        // show error
      }
    }

    setButtonsDisabled(true);
  };

  const handleCancel = () => {
    // Revert to original days
    if (days) {
      setLocalDays(days);
    }
    setUnassignedEvents(unassigned || []);
    setButtonsDisabled(true);
  };

  if (!localDays || localDays.length === 0) {
    return <div className="itinerary-section">No itinerary data available</div>;
  }

  const currentDay = localDays[selectedDayIndex];

  const getTimeRange = (timeLabel: string): string => {
    switch (timeLabel) {
      case "Morning":
        return "6:00 AM - 12:00 PM";
      case "Afternoon":
        return "12:00 PM - 6:00 PM";
      case "Evening":
        return "6:00 PM - 12:00 AM";
      default:
        return "";
    }
  };

  return (
    <div className={`itinerary-section ${compact ? "compact" : ""}`}>
      {/* Header Row */}
      <div className="itinerary-header">
        <h3>{title || "Itinerary"}</h3>

        {editMode && (
          <div className="edit-buttons-container">
            <button
              className="edit-button"
              id="itinerary-save"
              onClick={handleSave}
              disabled={buttonsDisabled}
            >
              Save
            </button>
            <button
              className="edit-button"
              id="itinerary-cancel"
              onClick={handleCancel}
              disabled={buttonsDisabled}
            >
              Cancel
            </button>
          </div>
        )}
      </div>

      {/* Unassigned Events */}
      {editMode && (
        <div className="unassigned-events">
          <div
            className={"time-block editable"}
            onDrop={(e) => onDrop(e, -1)}
            onDragOver={onDragOver}
          >
            <div className="events-area">
              {unassignedEvents.map((event) => (
                <EventCard
                  key={event.id}
                  event_name={event.event_name}
                  event_description={event.event_description}
                  street_address={event.street_address}
                  city={event.city}
                  event_type={event.event_type}
                  user_created={event.user_created}
                  account_id={event.account_id}
                  hard_start={event.hard_start}
                  hard_end={event.hard_end}
                  draggable={editMode}
                  onDragStart={(e) => onDragStart(e, event, -1)}
                />
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Day Tabs */}
      <div className="day-tabs">
        {localDays.map((day, index) => (
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
        {currentDay.timeBlocks.map((block, timeIndex) => (
          <div
            key={block.time}
            className={`time-block ${editMode ? "editable" : ""}`}
            onDrop={(e) => onDrop(e, timeIndex)}
            onDragOver={onDragOver}
          >
            <div className="time-label">
              <span>{block.time}</span>
              <span className="time-range">{getTimeRange(block.time)}</span>
            </div>
            <div className="events-area">
              {block.events.map((event) => (
                <EventCard
                  key={event.id}
                  event_name={event.event_name}
                  event_description={event.event_description}
                  time={block.time}
                  street_address={event.street_address}
                  city={event.city}
                  event_type={event.event_type}
                  user_created={event.user_created}
                  account_id={event.account_id}
                  hard_start={event.hard_start}
                  hard_end={event.hard_end}
                  draggable={editMode}
                  onDragStart={(e) => onDragStart(e, event, timeIndex)}
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
