import React, { useState } from "react";
import "../styles/EventCard.css";
import { apiDeleteUserEvent } from "../api/itinerary";
import { useNavigate } from "react-router-dom";
import type { DayItinerary } from "../helpers/itinerary";
import type { Event } from "../models/itinerary";

interface EventCardProps {
  event_id: number;
  event_name: string;
  event_description: string | null;
  draggable: boolean;
  time?: string;
  street_address: string | null;
  postal_code: number | null;
  city: string | null;
  country: string | null;
  event_type: string | null;
  user_created: boolean;
  hard_start: Date | null;
  hard_end: Date | null;

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
  event_id,
  event_name,
  event_description,
  time,
  street_address,
  city,
  country,
  postal_code,
  event_type,
  hard_start,
  hard_end,
  user_created,
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

  const navigate = useNavigate();

  const openModal = () => {
    if (!isDragging) setIsOpen(true);
  };

  const closeModal = () => setIsOpen(false);

  const handleDragStart = (e: React.DragEvent) => {
    setIsDragging(true);
    if (onDragStart) {
      onDragStart(e, { event_name, event_description, time });
    }
  };

  const handleDragEnd = (e: React.DragEvent) => {
    setIsDragging(false);
    if (onDragEnd) {
      onDragEnd(e);
    }
  };

  const formatDateTime = (date: Date | null | undefined) => {
    if (!date) return null;
    try {
      const dateObj = typeof date === "string" ? new Date(date) : date;
      return dateObj.toLocaleString();
    } catch {
      return null;
    }
  };

  const formatAddress = () => {
    let addr = "";
    //nested 'if' hell
    if (street_address) {
      addr += street_address;
      if (city || country) {
        addr += ", ";
      }
    }
    if (city) {
      addr += city;
      if (country) {
        addr += ", ";
      }
    }
    if (country) {
      addr += country;
    }
    if (postal_code) {
      addr = (addr + " " + postal_code).trim();
    }
    if (addr === "") {
      addr = "N/A";
    }
    return addr;
  };

  const onSaveUserEvent = async () => {
    alert("TODO");
  };

  const onDeleteUserEvent = async () => {
    const result = await apiDeleteUserEvent(event_id);
    if (result.status === 401) {
      navigate("/login");
    } else if (result.status !== 200) {
      alert("User-event could not be deleted - TODO handle error properly");
      return;
    }
    unassignedEvents = unassignedEvents.filter(e => e.id !== event_id);
    setUnassignedEvents(unassignedEvents);
    localDays = localDays.map(d => {
      return {
        ...d, timeBlocks: d.timeBlocks.map(b => {
          return { ...b, events: b.events.filter(e => e.id !== event_id) };
        })
      };
    });
    setLocalDays(localDays);
  }

  return (
    <>
      <div
        className={`event-card ${draggable ? "draggable" : ""}`}
        draggable={draggable}
        onDragStart={handleDragStart}
        onDragEnd={handleDragEnd}
        onClick={openModal}
      >
        <h3 className="event-title">{event_name}</h3>
        {(street_address || city || country || postal_code) && (
          <p className="event-location">
            {formatAddress()}
          </p>
        )}
        {event_type && <p className="event-type">{event_type}</p>}
      </div>

      {isOpen && (
        <div className="event-modal-overlay" onClick={closeModal}>
          <div className="event-modal" onClick={(e) => e.stopPropagation()}>
            <div className="event-card-buttons">
              {user_created && <button className="card-edit-button" onClick={onDeleteUserEvent}>
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="20"
                  height="20"
                  fill="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path d="M9 3v1H4v2h16V4h-5V3H9zm1 5v10h2V8h-2zm4 0v10h2V8h-2zM6 8v12a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V8H6z"/>
                </svg>
              </button>}
              {user_created && <button className="card-edit-button" onClick={onSaveUserEvent}>
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
              </button>}
              <button className="close-button" onClick={closeModal}>
                ✕
              </button>
            </div>
            <h2>{event_name}</h2>
            {event_description && <p>{event_description}</p>}
            {event_type && (
              <p>
                <strong>Type:</strong> {event_type}
              </p>
            )}
            {time && (
              <p>
                <strong>Time:</strong> {time}
              </p>
            )}
            {(street_address || city || country || postal_code) && (
              <p>
                <strong>Location:</strong> {formatAddress()}
              </p>
            )}
            {hard_start && (
              <p>
                <strong>Start:</strong> {formatDateTime(hard_start)}
              </p>
            )}
            {hard_end && (
              <p>
                <strong>End:</strong> {formatDateTime(hard_end)}
              </p>
            )}
          </div>
        </div>
      )}
    </>
  );
};

export default EventCard;
