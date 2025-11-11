import { useNavigate } from "react-router-dom";
import "../styles/ItinerarySideBar.css";
import Itinerary from "./Itinerary";
import type { DayItinerary } from "../helpers/itinerary";

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

  const handleSaveItinerary = () => {
    if (selectedItineraryId !== null) {
      console.log("Saving itinerary with ID:", selectedItineraryId);
    } else {
      console.log("No itinerary selected to save");
    }
  };

  // TODO this will need to change but will be based on how ViewItinerary is set up
  const handleViewItinerary = () => {
    if (selectedItineraryId !== null) {
      navigate("/view", { state: { itineraryId: selectedItineraryId } });
    }
  };

  return (
    <div className={`itinerary-sidebar ${sidebarVisible ? "open" : "closed"}`}>
      <div className="itinerary-sidebar-top">
        <div className="itinerary-sidebar-title">Itinerary</div>
        <button
          className="itinerary-sidebar-toggle-btn"
          onClick={onToggleSidebar}
          aria-label="Toggle itinerary sidebar"
        >
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M3 6H21" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
            <path d="M3 12H21" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
            <path d="M3 18H21" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
        </button>
      </div>

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
            disabled={selectedItineraryId === null}
          >
            Save
          </button>
        </div>
      </div>
    </div>
  );
}
