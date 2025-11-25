import { useEffect, useState, useRef } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import Itinerary from "../components/Itinerary";
import ViewPageSidebar from "../components/ViewPageSidebar";
import { convertToApiFormat, fetchItinerary, getUnassignedEvents } from "../helpers/itinerary";
import type { DayItinerary, Event } from "../models/itinerary";
import { apiItineraryDetails, apiSaveItineraryChanges } from "../api/itinerary";
import "../styles/Itinerary.css";

function ViewItineraryPage() {
  const location = useLocation();
  const navigate = useNavigate();
  const [days, setDays] = useState<DayItinerary[]>([]);
  const [unassignedEvents, setUnassignedEvents] = useState<Event[]>([]);
  const [createModalOpen, setCreateModalOpen] = useState(false);
  const [searchModalOpen, setSearchModalOpen] = useState(false);

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
  const daysRef = useRef(days);
  const unassignedEventsRef = useRef(unassignedEvents);
  const saveTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Keep refs in sync with state
  useEffect(() => {
    daysRef.current = days;
  }, [days]);

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
    setDays(updatedDays);
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

  const autoSave = async (updatedDays: DayItinerary[], updatedUnassigned?: Event[]) => {
    try {
      const unassignedToUse = updatedUnassigned !== undefined ? updatedUnassigned : unassignedEvents;

      // Calculate start_date and end_date from the days array
      const startDate = updatedDays.length > 0 ? updatedDays[0].date : itineraryMetadata.startDate;
      const endDate = updatedDays.length > 0 ? updatedDays[updatedDays.length - 1].date : itineraryMetadata.endDate;

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
    const last = new Date(days[days.length - 1].date);
    last.setDate(last.getDate() + 1);
    const newDay: DayItinerary = {
      date: last.toISOString().slice(0, 10),
      timeBlocks: []
    };
    const updatedDays = [...days, newDay];
    setDays(updatedDays);
    // Update ref immediately before debounced save
    daysRef.current = updatedDays;
    debouncedAutoSave();
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
          setDays(data);

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
    <div className="view-page view-page--gradient with-sidebar">
      <ViewPageSidebar
        onCreateEvent={() => setCreateModalOpen(true)}
        onSearchEvents={() => setSearchModalOpen(true)}
        onAddDay={handleAddDay}
        onEditWithAI={handleEditWithAI}
      />
      <div className="view-content with-sidebar">
        <Itinerary
          days={days}
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
    </div>
  );
}
export default ViewItineraryPage;
