import { useEffect, useState } from "react";
import Itinerary from "../components/Itinerary";
import UnassignedEvents from "../components/UnassignedEvents";
import type { Event } from "../components/UnassignedEvents";
import { convertToApiFormat, fetchItinerary } from "../helpers/itinerary";
import type { DayItinerary } from "../helpers/itinerary";
import { apiItineraryDetails, saveItineraryChanges } from "../api/itinerary";
import "../styles/Itinerary.css";

export default function ViewItineraryPage() {
  const [days, setDays] = useState<DayItinerary[]>([]);
  
  // Store metadata needed for saving
  const [itineraryMetadata, setItineraryMetadata] = useState({
    id: 6,
    startDate: "",
    endDate: "",
    title: "",
    chatSessionId: null as number | null,
  });

  const unassignedEvents: Event[] = [
    { id: "1", title: "Breakfast", desc: "Saxbys coffee and bagel" },
    { id: "2", title: "Meeting", desc: "Capping discussion" }
  ];

  const onDragStart = (e: React.DragEvent, event: Event) => {
    e.dataTransfer.setData("eventId", event.id);
    e.dataTransfer.setData("eventTitle", event.title);
    e.dataTransfer.setData("eventDesc", event.desc || "");
  };

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
      <UnassignedEvents events={unassignedEvents} onDragStart={onDragStart} />
      <Itinerary 
        days={days} 
        onUpdate={handleItineraryUpdate}
        onSave={handleSave}
      />
      <button>Edit with AI</button>
    </div>
  );
}
