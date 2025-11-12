import React, { useState } from "react";
import "../styles/EventCard.css";
import { apiDeleteUserEvent, apiUserEvent } from "../api/itinerary";
import { useNavigate } from "react-router-dom";
import { sanitize } from "../helpers/itinerary";
import type {
  Event,
  UserEventRequest,
  DayItinerary
} from "../models/itinerary";

interface EventCardProps {
  draggable: boolean;
  time?: string;
  event: Event;

  localDays: DayItinerary[];
  setLocalDays: React.Dispatch<React.SetStateAction<DayItinerary[]>>;
  unassignedEvents: Event[];
  setUnassignedEvents: React.Dispatch<React.SetStateAction<Event[]>>;

  // Added handlers for drag logic
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  onDragStart?: (e: React.DragEvent, eventData: any) => void;
  onDragEnd?: (e: React.DragEvent) => void;
}

const EventCard: React.FC<EventCardProps> = ({
  time,
  event,
  draggable = false,
  localDays,
  setLocalDays,
  unassignedEvents,
  setUnassignedEvents,
  onDragStart,
  onDragEnd
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [isDragging, setIsDragging] = useState(false);
  const [eventData, setEventData] = useState(event);
  const [inputEvent, setInputEvent] = useState(JSON.parse(JSON.stringify(event)));

  const navigate = useNavigate();

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

  const onSaveUserEvent = async () => {
    const userEvent: UserEventRequest = {
      id: eventData.id,
      event_name: sanitize(inputEvent.event_name) ?? "", //TODO: name must not be null or empty, so we could handle the error before sending the request
      event_description: sanitize(inputEvent.event_description),
      event_type: sanitize(inputEvent.event_type),
      street_address: sanitize(inputEvent.street_address),
      city: sanitize(inputEvent.city),
      country: sanitize(inputEvent.country),
      postal_code: inputEvent.postal_code,
      hard_start: inputEvent.hard_start?.substring(0, 19) ?? null,
      hard_end: inputEvent.hard_end?.substring(0, 19) ?? null
    };
    const result = await apiUserEvent(userEvent);
    if (result.status === 401) {
      navigate("/login");
      return;
    } else if (result.result === null || result.status !== 200) {
      alert("Error updaing user-event - TODO: handle error properly");
      return;
    }
    setEventData(inputEvent);
    setIsOpen(false);
  };

  const onDeleteUserEvent = async () => {
    const result = await apiDeleteUserEvent(event.id);
    if (result.status === 401) {
      navigate("/login");
    } else if (result.status !== 200) {
      alert("User-event could not be deleted - TODO handle error properly");
      return;
    }
    unassignedEvents = unassignedEvents.filter((e) => e.id !== event.id);
    setUnassignedEvents(unassignedEvents);
    localDays = localDays.map((d) => {
      return {
        ...d,
        timeBlocks: d.timeBlocks.map((b) => {
          return { ...b, events: b.events.filter((e) => e.id !== event.id) };
        })
      };
    });
    setLocalDays(localDays);
    setIsOpen(false);
  };

  return (
    <>
      <div
        className={`event-card ${draggable ? "draggable" : ""}`}
        draggable={draggable}
        onDragStart={handleDragStart}
        onDragEnd={handleDragEnd}
        onClick={openModal}
      >
        <h3 className="event-title">{eventData.event_name}</h3>
        {eventData.event_type && (
          <p className="event-type">{eventData.event_type}</p>
        )}
        {(eventData.street_address ||
          eventData.city ||
          eventData.country ||
          eventData.postal_code) && (
          <p className="event-location">{formatAddress()}</p>
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
                <button className="card-edit-button" onClick={onSaveUserEvent}>
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
              <div className="editable-card-contents">
                <h2>
                  <strong>Name:</strong>{" "}
                  <input
                    type="text"
                    value={inputEvent.event_name}
                    onChange={(e) =>
                      setInputEvent({...inputEvent, event_name: e.target.value})
                    }
                  />
                </h2>
                <textarea
                  value={inputEvent.event_description || ""}
                  onChange={(e) =>
                    setInputEvent({...inputEvent, event_description: e.target.value})
                  }
                  placeholder="Description"
                />
                <p>
                  <strong>Type:</strong>{" "}
                  <input
                    type="text"
                    value={inputEvent.event_type || ""}
                    onChange={(e) =>
                      setInputEvent({...inputEvent, event_type: e.target.value})
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
                        setInputEvent({...inputEvent, street_address: e.target.value})
                      }
                    />
                  </p>
                  <p>
                    <strong>City:</strong>{" "}
                    <input
                      type="text"
                      value={inputEvent.city || ""}
                      onChange={(e) =>
                        setInputEvent({...inputEvent, city: e.target.value})
                      }
                    />
                  </p>
                  <p>
                    <strong>Country:</strong>{" "}
                    <input
                      type="text"
                      value={inputEvent.country || ""}
                      onChange={(e) =>
                        setInputEvent({...inputEvent, country: e.target.value})
                      }
                    />
                  </p>
                  <p>
                    <strong>Postal Code:</strong>{" "}
                    <input
                      type="text"
                      value={inputEvent.postal_code || ""}
                      onChange={(e) =>
                        setInputEvent({...inputEvent, postal_code: e.target.value})
                      }
                    />
                  </p>
                  <p>
                    {/*NOTICE! Input elements must use the browser's timezone*/}
                    <strong>
                      Start ({Intl.DateTimeFormat().resolvedOptions().timeZone}):
                    </strong>{" "}
                    <input
                      value={
                        inputEvent.hard_start
                          ? new Date(
                              inputEvent.hard_start.substring(0, 19) + "Z"
                            )
                              .toLocaleString("sv-SE", {
                                timeZoneName: "short"
                              })
                              .slice(0, 16)
                          : ""
                      }
                      type="datetime-local"
                      onChange={(e) =>
                        setInputEvent({...inputEvent, hard_start: new Date(e.target.value).toISOString()})
                      }
                    />
                  </p>
                  <p>
                    {/*NOTICE! Input elements must use the browser's timezone*/}
                    <strong>
                      End ({Intl.DateTimeFormat().resolvedOptions().timeZone}):
                    </strong>{" "}
                    <input
                      value={
                        inputEvent.hard_end
                          ? new Date(inputEvent.hard_end.substring(0, 19) + "Z")
                              .toLocaleString("sv-SE", {
                                timeZoneName: "short"
                              })
                              .slice(0, 16)
                          : ""
                      }
                      type="datetime-local"
                      onChange={(e) =>
                        setInputEvent({...inputEvent, hard_end: new Date(e.target.value).toISOString()})
                      }
                    />
                  </p>
                </div>
              </div>
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
                    <strong>Start (UTC):</strong> {eventData.hard_start}
                  </p>
                )}
                {eventData.hard_end && (
                  <p>
                    <strong>End (UTC):</strong> {eventData.hard_end}
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
