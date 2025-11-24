import React, { useState } from "react";
import "../styles/EventCard.css";
import { apiDeleteUserEvent, apiUserEvent } from "../api/itinerary";
import { useNavigate } from "react-router-dom";
import { sanitize } from "../helpers/itinerary";
import {
  type Event,
  type UserEventRequest,
  type DayItinerary,
  TIMEZONES
} from "../models/itinerary";

interface EventCardProps {
  draggable: boolean;
  time?: string;
  event: Event;
  displayTime?: string;
  imageOnLeft?: boolean;
  variant?: "workspace" | "timeline";

  localDays: DayItinerary[];
  setLocalDays: React.Dispatch<React.SetStateAction<DayItinerary[]>>;
  unassignedEvents: Event[];
  setUnassignedEvents: React.Dispatch<React.SetStateAction<Event[]>>;

  // Callbacks to notify parent of changes
  onDaysUpdate?: (updatedDays: DayItinerary[]) => void;
  onUnassignedUpdate?: (updatedUnassigned: Event[]) => void;

  // Added handlers for drag logic
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  onDragStart?: (e: React.DragEvent, eventData: any) => void;
  onDragEnd?: (e: React.DragEvent) => void;
}

const EventCard: React.FC<EventCardProps> = ({
  time,
  event,
  draggable = false,
  displayTime,
  imageOnLeft = true,
  variant = "timeline",
  localDays,
  setLocalDays,
  unassignedEvents,
  setUnassignedEvents,
  onDaysUpdate,
  onUnassignedUpdate,
  onDragStart,
  onDragEnd
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [isDragging, setIsDragging] = useState(false);
  const [eventData, setEventData] = useState(event);
  const [inputEvent, setInputEvent] = useState({
    ...JSON.parse(JSON.stringify(event)),
    timezoneIndex: TIMEZONES.findIndex((tz) => tz === event.timezone)
  });

  const navigate = useNavigate();

  const getDateTimeLabel = () => {
    if (eventData.hard_start) {
      try {
        const date = new Date(eventData.hard_start);
        if (!isNaN(date.getTime())) {
          const today = new Date();
          const sameYear = date.getFullYear() === today.getFullYear();
          const options: Intl.DateTimeFormatOptions = {
            month: "short",
            day: "numeric",
            hour: "numeric",
            minute: "2-digit"
          };
          if (!sameYear) {
            options.year = "numeric";
          }
          return date.toLocaleString(undefined, options);
        }
      } catch {
        // fall through
      }
    }
    return displayTime || "";
  };

  const openModal = () => {
    if (!isDragging) setIsOpen(true);
  };

  const closeModal = () => {
    setInputEvent(eventData);
    setIsOpen(false);
  };

  const handleDragStart = (e: React.DragEvent) => {
    setIsDragging(true);
    if (onDragStart) {
      onDragStart(e, {
        event_name: eventData.event_name,
        event_description: eventData.event_description,
        time
      });
    }
  };

  const handleDragEnd = (e: React.DragEvent) => {
    setIsDragging(false);
    if (onDragEnd) {
      onDragEnd(e);
    }
  };

  const formatAddress = () => {
    let addr = "";
    //nested 'if' hell
    if (eventData.street_address) {
      addr += eventData.street_address;
      if (eventData.city || eventData.country) {
        addr += ", ";
      }
    }
    if (eventData.city) {
      addr += eventData.city;
      if (eventData.country) {
        addr += ", ";
      }
    }
    if (eventData.country) {
      addr += eventData.country;
    }
    if (eventData.postal_code) {
      addr = (addr + " " + eventData.postal_code).trim();
    }
    if (addr === "") {
      addr = "N/A";
    }
    return addr;
  };

  const onSaveUserEvent = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const userEvent: UserEventRequest = {
      id: eventData.id,
      event_name: sanitize(inputEvent.event_name)!,
      event_description: sanitize(inputEvent.event_description),
      event_type: sanitize(inputEvent.event_type),
      street_address: sanitize(inputEvent.street_address),
      city: sanitize(inputEvent.city),
      country: sanitize(inputEvent.country),
      postal_code: inputEvent.postal_code,
      hard_start: sanitize(inputEvent.hard_start),
      hard_end: sanitize(inputEvent.hard_end),
      timezone:
        inputEvent.timezoneIndex === -1
          ? null
          : TIMEZONES[inputEvent.timezoneIndex]
    };
    const result = await apiUserEvent(userEvent);
    if (result.status === 401) {
      navigate("/login");
      return;
    } else if (result.result === null || result.status !== 200) {
      alert("TODO: handle error properly - could not update user event");
      return;
    }
    setEventData(userEvent as Event);
    event.city = userEvent.city;
    event.country = userEvent.country;
    event.event_description = userEvent.event_description;
    event.event_name = userEvent.event_name;
    event.event_type = userEvent.event_type;
    event.hard_end = userEvent.hard_end;
    event.hard_start = userEvent.hard_start;
    event.postal_code = userEvent.postal_code;
    event.street_address = userEvent.street_address;
    event.timezone = userEvent.timezone;
    setIsOpen(false);
  };

  const onDeleteUserEvent = async () => {
    const result = await apiDeleteUserEvent(event.id);
    if (result.status === 401) {
      navigate("/login");
    } else if (result.status !== 200) {
      alert("TODO: handle error properly - could not delete user event");
      return;
    }
    const updatedUnassigned = unassignedEvents.filter((e) => e.id !== event.id);
    setUnassignedEvents(updatedUnassigned);
    const updatedDays = localDays.map((d) => {
      return {
        ...d,
        timeBlocks: d.timeBlocks.map((b) => {
          return { ...b, events: b.events.filter((e) => e.id !== event.id) };
        })
      };
    });
    setLocalDays(updatedDays);
    
    // Notify parent of changes
    if (onUnassignedUpdate) {
      onUnassignedUpdate(updatedUnassigned);
    }
    if (onDaysUpdate) {
      onDaysUpdate(updatedDays);
    }
    
    setIsOpen(false);
  };

  const variantClass =
    variant === "workspace" ? "event-card--workspace" : "";

  return (
    <>
      <div
        className={`event-card ${variantClass} ${
          draggable ? "draggable" : ""
        } ${imageOnLeft ? "image-left" : "image-right"}`}
        draggable={draggable}
        onDragStart={handleDragStart}
        onDragEnd={handleDragEnd}
      >
        <div className="event-image-container">
          <div className="event-image-placeholder">
            <svg
              width="100%"
              height="100%"
              viewBox="0 0 200 200"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <rect width="200" height="200" fill="#f0f0f0" />
              <path
                d="M80 70L100 90L120 70L140 90L160 70V150H40V70L80 70Z"
                fill="#d0d0d0"
              />
              <circle cx="70" cy="60" r="15" fill="#d0d0d0" />
            </svg>
          </div>
        </div>
        <div className="event-content">
          {/* Title should always appear above the date/time */}
          <h3 className="event-title">{eventData.event_name}</h3>

          {getDateTimeLabel() && (
            <p className="event-datetime">{getDateTimeLabel()}</p>
          )}

          {variant !== "workspace" && eventData.event_description && (
            <p className="event-description">{eventData.event_description}</p>
          )}

          {variant !== "workspace" && eventData.event_type && (
            <p className="event-type">{eventData.event_type}</p>
          )}

          {variant !== "workspace" &&
            (eventData.street_address ||
              eventData.city ||
              eventData.country ||
              eventData.postal_code) && (
              <p className="event-location">{formatAddress()}</p>
            )}
        </div>
        
        {/* Delete button for workspace events */}
        {variant === "workspace" && (
          <button
            className="event-card-delete-btn"
            onClick={(e) => {
              e.stopPropagation();
              if (event.user_created) {
                onDeleteUserEvent();
              } else {
                // For non-user-created events, just remove from workspace
                const updatedUnassigned = unassignedEvents.filter((ev) => ev.id !== event.id);
                setUnassignedEvents(updatedUnassigned);
                if (onUnassignedUpdate) {
                  onUnassignedUpdate(updatedUnassigned);
                }
              }
            }}
            title="Remove from workspace"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="20"
              height="20"
              fill="currentColor"
              viewBox="0 0 24 24"
            >
              <path d="M3 6H21" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
              <path d="M19 6V20C19 21 18 22 17 22H7C6 22 5 21 5 20V6" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
              <path d="M8 6V4C8 3 9 2 10 2H14C15 2 16 3 16 4V6" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
              <path d="M10 11V17" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
              <path d="M14 11V17" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </button>
        )}
      </div>

      {isOpen && (
        <div className="event-modal-overlay" onClick={closeModal}>
          <div className="event-modal" onClick={(e) => e.stopPropagation()}>
            <div className="event-card-buttons">
              {event.user_created && (
                <button
                  className="card-edit-button"
                  id="user-card-delete"
                  onClick={onDeleteUserEvent}
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="20"
                    height="20"
                    fill="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path d="M9 3v1H4v2h16V4h-5V3H9zm1 5v10h2V8h-2zm4 0v10h2V8h-2zM6 8v12a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V8H6z" />
                  </svg>
                </button>
              )}
              {event.user_created && (
                <button
                  className="card-edit-button"
                  form="editable-card-contents"
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
              )}
              <button className="close-button" onClick={closeModal}>
                ✕
              </button>
            </div>
            {eventData.user_created ? (
              <form
                id="editable-card-contents"
                className="editable-card-contents"
                onSubmit={onSaveUserEvent}
              >
                <h2>
                  <strong>Name:</strong>{" "}
                  <input
                    type="text"
                    value={inputEvent.event_name}
                    required
                    onChange={(e) =>
                      setInputEvent({
                        ...inputEvent,
                        event_name: e.target.value
                      })
                    }
                  />
                </h2>
                <textarea
                  value={inputEvent.event_description || ""}
                  onChange={(e) =>
                    setInputEvent({
                      ...inputEvent,
                      event_description: e.target.value
                    })
                  }
                  placeholder="Description"
                />
                <p>
                  <strong>Type:</strong>{" "}
                  <input
                    type="text"
                    value={inputEvent.event_type || ""}
                    onChange={(e) =>
                      setInputEvent({
                        ...inputEvent,
                        event_type: e.target.value
                      })
                    }
                  />
                </p>
                <div className="editable-card-components-grid">
                  <p>
                    <strong>Address:</strong>{" "}
                    <input
                      type="text"
                      value={inputEvent.street_address || ""}
                      onChange={(e) =>
                        setInputEvent({
                          ...inputEvent,
                          street_address: e.target.value
                        })
                      }
                    />
                  </p>
                  <p>
                    <strong>City:</strong>{" "}
                    <input
                      type="text"
                      value={inputEvent.city || ""}
                      onChange={(e) =>
                        setInputEvent({ ...inputEvent, city: e.target.value })
                      }
                    />
                  </p>
                  <p>
                    <strong>Country:</strong>{" "}
                    <input
                      type="text"
                      value={inputEvent.country || ""}
                      onChange={(e) =>
                        setInputEvent({
                          ...inputEvent,
                          country: e.target.value
                        })
                      }
                    />
                  </p>
                  <p>
                    <strong>Postal Code:</strong>{" "}
                    <input
                      type="text"
                      value={inputEvent.postal_code || ""}
                      onChange={(e) =>
                        setInputEvent({
                          ...inputEvent,
                          postal_code: e.target.value
                        })
                      }
                    />
                  </p>
                  <p>
                    <strong>Start:</strong>{" "}
                    <input
                      value={inputEvent.hard_start ? inputEvent.hard_start : ""}
                      type="datetime-local"
                      onChange={(e) => {
                        let hard_start = e.target.value;
                        if (hard_start !== "") {
                          hard_start += ":00";
                        }
                        setInputEvent({ ...inputEvent, hard_start });
                      }}
                    />
                  </p>
                  <p>
                    <strong>End:</strong>{" "}
                    <input
                      value={inputEvent.hard_end ? inputEvent.hard_end : ""}
                      type="datetime-local"
                      onChange={(e) => {
                        let hard_end = e.target.value;
                        if (hard_end !== "") {
                          hard_end += ":00";
                        }
                        setInputEvent({ ...inputEvent, hard_end });
                      }}
                    />
                  </p>
                </div>
                {(inputEvent.hard_start || inputEvent.hard_end) && (
                  <p>
                    <strong>Timezone:</strong>
                    <select
                      value={inputEvent.timezoneIndex}
                      onChange={(e) =>
                        setInputEvent({
                          ...inputEvent,
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
                  </p>
                )}
              </form>
            ) : (
              <div className="readonly-card-contents">
                <h2>{eventData.event_name}</h2>
                {eventData.event_description && (
                  <p>{eventData.event_description}</p>
                )}
                {eventData.event_type && (
                  <p>
                    <strong>Type:</strong> {eventData.event_type}
                  </p>
                )}
                {(eventData.street_address ||
                  eventData.city ||
                  eventData.country ||
                  eventData.postal_code) && (
                  <p>
                    <strong>Location:</strong> {formatAddress()}
                  </p>
                )}
                {eventData.hard_start && (
                  <p>
                    <strong>Start:</strong> {eventData.hard_start}
                  </p>
                )}
                {eventData.hard_end && (
                  <p>
                    <strong>End:</strong> {eventData.hard_end}
                  </p>
                )}
                {(eventData.hard_start || eventData.hard_end) &&
                  eventData.timezone && (
                    <p>
                      <strong>Timezone:</strong> {eventData.timezone}
                    </p>
                  )}
              </div>
            )}
          </div>
        </div>
      )}
    </>
  );
};

export default EventCard;
