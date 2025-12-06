import React, { useState } from "react";
import EventCard from "./EventCard";
import {
  EVENT_DEFAULT,
  TIMEZONES,
  type DayItinerary,
  type Event,
  type SearchEventRequest,
  type UserEventRequest
} from "../models/itinerary";
import "../styles/Itinerary.css";
import { apiSearchEvent, apiUserEvent } from "../api/itinerary";
import { useNavigate } from "react-router-dom";
import {
  sanitize,
  canDropEventInTimeBlock,
  getTimeBlockFromTimestamp,
  getDateFromTimestamp
} from "../helpers/itinerary";
import { toast } from "./Toast";

interface ItineraryProps {
  localDays: DayItinerary[];
  unassigned: Event[];
  onUpdate: (updatedDays: DayItinerary[]) => void;
  onUnassignedUpdate: (unassignedEvents: Event[]) => void;
  editMode?: boolean;
  title?: string;
  compact?: boolean;
  externalCreateModal?: boolean;
  externalSearchModal?: boolean;
  onCreateModalChange?: (open: boolean) => void;
  onSearchModalChange?: (open: boolean) => void;
}

const Itinerary: React.FC<ItineraryProps> = ({
  localDays,
  unassigned,
  onUpdate,
  onUnassignedUpdate,
  editMode,
  title,
  compact = false,
  externalCreateModal,
  externalSearchModal,
  onCreateModalChange,
  onSearchModalChange
}) => {
  const [selectedDayIndex, setSelectedDayIndex] = useState(0);
  const [internalCreateModalOpen, setInternalCreateModalOpen] = useState(false);
  const [internalSearchModalOpen, setInternalSearchModalOpen] = useState(false);
  const [isDragging, setIsDragging] = useState(false);
  const [imagePreview, setImagePreview] = useState<string | null>(null);


  // Use external modal state if provided, otherwise use internal state
  const createModalOpen =
    externalCreateModal !== undefined
      ? externalCreateModal
      : internalCreateModalOpen;
  const searchModalOpen =
    externalSearchModal !== undefined
      ? externalSearchModal
      : internalSearchModalOpen;

  const setCreateModalOpen = (open: boolean) => {
    if (onCreateModalChange) {
      onCreateModalChange(open);
    } else {
      setInternalCreateModalOpen(open);
    }
  };

  const setSearchModalOpen = (open: boolean) => {
    if (onSearchModalChange) {
      onSearchModalChange(open);
    } else {
      setInternalSearchModalOpen(open);
    }
  };

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
    timezoneIndex: -1,
    photoName: ""
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

  const onDragStart = (e: React.DragEvent, event: Event, timeIndex: number) => {
    e.dataTransfer.setData("eventId", event.id.toString());
    e.dataTransfer.setData("eventName", event.event_name);
    e.dataTransfer.setData("eventDescription", event.event_description || "");
    e.dataTransfer.setData("sourceTimeIndex", timeIndex.toString());
    setIsDragging(true);
  };

  const onDragEnd = () => {
    setIsDragging(false);
  };

  const onDrop = (
    e: React.DragEvent,
    targetTimeIndex: number,
    targetEventId?: number
  ) => {
    e.preventDefault();
    e.stopPropagation(); // Prevent event from bubbling
    onDragEnd(); // Reset dragging state

    const eventIdStr = e.dataTransfer.getData("eventId");
    const eventName = e.dataTransfer.getData("eventName");
    const eventDescription = e.dataTransfer.getData("eventDescription");
    const sourceTimeIndexStr = e.dataTransfer.getData("sourceTimeIndex");

    if (!eventIdStr || !eventName) {
      return;
    }

    const eventId = parseInt(eventIdStr);
    const sourceTimeIndex = sourceTimeIndexStr
      ? parseInt(sourceTimeIndexStr)
      : -1;

    // Don't do anything if dropping on itself
    if (targetEventId && targetEventId === eventId) {
      return;
    }

    // Create a copy of localDays
    const updatedDays = JSON.parse(JSON.stringify(localDays)) as DayItinerary[];
    const currentDay = updatedDays[selectedDayIndex];
    let unassigned_events = JSON.parse(JSON.stringify(unassigned)) as Event[];

    // Find the full event object from the source
    let draggedEvent: Event | undefined =
      sourceTimeIndex >= 0
        ? localDays[selectedDayIndex].timeBlocks[sourceTimeIndex].events.find(
            (e) => e.id === eventId
          )
        : unassigned.find((e) => e.id === eventId);

    if (!draggedEvent) {
      // Fallback if we can't find the full event
      draggedEvent = {
        ...EVENT_DEFAULT,
        id: eventId,
        event_name: eventName,
        event_description: eventDescription,
        street_address: "",
        postal_code: 0,
        city: "",
        country: "",
        event_type: ""
      };
    }

    //  checks to see if being dropped in a time block, and the event has a hard start
    if (targetTimeIndex >= 0 && draggedEvent.hard_start) {
      const currentDay = localDays[selectedDayIndex];
      const targetTimeBlock = currentDay.timeBlocks[targetTimeIndex].time;
      const targetDate = currentDay.date;

      if (
        !canDropEventInTimeBlock(
          draggedEvent,
          targetTimeBlock,
          targetDate,
          targetTimeIndex
        )
      ) {
        const requiredTimeBlock = getTimeBlockFromTimestamp(
          draggedEvent.hard_start
        );
        const requiredDate = getDateFromTimestamp(draggedEvent.hard_start);
        toast.error(
          `"${draggedEvent.event_name}" has a fixed start time and must be placed in the ${requiredTimeBlock} block on ${requiredDate}.`,
          5000
        );
        return;
      }
    }

    // Handle dropping on specific event (swap positions)
    if (targetEventId !== undefined) {
      // Check if in same container
      const targetInSameBlock = sourceTimeIndex === targetTimeIndex;

      if (targetInSameBlock) {
        // Swap positions within same container
        if (targetTimeIndex >= 0) {
          // In a time block
          const targetBlock = currentDay.timeBlocks[targetTimeIndex];
          const draggedIndex = targetBlock.events.findIndex(
            (e) => e.id === eventId
          );
          const targetIndex = targetBlock.events.findIndex(
            (e) => e.id === targetEventId
          );

          if (draggedIndex !== -1 && targetIndex !== -1) {
            // Swap the events
            [
              targetBlock.events[draggedIndex],
              targetBlock.events[targetIndex]
            ] = [
              targetBlock.events[targetIndex],
              targetBlock.events[draggedIndex]
            ];
          }
        } else {
          // In unassigned
          const draggedIndex = unassigned_events.findIndex(
            (e) => e.id === eventId
          );
          const targetIndex = unassigned_events.findIndex(
            (e) => e.id === targetEventId
          );

          if (draggedIndex !== -1 && targetIndex !== -1) {
            // Swap the events
            [unassigned_events[draggedIndex], unassigned_events[targetIndex]] =
              [unassigned_events[targetIndex], unassigned_events[draggedIndex]];
          }
        }
      } else {
        // Different containers - insert at target position
        // Remove from source
        if (sourceTimeIndex >= 0) {
          currentDay.timeBlocks[sourceTimeIndex].events = currentDay.timeBlocks[
            sourceTimeIndex
          ].events.filter((e) => e.id !== eventId);
        } else {
          unassigned_events = unassigned_events.filter((e) => e.id !== eventId);
        }

        // Add at target position
        if (targetTimeIndex >= 0) {
          const targetBlock = currentDay.timeBlocks[targetTimeIndex];
          const targetIndex = targetBlock.events.findIndex(
            (e) => e.id === targetEventId
          );
          if (targetIndex !== -1) {
            targetBlock.events.splice(targetIndex, 0, draggedEvent);
          } else {
            targetBlock.events.push(draggedEvent);
          }
        } else {
          const targetIndex = unassigned_events.findIndex(
            (e) => e.id === targetEventId
          );
          if (targetIndex !== -1) {
            unassigned_events.splice(targetIndex, 0, draggedEvent);
          } else {
            unassigned_events.push(draggedEvent);
          }
        }
      }
    } else {
      // Regular drop on container (not on specific event)
      // Remove event from source time block if it exists
      if (sourceTimeIndex >= 0) {
        currentDay.timeBlocks[sourceTimeIndex].events = currentDay.timeBlocks[
          sourceTimeIndex
        ].events.filter((e) => e.id !== eventId);
      } else {
        unassigned_events = unassigned_events.filter((e) => e.id !== eventId);
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
    }

    // Notify parent component of changes
    onUpdate(updatedDays);
    onUnassignedUpdate(unassigned_events);
  };

  const onDragOver = (e: React.DragEvent) => {
    if (editMode) {
      e.preventDefault();
    }
  };

  if (!localDays || localDays.length === 0) {
    return null;
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

  const closeCreateModal = () => setCreateModalOpen(false);
  
  const onSaveUserEvent = async (e: React.FormEvent<HTMLFormElement>) => {
  e.preventDefault();

  // Validate image size before proceeding
  if (userEventForm.photoName && userEventForm.photoName.startsWith('data:')) {
    const base64Length = userEventForm.photoName.length - (userEventForm.photoName.indexOf(',') + 1);
    const sizeInBytes = (base64Length * 3) / 4;
    if (sizeInBytes > 5 * 1024 * 1024) {
      toast.error('Image size must be less than 5MB');
      return;
    }
  }

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
        : TIMEZONES[userEventForm.timezoneIndex],
    photo_name: userEventForm.photoName || null,
  };
  const result = await apiUserEvent(userEvent);
  if (result.status === 401) {
    toast.error("Unauthorized user, please log in.");
    navigate("/login");
    return;
  }

  if (result.status == 404) {
    toast.error("User-event not found for this user.");
    return;
  }

  if (result.result === null || result.status !== 200) {
    toast.error("Failed to update user event, please try again.");
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
    timezoneIndex: -1,
    photoName: ""
  });
  setImagePreview(null); 
  
  const event: Event = {
    ...EVENT_DEFAULT,
    id: result.result.id,
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
        : TIMEZONES[userEventForm.timezoneIndex],
    user_created: true,
    photo_name: userEventForm.photoName || null, // Add this line
  };

  const updatedUnassigned = [...unassigned, event];
  onUnassignedUpdate(updatedUnassigned);
  setCreateModalOpen(false);
};

  const closeSearchModal = () => setSearchModalOpen(false);
  const onSearchSend = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
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
      toast.error("Unauthorized user, please log in.");
      navigate("/login");
      return;
    }

    if (result.result === null || result.status !== 200) {
      toast.error("Failed to find event, please try again.");
      setSearchResultCaption("Error Searching Events");
      setSearchResult([]);
      return;
    }

    const displayEvents = result.result.events.filter(
      (e) => !unassigned.some((v) => v.id === e.id)
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
    if (!unassigned.some((e) => e.id === event.id)) {
      // Create a new array instead of mutating the existing one
      const updatedUnassigned = [...unassigned, event];
      onUnassignedUpdate(updatedUnassigned);
    }
    setSearchResult(searchResult!.filter((e) => e.id !== event.id));
  };

  const deleteDay = (indexToDelete: number) => {
    const dayNumber = indexToDelete + 1;
    const dayDate = localDays[indexToDelete].date;

    if (
      !window.confirm(
        `Are you sure you want to delete Day ${dayNumber} (${dayDate})? All events from this day will be moved to unassigned events.`
      )
    ) {
      return;
    }

    // Create a new array without the day at indexToDelete
    const updatedDays = localDays.filter((_, index) => index !== indexToDelete);

    // Move all events from the deleted day to unassigned
    const updatedUnassigned = [
      ...unassigned,
      ...localDays[indexToDelete].timeBlocks.flatMap((b) => b.events)
    ];

    onUpdate(updatedDays);
    onUnassignedUpdate(updatedUnassigned);
  };


  // Add these helper functions for image handling
