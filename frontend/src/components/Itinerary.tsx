import React, { useEffect, useState } from "react";
import EventCard from "./EventCard";
import {
  TIMEZONES,
  type DayItinerary,
  type Event,
  type SearchEventRequest,
  type UserEventRequest
} from "../models/itinerary";
import "../styles/Itinerary.css";
import { apiSearchEvent, apiUserEvent } from "../api/itinerary";
import { useNavigate } from "react-router-dom";
import { sanitize, canDropEventInTimeBlock, getTimeBlockFromTimestamp, getDateFromTimestamp } from "../helpers/itinerary";

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
  const [userEventForm, setUserEventForm] = useState({
    name: "",
    description: "",
    type: "",
    address: "",
    city: "",
    country: "",
    postalCode: "",
    start: "",
    end: "",
    timezoneIndex: -1
  });
  const [searchEventForm, setSearchEventForm] = useState({
    name: "",
    description: "",
    id: "",
    type: "",
    address: "",
    city: "",
    country: "",
    postalCode: "",
    startsBefore: "",
    startsAfter: "",
    endsBefore: "",
    endsAfter: "",
    timezoneIndex: -1
  });
  const [searchResult, setSearchResult] = useState<Event[] | null>(null);
  const [searchResultCaption, setSearchResultCaption] = useState<string>("");

  const navigate = useNavigate();

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
        hard_start: null,
        hard_end: null,
        timezone: null
      };
    }

    //  checks to see if being dropped in a time block, and the event has a hard start
    if (targetTimeIndex >= 0 && draggedEvent.hard_start) {
		const currentDay = localDays[selectedDayIndex];
		const targetTimeBlock = currentDay.timeBlocks[targetTimeIndex].time;
		const targetDate = currentDay.date;
		
		if (!canDropEventInTimeBlock(draggedEvent, targetTimeBlock, targetDate, targetTimeIndex)) {
			const requiredTimeBlock = getTimeBlockFromTimestamp(draggedEvent.hard_start);
			const requiredDate = getDateFromTimestamp(draggedEvent.hard_start);
			alert(`"${draggedEvent.event_name}" has a fixed start time and must be placed in the ${requiredTimeBlock} block on ${requiredDate}.`);
			return;
		}
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
        return "4:00 AM - 12:00 PM";
      case "Afternoon":
        return "12:00 PM - 6:00 PM";
      case "Evening":
        return "6:00 PM - 4:00 AM";
      default:
        return "";
    }
  };

  const onCreateEvent = () => setCreateModalOpen(true);
  const closeCreateModal = () => setCreateModalOpen(false);
  const onSaveUserEvent = async () => {
    const userEvent: UserEventRequest = {
      id: null,
      event_name: sanitize(userEventForm.name)!,
      event_description: sanitize(userEventForm.description),
      event_type: sanitize(userEventForm.type),
      street_address: sanitize(userEventForm.address),
      city: sanitize(userEventForm.city),
      country: sanitize(userEventForm.country),
      postal_code:
        userEventForm.postalCode && userEventForm.postalCode.trim() !== ""
          ? parseInt(userEventForm.postalCode)
          : null,
      hard_start: sanitize(userEventForm.start),
      hard_end: sanitize(userEventForm.end),
      timezone:
        userEventForm.timezoneIndex === -1
          ? null
          : TIMEZONES[userEventForm.timezoneIndex]
    };
    const result = await apiUserEvent(userEvent);
    if (result.status === 401) {
      navigate("/login");
      return;
    } else if (result.result === null || result.status !== 200) {
      alert("TODO: handle error properly - could not create user event");
      return;
    }
    setUserEventForm({
      name: "",
      description: "",
      type: "",
      address: "",
      city: "",
      country: "",
      postalCode: "",
      start: "",
      end: "",
      timezoneIndex: -1
    });
    const event = userEvent as Event;
    event.id = result.result.id;
    event.user_created = true;
    unassignedEvents.push(event);
    setUnassignedEvents(unassignedEvents);
    setCreateModalOpen(false);
  };

  const onSearchEvents = () => setSearchModalOpen(true);
  const closeSearchModal = () => setSearchModalOpen(false);
  const onSearchSend = async () => {
    const searchEvent: SearchEventRequest = {
      id:
        searchEventForm.id && searchEventForm.id.trim() !== ""
          ? parseInt(searchEventForm.id)
          : null,
      street_address: sanitize(searchEventForm.address),
      postal_code:
        searchEventForm.postalCode && searchEventForm.postalCode.trim() !== ""
          ? parseInt(searchEventForm.postalCode)
          : null,
      city: sanitize(searchEventForm.city),
      country: sanitize(searchEventForm.country),
      event_type: sanitize(searchEventForm.type),
      event_description: sanitize(searchEventForm.description),
      event_name: sanitize(searchEventForm.name),
      hard_start_before: sanitize(searchEventForm.startsBefore),
      hard_start_after: sanitize(searchEventForm.startsAfter),
      hard_end_before: sanitize(searchEventForm.endsBefore),
      hard_end_after: sanitize(searchEventForm.endsAfter),
      timezone:
        searchEventForm.timezoneIndex === -1
          ? null
          : TIMEZONES[searchEventForm.timezoneIndex]
    };
    const result = await apiSearchEvent(searchEvent);
    if (result.status === 401) {
      navigate("/login");
      return;
    } else if (result.result === null || result.status !== 200) {
      setSearchResultCaption("Error Searching Events");
      setSearchResult([]);
      return;
    }
    const displayEvents = result.result.events.filter(
      (e) => !unassignedEvents.some((v) => v.id === e.id)
    );
    if (result.result.events.length === 0) {
      setSearchResultCaption("No events found that match your filter");
    } else if (displayEvents.length === 0) {
      setSearchResultCaption(
        "You already have all events matching these filters"
      );
    } else {
      setSearchResultCaption("Add events to your itinerary");
    }
    setSearchResult(displayEvents);
  };

  const formatAddress = (event: Event): string => {
    let addr = "";
    //nested 'if' hell
    if (event.street_address) {
      addr += event.street_address;
      if (event.city || event.country) {
        addr += ", ";
      }
    }
    if (event.city) {
      addr += event.city;
      if (event.country) {
        addr += ", ";
      }
    }
    if (event.country) {
      addr += event.country;
    }
    if (event.postal_code) {
      addr = (addr + " " + event.postal_code).trim();
    }
    if (addr === "") {
      addr = "N/A";
    }
    return addr;
  };

  const addEventFromSearch = (event: Event) => {
    if (!unassignedEvents.some((e) => e.id === event.id)) {
      unassignedEvents.push(event);
      setUnassignedEvents(unassignedEvents);
    }
    setSearchResult(searchResult!.filter((e) => e.id !== event.id));
  };

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
            >
              Create Event
            </button>
            <button
              className="itinerary-edit-button"
              id="itinerary-search"
              onClick={onSearchEvents}
            >
              Search Events
            </button>
            <button
              className="itinerary-edit-button"
              id="itinerary-save"
              onClick={handleSave}
              disabled={buttonsDisabled}
            >
              Save
            </button>
            <button
              className="itinerary-edit-button"
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
                  event={event}
                  unassignedEvents={unassignedEvents}
                  setUnassignedEvents={setUnassignedEvents}
                  localDays={localDays}
                  setLocalDays={setLocalDays}
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
            key={day.date.toString()}
            className={`day-tab ${index === selectedDayIndex ? "active" : ""}`}
            onClick={() => setSelectedDayIndex(index)}
          >
            Day {index + 1} ({day.date.toString()})
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
                  time={block.time}
                  event={event}
                  unassignedEvents={unassignedEvents}
                  setUnassignedEvents={setUnassignedEvents}
                  localDays={localDays}
                  setLocalDays={setLocalDays}
                  draggable={editMode ?? false}
                  onDragStart={(e) => onDragStart(e, event, timeIndex)}
                />
              ))}
            </div>
          </div>
        ))}
      </div>

      {createModalOpen && (
        <div className="user-event-modal-overlay" onClick={closeCreateModal}>
          <div
            className="user-event-modal"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-header">
              <h2>Create a custom event</h2>
              <div className="event-card-buttons">
                <button
                  className="card-save-button"
                  onClick={onSaveUserEvent}
                  title="Save"
                  form="user-event-form"
                  type="submit"
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="18"
                    height="18"
                    fill="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path d="M17 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V7l-4-4zM5 19V5h11v4h4v10H5z" />
                    <path d="M12 12a2 2 0 1 0 0-4 2 2 0 0 0 0 4zM6 18h12v-2H6v2z" />
                  </svg>
                </button>
                <button
                  className="close-button"
                  onClick={closeCreateModal}
                  title="Close"
                >
                  ✕
                </button>
              </div>
            </div>

            <form
              id="user-event-form"
              className="user-event-form"
              onSubmit={onSaveUserEvent}
            >
              <label>
                Name
                <input
                  onChange={(e) =>
                    setUserEventForm({ ...userEventForm, name: e.target.value })
                  }
                  required
                />
              </label>

              <label>
                Description
                <textarea
                  onChange={(e) =>
                    setUserEventForm({
                      ...userEventForm,
                      description: e.target.value
                    })
                  }
                  rows={4}
                />
              </label>

              <label>
                Type of Event
                <input
                  onChange={(e) =>
                    setUserEventForm({ ...userEventForm, type: e.target.value })
                  }
                />
              </label>

              <div className="location-grid">
                <div className="location-grid-row">
                  <label>
                    <span>Address</span>
                    <input
                      onChange={(e) =>
                        setUserEventForm({
                          ...userEventForm,
                          address: e.target.value
                        })
                      }
                    />
                  </label>
                  <label>
                    <span>City</span>
                    <input
                      onChange={(e) =>
                        setUserEventForm({
                          ...userEventForm,
                          city: e.target.value
                        })
                      }
                    />
                  </label>
                </div>

                <div className="location-grid-row">
                  <label>
                    <span>Country</span>
                    <input
                      onChange={(e) =>
                        setUserEventForm({
                          ...userEventForm,
                          country: e.target.value
                        })
                      }
                    />
                  </label>
                  <label>
                    <span>Postal Code</span>
                    <input
                      onChange={(e) =>
                        setUserEventForm({
                          ...userEventForm,
                          postalCode: e.target.value
                        })
                      }
                      type="number"
                      maxLength={5}
                    />
                  </label>
                </div>

                <div className="location-grid-row">
                  <label>
                    <span>Start Time</span>
                    <input
                      type="datetime-local"
                      onChange={(e) => {
                        let start = e.target.value;
                        if (start !== "") {
                          start += ":00";
                        }
                        setUserEventForm({ ...userEventForm, start });
                      }}
                    />
                  </label>
                  <label>
                    <span>End Time</span>
                    <input
                      type="datetime-local"
                      onChange={(e) => {
                        let end = e.target.value;
                        if (end !== "") {
                          end += ":00";
                        }
                        setUserEventForm({ ...userEventForm, end });
                      }}
                    />
                  </label>
                </div>
              </div>

              {(userEventForm.start || userEventForm.end) && (
                <label>
                  <span>Timezone</span>
                  <select
                    value={userEventForm.timezoneIndex}
                    onChange={(e) =>
                      setUserEventForm({
                        ...userEventForm,
                        timezoneIndex: +e.target.value
                      })
                    }
                  >
                    {[
                      <option key={-1} value={-1}>
                        No Timezone Selected
                      </option>,
                      ...TIMEZONES.map((tz, index) => (
                        <option key={index} value={index}>
                          {tz}
                        </option>
                      ))
                    ]}
                  </select>
                </label>
              )}
            </form>
          </div>
        </div>
      )}

      {searchModalOpen && (
        <div className="user-event-modal-overlay" onClick={closeSearchModal}>
          <div
            className="search-event-modal"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="searchContainer">
              <div className="searchFilters">
                <div className="modal-header">
                  <h2>Search for an event</h2>
                  <div className="event-card-buttons">
                    <button
                      className="card-save-button"
                      onClick={onSearchSend}
                      title="Search"
                    >
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="18"
                        height="18"
                        fill="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path d="M10 2a8 8 0 1 0 5.293 14.293l4.707 4.707 1.414-1.414-4.707-4.707A8 8 0 0 0 10 2zm0 2a6 6 0 1 1 0 12A6 6 0 0 1 10 4z" />
                      </svg>
                    </button>
                    <button
                      className="close-button"
                      onClick={closeSearchModal}
                      title="Close"
                    >
                      ✕
                    </button>
                  </div>
                </div>

                <form className="user-event-form" onSubmit={onSearchSend}>
                  <div className="location-grid">
                    <label>
                      Name
                      <input
                        onChange={(e) =>
                          setSearchEventForm({
                            ...searchEventForm,
                            name: e.target.value
                          })
                        }
                      />
                    </label>

                    <label>
                      Description
                      <input
                        onChange={(e) =>
                          setSearchEventForm({
                            ...searchEventForm,
                            description: e.target.value
                          })
                        }
                      />
                    </label>

                    <label>
                      ID
                      <input
                        onChange={(e) =>
                          setSearchEventForm({
                            ...searchEventForm,
                            id: e.target.value
                          })
                        }
                        type="number"
                      />
                    </label>

                    <label>
                      Type of Event
                      <input
                        onChange={(e) =>
                          setSearchEventForm({
                            ...searchEventForm,
                            type: e.target.value
                          })
                        }
                      />
                    </label>

                    <label>
                      Address
                      <input
                        onChange={(e) =>
                          setSearchEventForm({
                            ...searchEventForm,
                            address: e.target.value
                          })
                        }
                      />
                    </label>

                    <label>
                      City
                      <input
                        onChange={(e) =>
                          setSearchEventForm({
                            ...searchEventForm,
                            city: e.target.value
                          })
                        }
                      />
                    </label>

                    <label>
                      Country
                      <input
                        onChange={(e) =>
                          setSearchEventForm({
                            ...searchEventForm,
                            country: e.target.value
                          })
                        }
                      />
                    </label>

                    <label>
                      Postal Code
                      <input
                        onChange={(e) =>
                          setSearchEventForm({
                            ...searchEventForm,
                            postalCode: e.target.value
                          })
                        }
                        type="number"
                        maxLength={5}
                      />
                    </label>

                    <label>
                      Starts Before
                      <input
                        onChange={(e) => {
                          let startsBefore = e.target.value;
                          if (startsBefore !== "") {
                            startsBefore += ":00";
                          }
                          setSearchEventForm({
                            ...searchEventForm,
                            startsBefore
                          });
                        }}
                        type="datetime-local"
                      />
                    </label>

                    <label>
                      Starts After
                      <input
                        onChange={(e) => {
                          let startsAfter = e.target.value;
                          if (startsAfter !== "") {
                            startsAfter += ":00";
                          }
                          setSearchEventForm({
                            ...searchEventForm,
                            startsAfter
                          });
                        }}
                        type="datetime-local"
                      />
                    </label>

                    <label>
                      Ends Before
                      <input
                        onChange={(e) => {
                          let endsBefore = e.target.value;
                          if (endsBefore !== "") {
                            endsBefore += ":00";
                          }
                          setSearchEventForm({
                            ...searchEventForm,
                            endsBefore
                          });
                        }}
                        type="datetime-local"
                      />
                    </label>

                    <label>
                      Ends After
                      <input
                        onChange={(e) => {
                          let endsAfter = e.target.value;
                          if (endsAfter !== "") {
                            endsAfter += ":00";
                          }
                          setSearchEventForm({ ...searchEventForm, endsAfter });
                        }}
                        type="datetime-local"
                      />
                    </label>

                    <label>
                      <span>Timezone</span>
                      <select
                        onChange={(e) =>
                          setSearchEventForm({
                            ...searchEventForm,
                            timezoneIndex: +e.target.value
                          })
                        }
                      >
                        {[
                          <option key={-1} value={-1}>
                            No Timezone Selected
                          </option>,
                          ...TIMEZONES.map((tz, index) => (
                            <option key={index} value={index}>
                              {tz}
                            </option>
                          ))
                        ]}
                      </select>
                    </label>
                  </div>
                </form>
              </div>
              {searchResult && (
                <div className="searchResults">
                  <h2>{searchResultCaption}</h2>
                  <div className="resultsGrid">
                    {searchResult.map((event: Event) => (
                      <div key={event.id} className="resultCard">
                        <button
                          className="card-edit-button"
                          onClick={() => addEventFromSearch(event)}
                        >
                          +
                        </button>
                        <h3 className="event-title">{event.event_name}</h3>
                        {(event.street_address ||
                          event.city ||
                          event.country ||
                          event.postal_code) && (
                          <p className="event-location">
                            {formatAddress(event)}
                          </p>
                        )}
                        {event.event_type && (
                          <p className="event-type">{event.event_type}</p>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default Itinerary;
