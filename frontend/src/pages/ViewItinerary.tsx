import { useEffect, useState, useRef } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import Itinerary from "../components/Itinerary";
import ViewPageSidebar from "../components/ViewPageSidebar";
import {
  convertToApiFormat,
  fetchItinerary,
  getUnassignedEvents
} from "../helpers/itinerary";
import type { DayItinerary, Event } from "../models/itinerary";
import { apiItineraryDetails, apiSaveItineraryChanges } from "../api/itinerary";
import "../styles/Itinerary.css";

function ViewItineraryPage() {
  const location = useLocation();
  const navigate = useNavigate();
  const [createModalOpen, setCreateModalOpen] = useState(false);
  const [searchModalOpen, setSearchModalOpen] = useState(false);
  const [addDayModalOpen, setAddDayModalOpen] = useState(false);
  const [selectedDate, setSelectedDate] = useState("");
  const [localDays, setLocalDays] = useState<DayItinerary[]>([]);
  const [unassignedEvents, setUnassignedEvents] = useState<Event[]>([]);

  // Get itinerary ID from navigation state
  const itineraryId = location.state?.itineraryId;

  // Store metadata needed for saving
  const [itineraryMetadata, setItineraryMetadata] = useState({
    id: 0,
    startDate: "",
    endDate: "",
    title: "",
    chatSessionId: null as number | null
  });

  // Use refs to track the latest values for autosave
  const daysRef = useRef(localDays);
  const unassignedEventsRef = useRef(unassignedEvents);
  const saveTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Keep refs in sync with state
  useEffect(() => {
    daysRef.current = localDays;
  }, [localDays]);

  useEffect(() => {
    unassignedEventsRef.current = unassignedEvents;
  }, [unassignedEvents]);

  const debouncedAutoSave = () => {
    // Clear any pending save
    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current);
    }

    // Schedule a new save with the latest values from refs
    saveTimeoutRef.current = setTimeout(() => {
      autoSave(daysRef.current, unassignedEventsRef.current);
    }, 500); // Wait 500ms after last update
  };

  const handleItineraryUpdate = (updatedDays: DayItinerary[]) => {
    const toReassign = [];
    // Pull events that are in the wrong spot
    for (const day of updatedDays) {
      for (const timeblock of day.timeBlocks) {
        if (timeblock.events.length === 0) {
          continue;
        }
        for (let i = 0; i < timeblock.events.length; i++) {
          const curr = timeblock.events[i];
          if (!curr.hard_start) {
            continue;
          }
          if (curr.hard_start.slice(0, 10) !== day.date) {
            timeblock.events.splice(i, 1);
            toReassign.push(curr);
            i--;
            continue;
          }

          const time = new Date(curr.hard_start).getTime();

          const fourAM = new Date(curr.hard_start!);
          const noon = new Date(curr.hard_start!);
          const sixPM = new Date(curr.hard_start!);
          fourAM.setHours(4, 0, 0, 0);
          noon.setHours(12, 0, 0, 0);
          sixPM.setHours(18, 0, 0, 0);
          const _4am = fourAM.getTime();
          const _12pm = noon.getTime();
          const _6pm = sixPM.getTime();

          // I'm too lazy to convert this into a proper conditional
          if (timeblock.time === "Morning" && time >= _4am && time < _12pm) {
            continue;
          }
          if (timeblock.time === "Afternoon" && time >= _12pm && time < _6pm) {
            continue;
          }
          if (timeblock.time === "Morning" && time >= _6pm && time < _4am) {
            continue;
          }

          timeblock.events.splice(i, 1);
          toReassign.push(curr);
          i--;
        }
      }
    }
    // reinsert pulled events
    for (let i = 0; i < toReassign.length; i++) {
      const pulled = toReassign[i];
      for (const day of updatedDays) {
        if (day.date !== pulled.hard_start!.slice(0, 10)) {
          continue;
        }
        toReassign.splice(i, 1);
        i--;

        const time = new Date(pulled.hard_start!).getTime();

        const fourAM = new Date(pulled.hard_start!);
        const noon = new Date(pulled.hard_start!);
        const sixPM = new Date(pulled.hard_start!);
        fourAM.setHours(4, 0, 0, 0);
        noon.setHours(12, 0, 0, 0);
        sixPM.setHours(18, 0, 0, 0);
        const _4am = fourAM.getTime();
        const _12pm = noon.getTime();
        const _6pm = sixPM.getTime();

        if (time >= _4am && time < _12pm) {
          day.timeBlocks[0].events.push(pulled);
        } else if (time >= _12pm && time < _6pm) {
          day.timeBlocks[1].events.push(pulled);
        } else if (time >= _6pm && time < _4am) {
          day.timeBlocks[2].events.push(pulled);
        } else {
          console.error("Unreachable statement");
        }
      }
    }
    // put non-reassigned in unassigned
    if (toReassign.length > 0) {
      setUnassignedEvents([...unassignedEvents, ...toReassign]);
    }
    // sort events
    for (const day of updatedDays) {
      for (const timeblock of day.timeBlocks) {
        if (timeblock.events.length === 0) {
          continue;
        }

        const sortable = timeblock.events
          .map((ev, index) => ({ ev, index }))
          .filter((item) => item.ev.hard_start !== null);

        // Sort those by datetime
        sortable.sort(
          (a, b) =>
            new Date(a.ev.hard_start!).getTime() -
            new Date(b.ev.hard_start!).getTime()
        );

        // Create a result array
        const result = [...timeblock.events];

        // Place sorted events back into their original non-null slots (in order)
        let si = 0;
        for (let i = 0; i < result.length; i++) {
          if (result[i].hard_start === null) {
            result[i].block_index = i;
            continue;
          }
          result[i] = sortable[si].ev;
          result[i].block_index = i;
          si++;
        }

        timeblock.events = result;
      }
    }

    setLocalDays(updatedDays);
    // Update ref immediately before debounced save
    daysRef.current = updatedDays;
    debouncedAutoSave();
  };

  const handleUnassignedUpdate = (updatedUnassigned: Event[]) => {
    setUnassignedEvents(updatedUnassigned);
    // Update ref immediately before debounced save
    unassignedEventsRef.current = updatedUnassigned;
    debouncedAutoSave();
  };

  const autoSave = async (
    updatedDays: DayItinerary[],
    updatedUnassigned?: Event[]
  ) => {
    try {
      const unassignedToUse =
        updatedUnassigned !== undefined ? updatedUnassigned : unassignedEvents;

      // Calculate start_date and end_date from the days array
      const startDate =
        updatedDays.length > 0
          ? updatedDays[0].date
          : itineraryMetadata.startDate;
      const endDate =
        updatedDays.length > 0
          ? updatedDays[updatedDays.length - 1].date
          : itineraryMetadata.endDate;

      const apiPayload = convertToApiFormat(
        updatedDays,
        itineraryMetadata.id,
        startDate,
        endDate,
        itineraryMetadata.title,
        itineraryMetadata.chatSessionId,
        unassignedToUse
      );

      await apiSaveItineraryChanges(apiPayload);
    } catch (error) {
      console.error("Auto-save failed:", error);
      // Silent fail - don't interrupt user with alerts
    }
  };

  const handleEditWithAI = () => {
    console.log("Edit with AI clicked", {
      itineraryId: itineraryMetadata.id,
      chatSessionId: itineraryMetadata.chatSessionId
    });
    // Navigate to home with both the itinerary ID and chat session ID
    navigate("/home", {
      state: {
        selectedItineraryId: itineraryMetadata.id,
        chatSessionId: itineraryMetadata.chatSessionId,
        openItinerarySidebar: true
      }
    });
  };

  const handleAddDay = () => {
    // Calculate the next day as default
    let defaultDate: string;
    if (localDays.length > 0) {
      const last = new Date(localDays[localDays.length - 1].date);
      last.setDate(last.getDate() + 1);
      defaultDate = last.toISOString().slice(0, 10);
    } else {
      // If no days exist, default to today
      defaultDate = new Date().toISOString().slice(0, 10);
    }
    setSelectedDate(defaultDate);
    setAddDayModalOpen(true);
  };

  const confirmAddDay = () => {
    if (!selectedDate) {
      alert("Please select a date");
      return;
    }

    // Check if date already exists
    if (localDays.some((day) => day.date === selectedDate)) {
      alert("A day with this date already exists in your itinerary");
      return;
    }

    const newDay: DayItinerary = {
      date: selectedDate,
      timeBlocks: [
        {
          time: "Morning",
          events: []
        },
        {
          time: "Afternoon",
          events: []
        },
        {
          time: "Evening",
          events: []
        }
      ]
    };

    // Insert the day in chronological order
    const updatedDays = [...localDays, newDay].sort(
      (a, b) => new Date(a.date).getTime() - new Date(b.date).getTime()
    );

    setLocalDays(updatedDays);
    // Update ref immediately before debounced save
    daysRef.current = updatedDays;
    debouncedAutoSave();
    setAddDayModalOpen(false);
  };

  useEffect(() => {
    // Redirect back to home if no itinerary ID is provided
    if (!itineraryId) {
      console.error("No itinerary ID provided");
      navigate("/");
      return;
    }

    async function load() {
      try {
        // Fetch the full API response to get metadata
        const apiResponse = await apiItineraryDetails(itineraryId);

        if (apiResponse.result) {
          // Store metadata
          setItineraryMetadata({
            id: apiResponse.result.id,
            startDate: apiResponse.result.start_date,
            endDate: apiResponse.result.end_date,
            title: apiResponse.result.title,
            chatSessionId: apiResponse.result.chat_session_id
          });

          // Transform and store days
          const data = await fetchItinerary(itineraryId);
          setLocalDays(data);

          // Load unassigned events
          const unassigned = getUnassignedEvents(apiResponse.result);
          setUnassignedEvents(unassigned);
        }
      } catch (error) {
        console.error("Failed to load itinerary:", error);
        alert("Failed to load itinerary. Redirecting to home.");
        navigate("/");
      }
    }

    load();
  }, [itineraryId, navigate]);

  return (
    <div
      id="view-itinerary-page"
      className="view-page view-page--gradient with-sidebar"
    >
      <ViewPageSidebar
        onCreateEvent={() => setCreateModalOpen(true)}
        onSearchEvents={() => setSearchModalOpen(true)}
        onAddDay={handleAddDay}
        onEditWithAI={handleEditWithAI}
      />
      <div className="view-content with-sidebar">
        <Itinerary
          localDays={localDays}
          unassigned={unassignedEvents}
          onUpdate={handleItineraryUpdate}
          onUnassignedUpdate={handleUnassignedUpdate}
          title={itineraryMetadata.title}
          editMode={true}
          externalCreateModal={createModalOpen}
          externalSearchModal={searchModalOpen}
          onCreateModalChange={setCreateModalOpen}
          onSearchModalChange={setSearchModalOpen}
        />
      </div>

      {/* Add Day Modal */}
      {addDayModalOpen && (
        <div
          className="user-event-modal-overlay"
          onClick={() => setAddDayModalOpen(false)}
        >
          <div
            className="user-event-modal"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-header">
              <h2>Add a Day</h2>
              <button
                className="icon-button"
                onClick={() => setAddDayModalOpen(false)}
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
              className="user-event-form"
              onSubmit={(e) => {
                e.preventDefault();
                confirmAddDay();
              }}
            >
              <label>
                Date
                <input
                  type="date"
                  value={selectedDate}
                  onChange={(e) => setSelectedDate(e.target.value)}
                  required
                />
              </label>

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
                Add Day
              </button>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
export default ViewItineraryPage;