const handleImageChange = (e: React.ChangeEvent<HTMLInputElement>) => {
  const file = e.target.files?.[0];
  if (file) {
    if (!file.type.startsWith('image/')) {
      toast.error('Please select an image file');
      return;
    }
    
    if (file.size > 5 * 1024 * 1024) {
      toast.error('Image size must be less than 5MB');
      return;
    }

    const reader = new FileReader();
    reader.onloadend = () => {
      const base64String = reader.result as string;
      setImagePreview(base64String);
      setUserEventForm({
        ...userEventForm,
        photoName: base64String  
      });
    };
    reader.readAsDataURL(file);
  }
};

const handleImageDrop = (e: React.DragEvent<HTMLDivElement>) => {
  e.preventDefault();
  e.stopPropagation();
  
  const file = e.dataTransfer.files[0];
  if (file && file.type.startsWith('image/')) {
    if (file.size > 5 * 1024 * 1024) {
      toast.error('Image size must be less than 5MB');
      return;
    }

    const reader = new FileReader();
    reader.onloadend = () => {
      const base64String = reader.result as string;
      setImagePreview(base64String);
      setUserEventForm({
        ...userEventForm,
        photoName: base64String 
      });
    };
    reader.readAsDataURL(file);
  } else {
    toast.error('Please drop an image file');
  }
};

