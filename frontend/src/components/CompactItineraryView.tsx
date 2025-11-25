import React, { useState } from "react";
import type { DayItinerary, Event } from "../models/itinerary";
import "../styles/CompactItineraryView.css";

interface CompactItineraryViewProps {
  days?: DayItinerary[];
  title?: string;
}

const CompactItineraryView: React.FC<CompactItineraryViewProps> = ({
  days,
  title
}) => {
  const [selectedDayIndex, setSelectedDayIndex] = useState(0);

  if (!days || days.length === 0) {
    return (
      <div className="compact-itinerary-empty">
        <p>No itinerary to display</p>
      </div>
    );
  }

  const currentDay = days[selectedDayIndex];

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

  const formatEventTime = (event: Event, timeBlockName: string): string => {
    if (event.hard_start) {
      try {
        const date = new Date(event.hard_start);
        if (!isNaN(date.getTime())) {
          const hours = date.getHours();
          const minutes = date.getMinutes();
          const ampm = hours >= 12 ? "PM" : "AM";
          const displayHours = hours % 12 || 12;
          const displayMinutes = minutes.toString().padStart(2, "0");
          return `${displayHours}:${displayMinutes} ${ampm}`;
        }
      } catch (e) {
        // Fall through to time range
      }
    }
    // Return the time segment range as fallback
    return getTimeRange(timeBlockName);
  };

  const formatAddress = (event: Event): string => {
    const parts: string[] = [];
    if (event.street_address) parts.push(event.street_address);
    if (event.city) parts.push(event.city);
    if (event.country) parts.push(event.country);
    if (event.postal_code) parts.push(event.postal_code.toString());
    return parts.length > 0 ? parts.join(", ") : "";
  };

  // Group events by time block
  const getEventsByTimeBlock = (): { [key: string]: Event[] } => {
    const grouped: { [key: string]: Event[] } = {
      Morning: [],
      Afternoon: [],
      Evening: []
    };

    currentDay.timeBlocks.forEach((block) => {
      if (grouped[block.time]) {
        grouped[block.time] = block.events;
      }
    });

    return grouped;
  };

  const eventsByTimeBlock = getEventsByTimeBlock();

  // Format date for calendar display
  const formatDayOfWeek = (dateString: string): string => {
    const date = new Date(dateString);
    return date.toLocaleDateString("en-US", { weekday: "short" }).toUpperCase();
  };

  const formatDayOfMonth = (dateString: string): string => {
    const date = new Date(dateString);
    const day = date.getDate();
    return `${day}`;
  };

  return (
    <div className="compact-itinerary-view">
      {/* Title */}
      {title && (
        <div className="compact-itinerary-title">
          <h2>{title}</h2>
        </div>
      )}

      {/* Day Navigation */}
      <div className="compact-day-navigation">
        {days.map((day, index) => (
          <button
            key={day.date.toString()}
            className={`compact-day-button ${index === selectedDayIndex ? "active" : ""}`}
            onClick={() => setSelectedDayIndex(index)}
          >
            <span className="compact-day-weekday">
              {formatDayOfWeek(day.date.toString())}
            </span>
            <span className="compact-day-number">
              {formatDayOfMonth(day.date.toString())}
            </span>
          </button>
        ))}
      </div>

      {/* Time Segments */}
      <div className="compact-time-segments">
        {["Morning", "Afternoon", "Evening"].map((timeBlockName) => {
          const events = eventsByTimeBlock[timeBlockName] || [];
          
          return (
            <div key={timeBlockName} className="compact-time-segment">
              <div className="compact-segment-header">
                <h3 className="compact-segment-title">{timeBlockName}</h3>
                <span className="compact-segment-range">
                  {getTimeRange(timeBlockName)}
                </span>
              </div>

              <div className={`compact-events-list ${timeBlockName.toLowerCase()}`}>
                {events.length === 0 ? (
                  <p className="compact-no-events">
                    No {timeBlockName.toLowerCase()} events
                  </p>
                ) : (
                  events.map((event) => {
                    const eventTime = formatEventTime(event, timeBlockName);
                    const address = formatAddress(event);

                    return (
                      <div key={event.id} className="compact-event-card">
                        <div className="compact-event-header">
                          <h4 className="compact-event-name">
                            {event.event_name}
                          </h4>
                          <span className="compact-event-time">{eventTime}</span>
                        </div>

                        {event.event_description && (
                          <p className="compact-event-description">
                            {event.event_description}
                          </p>
                        )}

                        {address && (
                          <p className="compact-event-location">
                            {address}
                          </p>
                        )}

                        {event.event_type && (
                          <span className="compact-event-type">
                            {event.event_type}
                          </span>
                        )}
                      </div>
                    );
                  })
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default CompactItineraryView;

