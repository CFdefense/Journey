import { useNavigate } from "react-router-dom";
import "../styles/ItinerarySideBar.css";
import Itinerary from "./Itinerary";
import type { DayItinerary } from "../models/itinerary";
import { apiItineraryDetails, apiSaveItineraryChanges } from "../api/itinerary";
import { convertToApiFormat } from "../helpers/itinerary";

interface ItinerarySideBarProps {
  onToggleSidebar: () => void;
  sidebarVisible: boolean;
  itineraryData: DayItinerary[] | null;
  selectedItineraryId: number | null;
  itineraryTitle?: string;
}

export default function ItinerarySideBar({
  onToggleSidebar,
  sidebarVisible,
  itineraryData,
  selectedItineraryId,
  itineraryTitle
}: ItinerarySideBarProps) {
  const navigate = useNavigate();

  const handleSaveItinerary = async () => {
    if (selectedItineraryId === null || !itineraryData) {
      console.log("No itinerary selected to save");
      return;
    }

    try {
      // Fetch the full itinerary metadata
      const apiResponse = await apiItineraryDetails(selectedItineraryId);

      if (!apiResponse.result || apiResponse.status !== 200) {
        console.error("Failed to fetch itinerary details");
        return;
      }

      const itinerary = apiResponse.result;

      // set to the form needed to call the save itinerary api
      const payload = convertToApiFormat(
        itineraryData,
        itinerary.id,
        itinerary.start_date,
        itinerary.end_date,
        itinerary.title,
        itinerary.chat_session_id
      );

      // Save the itinerary
      await apiSaveItineraryChanges(payload);
    } catch (error) {
      console.error("Failed to save itinerary:", error);
    }
  };

  const handleViewItinerary = () => {
    if (selectedItineraryId !== null) {
      navigate("/view", { state: { itineraryId: selectedItineraryId } });
    }
  };

  return (
    <div className={`itinerary-sidebar ${sidebarVisible ? "open" : "closed"}`}>
      <div className="itinerary-sidebar-top">
        {sidebarVisible && (
          <div className="itinerary-sidebar-title">Itinerary</div>
        )}
        <button
          className="itinerary-sidebar-toggle-btn"
          onClick={onToggleSidebar}
        >
          ⋮
        </button>
      </div>

      {sidebarVisible && (
        <div className="itinerary-content">
          <Itinerary
            key={
              itineraryData
                ? JSON.stringify(itineraryData[0]?.date)
                : "no-itinerary"
            }
            days={itineraryData ?? undefined}
            compact={true}
            title={itineraryTitle}
            editMode={false}
          />

          <div className="itinerary-actions">
            <button
              className="edit-itinerary-btn"
              onClick={handleViewItinerary}
              disabled={selectedItineraryId === null}
            >
              Edit
            </button>
            <button
              className="save-itinerary-btn"
              onClick={handleSaveItinerary}
              disabled={selectedItineraryId === null || !itineraryData}
            >
              Save
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
