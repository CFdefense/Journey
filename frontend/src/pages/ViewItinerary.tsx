import { useEffect, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import Itinerary from "../components/Itinerary";
import ViewPageSidebar from "../components/ViewPageSidebar";
import { convertToApiFormat, fetchItinerary } from "../helpers/itinerary";
import type { DayItinerary } from "../models/itinerary";
import { apiItineraryDetails, apiSaveItineraryChanges } from "../api/itinerary";
import Navbar from "../components/Navbar";
import "../styles/Itinerary.css";

function ViewItineraryPage() {
  const location = useLocation();
  const navigate = useNavigate();
  const [days, setDays] = useState<DayItinerary[]>([]);
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

  const handleItineraryUpdate = (updatedDays: DayItinerary[]) => {
    setDays(updatedDays);
  };

  const handleSave = async (updatedDays: DayItinerary[]) => {
    try {
      const apiPayload = convertToApiFormat(
        updatedDays,
        itineraryMetadata.id,
        itineraryMetadata.startDate,
        itineraryMetadata.endDate,
        itineraryMetadata.title,
        itineraryMetadata.chatSessionId
      );

      const result = await apiSaveItineraryChanges(apiPayload);
      console.log("Save result:", result);
      alert("Itinerary saved successfully!");
    } catch (error) {
      console.error("Failed to save itinerary:", error);
      alert("Failed to save changes. Please try again.");
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
    setDays([...days, newDay]);
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
      <Navbar page="view" />
      <ViewPageSidebar
        onCreateEvent={() => setCreateModalOpen(true)}
        onSearchEvents={() => setSearchModalOpen(true)}
        onAddDay={handleAddDay}
        onEditWithAI={handleEditWithAI}
      />
      <div className="view-content with-sidebar">
        <Itinerary
          days={days}
          onUpdate={handleItineraryUpdate}
          onSave={handleSave}
          onEditWithAI={handleEditWithAI}
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
