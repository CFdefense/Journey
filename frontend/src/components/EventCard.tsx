import React, { useState } from "react";
import { createPortal } from "react-dom";
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
  unassignedEvents: Event[];

  // Callbacks to notify parent of changes
  onDaysUpdate: (updatedDays: DayItinerary[]) => void;
  onUnassignedUpdate: (updatedUnassigned: Event[]) => void;

  // Added handlers for drag logic
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  onDragStart?: (e: React.DragEvent, eventData: any) => void;
  onDragEnd?: (e: React.DragEvent) => void;
  onDrop?: (e: React.DragEvent, targetEventId: number) => void;
  onDragOver?: (e: React.DragEvent) => void;
}

const EventCard: React.FC<EventCardProps> = ({
  time,
  event,
  draggable = false,
  imageOnLeft = true,
  variant = "timeline",
  localDays,
  unassignedEvents,
  onDaysUpdate,
  onUnassignedUpdate,
  onDragStart,
  onDragEnd,
  onDrop,
  onDragOver
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [eventData, setEventData] = useState(event);
  const [isDragOver, setIsDragOver] = useState(false);
  const [inputEvent, setInputEvent] = useState({
    ...JSON.parse(JSON.stringify(event)),
    timezoneIndex: TIMEZONES.findIndex((tz) => tz === event.timezone)
  });

  const navigate = useNavigate();

  const getDateTimeLabel = (short: boolean = false) => {
    if (!eventData.hard_start) {
      return undefined;
    }
    const start_date = new Date(eventData.hard_start);
    if (isNaN(start_date.getTime())) {
      return undefined;
    }
    const today = new Date();
    const options: Intl.DateTimeFormatOptions = {
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit"
    };
    const sameYear = start_date.getFullYear() === today.getFullYear();
    if (!sameYear) {
      options.year = "numeric";
    }
    const start_date_display = start_date.toLocaleString(undefined, options);
    if (short) {
      return start_date_display;
    }
    if (!eventData.hard_end) {
      return start_date_display;
    }
    const end_date = new Date(eventData.hard_end);
    if (isNaN(end_date.getTime())) {
      return start_date_display;
    }
    return (
      start_date_display + " - " + end_date.toLocaleString(undefined, options)
    );
  };

  const closeModal = () => {
    setInputEvent(eventData);
    setIsOpen(false);
  };

  const handleDragStart = (e: React.DragEvent) => {
    if (onDragStart) {
      onDragStart(e, {
        event_name: eventData.event_name,
        event_description: eventData.event_description,
        time
      });
    }
  };

  const handleDragEnd = (e: React.DragEvent) => {
    if (onDragEnd) {
      onDragEnd(e);
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);
    if (onDrop) {
      onDrop(e, eventData.id);
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(true);
    if (onDragOver) {
      onDragOver(e);
    }
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);
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
    setEventData({...userEvent, user_created: true} as Event);
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
    (onDaysUpdate!)(localDays);
    setIsOpen(false);
  };

  const onDeleteUserEvent = async () => {
    if (
      !window.confirm(
        `Are you sure you want to delete "${event.event_name}"? This action cannot be undone.`
      )
    ) {
      return;
    }

    const result = await apiDeleteUserEvent(event.id);
    if (result.status === 401) {
      navigate("/login");
    } else if (result.status !== 200) {
      alert("TODO: handle error properly - could not delete user event");
      return;
    }
    const updatedUnassigned = unassignedEvents.filter((e) => e.id !== event.id);
    const updatedDays = localDays.map((d) => {
      return {
        ...d,
        timeBlocks: d.timeBlocks.map((b) => {
          return { ...b, events: b.events.filter((e) => e.id !== event.id) };
        })
      };
    });

    // Notify parent of changes
    onUnassignedUpdate(updatedUnassigned);
    onDaysUpdate(updatedDays);

    setIsOpen(false);
  };

  const variantClass = variant === "workspace" ? "event-card--workspace" : "";

  return (
    <>
      <div
        className={`event-card ${variantClass} ${
          draggable ? "draggable" : ""
        } ${imageOnLeft ? "image-left" : "image-right"}`}
        draggable={draggable}
        onDragStart={handleDragStart}
        onDragEnd={handleDragEnd}
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        data-drag-over={isDragOver}
      >
        {/* Edit button for all event cards */}
        <button
          className="event-card-edit-btn"
          onClick={(e) => {
            e.stopPropagation();
            setIsOpen(true);
          }}
          title={eventData.user_created ? "Edit event" : "Event Details"}
        >
          {eventData.user_created ? (
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
            </svg>
          ) : (
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <circle cx="12" cy="12" r="10" />
              <line x1="12" y1="12" x2="12" y2="18" />
              <circle cx="12" cy="7.5" r=".5" />
            </svg>
          )}
        </button>

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
            <p className="event-datetime">{getDateTimeLabel(true)}</p>
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
              // Just remove from workspace
              const updatedUnassigned = unassignedEvents.filter(
                (ev) => ev.id !== event.id
              );
              onUnassignedUpdate(updatedUnassigned);
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
              <path
                d="M3 6H21"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
              <path
                d="M19 6V20C19 21 18 22 17 22H7C6 22 5 21 5 20V6"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
              <path
                d="M8 6V4C8 3 9 2 10 2H14C15 2 16 3 16 4V6"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
              <path
                d="M10 11V17"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
              <path
                d="M14 11V17"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </button>
        )}
      </div>

      {isOpen &&
        createPortal(
          <div className="event-modal-overlay" onClick={closeModal}>
            <div
              className="user-event-modal"
              onClick={(e) => e.stopPropagation()}
            >
              <div className="modal-header">
                <h2>
                  {eventData.user_created ? "Edit Event" : "Event Details"}
                </h2>
                <div>
                  {eventData.user_created && (
                    <button
                      className="icon-button"
                      onClick={onDeleteUserEvent}
                      aria-label="Close modal"
                      title="Delete permanently"
                    >
                      <svg
                        width="20"
                        height="20"
                        viewBox="0 0 24 24"
                        fill="none"
                        strokeWidth="2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                      >
                        <path
                          d="M3 6H21"
                          stroke="red"
                          strokeWidth="2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                        />
                        <path
                          d="M19 6V20C19 21 18 22 17 22H7C6 22 5 21 5 20V6"
                          stroke="red"
                          strokeWidth="2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                        />
                        <path
                          d="M8 6V4C8 3 9 2 10 2H14C15 2 16 3 16 4V6"
                          stroke="red"
                          strokeWidth="2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                        />
                        <path
                          d="M10 11V17"
                          stroke="red"
                          strokeWidth="2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                        />
                        <path
                          d="M14 11V17"
                          stroke="red"
                          strokeWidth="2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                        />
                      </svg>
                    </button>
                  )}
                  <button
                    className="icon-button"
                    onClick={closeModal}
                    aria-label="Close modal"
                  >
                    <svg
                      width="20"
                      height="20"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                    >
                      <line x1="18" y1="6" x2="6" y2="18"></line>
                      <line x1="6" y1="6" x2="18" y2="18"></line>
                    </svg>
                  </button>
                </div>
              </div>
              {eventData.user_created ? (
                <form
                  id="editable-card-contents"
                  className="user-event-form"
                  onSubmit={onSaveUserEvent}
                >
                  <label>
                    Name
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
                  </label>

                  <label>
                    Description
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
                  </label>

                  <label>
                    Type Of Event
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
                  </label>

                  <div className="location-grid">
                    <div className="location-grid-row">
                      <label>
                        Address
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
                      </label>
                      <label>
                        City
                        <input
                          type="text"
                          value={inputEvent.city || ""}
                          onChange={(e) =>
                            setInputEvent({
                              ...inputEvent,
                              city: e.target.value
                            })
                          }
                        />
                      </label>
                    </div>
                    <div className="location-grid-row">
                      <label>
                        Country
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
                      </label>
                      <label>
                        Postal Code
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
                      </label>
                    </div>
                    <div className="location-grid-row">
                      <label>
                        Start
                        <input
                          value={
                            inputEvent.hard_start ? inputEvent.hard_start : ""
                          }
                          type="datetime-local"
                          onChange={(e) => {
                            let hard_start = e.target.value;
                            if (hard_start !== "") {
                              hard_start += ":00";
                            }
                            setInputEvent({ ...inputEvent, hard_start });
                          }}
                        />
                      </label>
                      <label>
                        End
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
                      </label>
                    </div>
                  </div>
                  {(inputEvent.hard_start || inputEvent.hard_end) && (
                    <label>
                      Timezone
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
                    </label>
                  )}

                  <button
                    type="submit"
                    style={{
                      width: "100%",
                      height: "48px",
                      borderRadius: "12px",
                      marginTop: "16px",
                      background: "linear-gradient(135deg, #006bbb, #2890c8)",
                      border: "none",
                      color: "#ffffff",
                      fontSize: "1rem",
                      fontWeight: "600",
                      cursor: "pointer",
                      transition: "all 0.2s ease",
                      boxShadow: "0 4px 12px rgba(0, 107, 187, 0.3)"
                    }}
                    onMouseEnter={(e) => {
                      e.currentTarget.style.transform = "translateY(-2px)";
                      e.currentTarget.style.boxShadow =
                        "0 6px 16px rgba(0, 107, 187, 0.4)";
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.transform = "translateY(0)";
                      e.currentTarget.style.boxShadow =
                        "0 4px 12px rgba(0, 107, 187, 0.3)";
                    }}
                  >
                    Save Changes
                  </button>
                </form>
              ) : (
                <div
                  className="user-event-form"
                  style={{ pointerEvents: "none" }}
                >
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
          </div>,
          document.body
        )}
    </>
  );
};

export default EventCard;