const handleImageDragOver = (e: React.DragEvent<HTMLDivElement>) => {
  e.preventDefault();
  e.stopPropagation();
};

const removeImage = () => {
  setImagePreview(null);
  setUserEventForm({
    ...userEventForm,
    photoName: ""  
  });
};

  return (
    <>
      {/* Scroll indicators when dragging - outside main container for proper fixed positioning */}
      {isDragging && (
        <>
          <div className="scroll-indicator scroll-indicator-top">
            <div className="scroll-indicator-content">
              <svg
                width="32"
                height="32"
                viewBox="0 0 24 24"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M12 19V5M12 5L5 12M12 5L19 12"
                  stroke="currentColor"
                  strokeWidth="2.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
              <span>Scroll Up</span>
            </div>
          </div>
          <div className="scroll-indicator scroll-indicator-bottom">
            <div className="scroll-indicator-content">
              <span>Scroll Down</span>
              <svg
                width="32"
                height="32"
                viewBox="0 0 24 24"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M12 5V19M12 19L19 12M12 19L5 12"
                  stroke="currentColor"
                  strokeWidth="2.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
            </div>
          </div>
        </>
      )}

      <div className={`itinerary-section ${compact ? "compact" : ""}`}>
        {/* Header Row */}
        <div className="itinerary-header">
          <h3>{title || "Itinerary"}</h3>
        </div>

        {/* Unassigned Events */}
        {editMode && (
          <div
            className="unassigned-events"
            key={`unassigned-${selectedDayIndex}`}
          >
            <div
              className={"time-block editable"}
              onDrop={(e) => onDrop(e, -1)}
              onDragOver={onDragOver}
            >
              <div className="events-area">
                {unassigned.length === 0 ? (
                  <p className="workspace-empty-text">No unassigned events</p>
                ) : (
                  <>
                    <p className="unassigned-events-label">Unassigned Events</p>
                    {unassigned.map((event) => (
                      <EventCard
                        key={event.id}
                        event={event}
                        variant="workspace"
                        unassignedEvents={unassigned}
                        localDays={localDays}
                        onDaysUpdate={onUpdate}
                        onUnassignedUpdate={onUnassignedUpdate}
                        draggable={editMode}
                        onDragStart={(e) => onDragStart(e, event, -1)}
                        onDragEnd={onDragEnd}
                        onDrop={(e, targetEventId) =>
                          onDrop(e, -1, targetEventId)
                        }
                        onDragOver={onDragOver}
                      />
                    ))}
                  </>
                )}
              </div>
            </div>
          </div>
        )}

        {/* Timeline Layout */}
        <div
          className="timeline-container"
          key={`timeline-${selectedDayIndex}`}
        >
          <div className="timeline-events">
            {(() => {
              return currentDay.timeBlocks.map((block, block_index) => {
                const blockEvents = block.events;

                if (blockEvents.length === 0) {
                  return (
                    <div key={block.time} className="time-block-empty-section">
                      <div className="time-block-header">
                        <h3 className="time-block-title">{block.time}</h3>
                        <span className="time-block-range">
                          {getTimeRange(block.time)}
                        </span>
                      </div>
                      <div
                        className={`time-block-glass-container ${block.time.toLowerCase()} ${editMode ? "droppable" : ""}`}
                        onDrop={(e) => editMode && onDrop(e, block_index)}
                        onDragOver={onDragOver}
                      >
                        <div className="time-block-empty">
                          <p>No {block.time.toLowerCase()} events</p>
                        </div>
                      </div>
                    </div>
                  );
                }

                return (
                  <div key={block.time} className="time-block-group">
                    <div className="time-block-header">
                      <h3 className="time-block-title">{block.time}</h3>
                      <span className="time-block-range">
                        {getTimeRange(block.time)}
                      </span>
                    </div>

                    <div
                      className={`time-block-events-wrapper ${block.time.toLowerCase()} ${editMode ? "droppable" : ""}`}
                      onDrop={(e) => editMode && onDrop(e, block_index)}
                      onDragOver={onDragOver}
                    >
                      {blockEvents.map((event, event_index) => {
                        const isRightSide = event_index % 2 === 0;

                        return (
                          <React.Fragment key={`${event.id}-${event_index}`}>
                            <div
                              className={`timeline-event-wrapper ${
                                isRightSide ? "right-aligned" : "left-aligned"
                              } ${editMode ? "editable" : ""}`}
                            >
                              <EventCard
                                time={block.time}
                                event={event}
                                unassignedEvents={unassigned}
                                localDays={localDays}
                                onDaysUpdate={onUpdate}
                                onUnassignedUpdate={onUnassignedUpdate}
                                draggable={editMode ?? false}
                                onDragStart={(e) =>
                                  onDragStart(e, event, block_index)
                                }
                                onDragEnd={onDragEnd}
                                onDrop={(e, targetEventId) =>
                                  onDrop(e, block_index, targetEventId)
                                }
                                onDragOver={onDragOver}
                                imageOnLeft={isRightSide}
                              />
                            </div>
                            {event_index !== blockEvents.length - 1 && (
                              <div
                                className={`timeline-arrow ${
                                  isRightSide
                                    ? "right-to-left"
                                    : "left-to-right"
                                }`}
                              >
                                <img
                                  src={
                                    isRightSide
                                      ? "/rightarrow.png"
                                      : "/left-arrow.png"
                                  }
                                  alt="timeline arrow"
                                  className="arrow-image"
                                />
                              </div>
                            )}
                          </React.Fragment>
                        );
                      })}
                    </div>
                  </div>
                );
              });
            })()}
          </div>
        </div>

        {/* Day Navigation - Bottom */}
        <div className="day-navigation-bottom">
          {localDays.map((day, index) => (
            <div
              key={day.date.toString()}
              className={`day-nav-item ${index === selectedDayIndex ? "active" : ""}`}
              onClick={() => setSelectedDayIndex(index)}
            >
              Day {index + 1} ({day.date.toString()})
              {editMode && localDays.length > 1 && (
                <button
                  className="day-delete"
                  onClick={(e) => {
                    e.stopPropagation();

                    deleteDay(index);

                    // Update selected day index after deletion
                    if (index < selectedDayIndex) {
                      // Deleted a day before the currently selected day
                      setSelectedDayIndex(selectedDayIndex - 1);
                    } else if (index === selectedDayIndex) {
                      // Deleted the currently selected day
                      // If it was the last day, select the new last day
                      // Otherwise, keep the same index (next day shifts into this position)
                      if (selectedDayIndex === localDays.length - 1) {
                        setSelectedDayIndex(selectedDayIndex - 1);
                      }
                      // else: keep selectedDayIndex the same
                    }
                    // If index > selectedDayIndex, no change needed
                  }}
                >
                  <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    xmlns="http://www.w3.org/2000/svg"
                  >
                    <path
                      d="M3 6H21"
                      stroke="#dc2626"
                      strokeWidth="2"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                    />
                    <path
                      d="M19 6V20C19 21 18 22 17 22H7C6 22 5 21 5 20V6"
                      stroke="#dc2626"
                      strokeWidth="2"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                    />
                    <path
                      d="M8 6V4C8 3 9 2 10 2H14C15 2 16 3 16 4V6"
                      stroke="#dc2626"
                      strokeWidth="2"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                    />
                    <path
                      d="M10 11V17"
                      stroke="#dc2626"
                      strokeWidth="2"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                    />
                    <path
                      d="M14 11V17"
                      stroke="#dc2626"
                      strokeWidth="2"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                    />
                  </svg>
                </button>
              )}
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
                <button
                  className="icon-button"
                  onClick={closeCreateModal}
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

              <form
                id="user-event-form"
                className="user-event-form"
                onSubmit={onSaveUserEvent}
              >
                <label>
                  Name
                  <input
                    onChange={(e) =>
                      setUserEventForm({
                        ...userEventForm,
                        name: e.target.value
                      })
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
                      setUserEventForm({
                        ...userEventForm,
                        type: e.target.value
                      })
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

                <div style={{ marginTop: '16px' }}>
  <label style={{ display: 'block', marginBottom: '8px', fontWeight: '500' }}>
    Custom Image (Optional)
  </label>
  
  {!imagePreview ? (
    <div
      onDrop={handleImageDrop}
      onDragOver={handleImageDragOver}
      style={{
        border: '2px dashed #ccc',
        borderRadius: '8px',
        padding: '32px',
        textAlign: 'center',
        cursor: 'pointer',
        transition: 'all 0.2s ease',
        backgroundColor: '#f9fafb'
      }}
      onClick={() => document.getElementById('image-upload-input')?.click()}
    >
      <svg
        width="48"
        height="48"
        viewBox="0 0 24 24"
        fill="none"
        stroke="#9ca3af"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
        style={{ margin: '0 auto 12px' }}
      >
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
        <circle cx="8.5" cy="8.5" r="1.5" />
        <polyline points="21 15 16 10 5 21" />
      </svg>
      <p style={{ color: '#6b7280', marginBottom: '8px' }}>
        Drop an image here or click to select
      </p>
      <p style={{ color: '#9ca3af', fontSize: '0.875rem' }}>
        PNG, JPG, GIF up to 5MB
      </p>
      <input
        id="image-upload-input"
        type="file"
        accept="image/*"
        onChange={handleImageChange}
        style={{ display: 'none' }}
      />
    </div>
  ) : (
    <div style={{ position: 'relative', borderRadius: '8px', overflow: 'hidden' }}>
      <img
        src={imagePreview}
        alt="Preview"
        style={{
          width: '100%',
          maxHeight: '300px',
          objectFit: 'cover',
          borderRadius: '8px'
        }}
      />
      <button
        type="button"
        onClick={removeImage}
        style={{
          position: 'absolute',
          top: '8px',
          right: '8px',
          backgroundColor: 'rgba(0, 0, 0, 0.6)',
          color: 'white',
          border: 'none',
          borderRadius: '50%',
          width: '32px',
          height: '32px',
          cursor: 'pointer',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: '18px',
          fontWeight: 'bold'
        }}
      >
        ×
      </button>
    </div>
  )}
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
                  Create Event
                </button>
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
                        // onClick={onSearchSend}
                        form="search-event-form"
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

                  <form
                    id="search-event-form"
                    className="user-event-form"
                    onSubmit={onSearchSend}
                  >
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
                            setSearchEventForm({
                              ...searchEventForm,
                              endsAfter
                            });
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
    </>
  );
};

export default Itinerary;
