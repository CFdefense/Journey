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
  const [createModalOpen, setCreateModalOpen] = useState(false);
  const [searchModalOpen, setSearchModalOpen] = useState(false);

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
        country: "",
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

  const onCreateEvent = () => setCreateModalOpen(true);
  const closeCreateModal = () => setCreateModalOpen(false);
  const onSaveUserEvent = () => { alert("TODO"); };

  const onSearchEvents = () => setSearchModalOpen(true);
  const closeSearchModal = () => setSearchModalOpen(false);
  const onSearchSend = () => { alert("TODO"); };

  return (
    <div className={`itinerary-section ${compact ? "compact" : ""}`}>
      {/* Header Row */}
      <div className="itinerary-header">
        <h3>{title || "Itinerary"}</h3>

        {editMode && (
          <div className="itinerary-edit-buttons-container">
            <button
              className="itinerary-edit-button"
              id="itinerary-create"
              onClick={onCreateEvent}
            >Create Event</button>
            <button
              className="itinerary-edit-button"
              id="itinerary-search"
              onClick={onSearchEvents}
            >Search Events</button>
            <button
              className="itinerary-edit-button"
              id="itinerary-save"
              onClick={handleSave}
              disabled={buttonsDisabled}
            >Save</button>
            <button
              className="itinerary-edit-button"
              id="itinerary-cancel"
              onClick={handleCancel}
              disabled={buttonsDisabled}
            >Cancel</button>
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
                  country={event.country}
                  postal_code={event.postal_code}
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

      {createModalOpen && (
        <div className="user-event-modal-overlay" onClick={closeCreateModal}>
          <div className="user-event-modal" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Create a custom event</h2>
              <div className="modal-actions">
                <button className="card-save-button" onClick={onSaveUserEvent} title="Save">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="18"
                    height="18"
                    fill="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path d="M17 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V7l-4-4zM5 19V5h11v4h4v10H5z"/>
                    <path d="M12 12a2 2 0 1 0 0-4 2 2 0 0 0 0 4zM6 18h12v-2H6v2z"/>
                  </svg>
                </button>
                <button className="close-button" onClick={closeCreateModal} title="Close">✕</button>
              </div>
            </div>

            <form className="user-event-form">
              <label>
                Name
                <input required />
              </label>

              <label>
                Description
                <textarea rows={4} />
              </label>

              <label>
                Type of Event
                <input />
              </label>

              <div className="location-grid">
                <label>
                  Address
                  <input />
                </label>

                <label>
                  City
                  <input />
                </label>

                <label>
                  Country
                  <input />
                </label>

                <label>
                  Postal Code
                  <input type="number" maxLength={5} />
                </label>

                <label>
                  Start Time
                  <input type="datetime-local" />
                </label>

                <label>
                  End Time
                  <input type="datetime-local" />
                </label>
              </div>
            </form>
          </div>
        </div>
      )}

      {searchModalOpen && (
        <div className="user-event-modal-overlay" onClick={closeSearchModal}>
          <div className="user-event-modal" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Search for an event</h2>
              <div className="modal-actions">
                <button className="card-save-button" onClick={onSearchSend} title="Search">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="18"
                    height="18"
                    fill="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path d="M10 2a8 8 0 1 0 5.293 14.293l4.707 4.707 1.414-1.414-4.707-4.707A8 8 0 0 0 10 2zm0 2a6 6 0 1 1 0 12A6 6 0 0 1 10 4z"/>
                  </svg>
                </button>
                <button className="close-button" onClick={closeSearchModal} title="Close">✕</button>
              </div>
            </div>

            <form className="user-event-form">
              <div className="location-grid">
                <label>
                  Name
                  <input/>
                </label>

                <label>
                  Description
                  <input/>
                </label>

                <label>
                  ID
                  <input />
                </label>

                <label>
                  Type of Event
                  <input />
                </label>

                <label>
                  Address
                  <input />
                </label>

                <label>
                  City
                  <input />
                </label>

                <label>
                  Country
                  <input />
                </label>

                <label>
                  Postal Code
                  <input type="number" maxLength={5} />
                </label>

                <label>
                  Starts Before
                  <input type="datetime-local" />
                </label>

                <label>
                  Starts After
                  <input type="datetime-local" />
                </label>

                <label>
                  Ends Before
                  <input type="datetime-local" />
                </label>

                <label>
                  Ends After
                  <input type="datetime-local" />
                </label>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};

export default Itinerary;
