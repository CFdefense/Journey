import { useEffect, useState } from "react";
import Itinerary from "../components/Itinerary";
import { convertToApiFormat, fetchItinerary } from "../helpers/itinerary";
import type { DayItinerary } from "../helpers/itinerary";
import { apiItineraryDetails, saveItineraryChanges } from "../api/itinerary";
import Navbar from "../components/Navbar";
import "../styles/Itinerary.css";
import { apiCurrent } from "../api/account";

export default function ViewItineraryPage() {
  const [days, setDays] = useState<DayItinerary[]>([]);
  const [editMode, setEditMode] = useState(false);
  const [firstName, setFirstName] = useState<string>("");
  
  // Store metadata needed for saving
  const [itineraryMetadata, setItineraryMetadata] = useState({
    id: 6,
    startDate: "",
    endDate: "",
    title: "",
    chatSessionId: null as number | null,
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
      
      const result = await saveItineraryChanges(apiPayload);
      console.log("Save successful! Itinerary ID:", result.id);
      alert("Itinerary saved successfully!");
    } catch (error) {
      console.error("Failed to save itinerary:", error);
      alert("Failed to save changes. Please try again.");
    }
  };

  useEffect(() => {
    async function load() {
      const itineraryId = 6; // <--itinerary ID for itinerary that is being fetched
      
      // Fetch the full API response to get metadata
      const apiResponse = await apiItineraryDetails(itineraryId);
      
      if (apiResponse.result) {
        // Store metadata
        setItineraryMetadata({
          id: apiResponse.result.id,
          startDate: apiResponse.result.start_date,
          endDate: apiResponse.result.end_date,
          title: apiResponse.result.title,
          chatSessionId: apiResponse.result.chat_session_id,
        });
        
        // Transform and store days
        const data = await fetchItinerary(itineraryId);
        setDays(data);
      }
    }
    load();
  }, []);

  return (
    <div className="view-page">
      <Navbar page="view" firstName={firstName} />
      <Itinerary 
        days={days} 
        onUpdate={handleItineraryUpdate}
        onSave={handleSave}
        editMode={editMode}
        onEditModeChange={setEditMode}
        title={itineraryMetadata.title}
      />

      {!editMode && <button className="edit-ai-button">Edit with AI</button>}

      {editMode && (
        <div className="edit-actions">
          <button 
            className="save-button"
            onClick={async () => {
              await handleSave(days);
              setEditMode(false);
            }}
          >
            Save Changes
          </button>
          <button 
            className="cancel-button"
            onClick={() => {
              setEditMode(false);
              async function reload() {
                const itineraryId = 6;
                const data = await fetchItinerary(itineraryId);
                setDays(data);
              }
              reload();
            }}
          >
            Cancel
          </button>
        </div>
      )}
  </div>
  );
}