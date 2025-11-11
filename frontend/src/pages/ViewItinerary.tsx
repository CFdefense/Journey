import { useEffect, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import Itinerary from "../components/Itinerary";
import { convertToApiFormat, fetchItinerary } from "../helpers/itinerary";
import type { DayItinerary } from "../helpers/itinerary";
import { apiItineraryDetails, apiSaveItineraryChanges } from "../api/itinerary";
import Navbar from "../components/Navbar";
import "../styles/Itinerary.css";
import { apiCurrent } from "../api/account";

export default function ViewItineraryPage() {
  const location = useLocation();
  const navigate = useNavigate();
  const [days, setDays] = useState<DayItinerary[]>([]);
  const [firstName, setFirstName] = useState<string>("");

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

  useEffect(() => {
    async function fetchAccount() {
      const currentResult = await apiCurrent();
      const account = currentResult.result;

      if (account && currentResult.status === 200) {
        setFirstName(account.first_name || "");
      }
    }

    fetchAccount();
  }, []);

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
    <div className="view-page">
      <Navbar page="view" firstName={firstName} />
      <Itinerary
        days={days}
        onUpdate={handleItineraryUpdate}
        onSave={handleSave}
        title={itineraryMetadata.title}
        editMode={true}
      />
      <button className="edit-ai-button">Edit with AI</button>
    </div>
  );
}
