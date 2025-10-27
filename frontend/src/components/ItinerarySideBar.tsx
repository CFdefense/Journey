import { useNavigate } from "react-router-dom";
import "../styles/ItinerarySideBar.css";
import Itinerary from "./Itinerary";
import type { DayItinerary } from "../helpers/itinerary";

interface ItinerarySideBarProps {
  onToggleSidebar: () => void;
  sidebarVisible: boolean;
  itineraryData: DayItinerary[] | null;
  selectedItineraryId: number | null;
}

export default function ItinerarySideBar({
  onToggleSidebar,
  sidebarVisible,
  itineraryData,
  selectedItineraryId
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
      navigate('/view', { state: { itineraryId: selectedItineraryId } });
    }
  };

  return (
    <div className={`itinerary-sidebar ${sidebarVisible ? "open" : "closed"}`}>
      <div className="itinerary-sidebar-top">
        {sidebarVisible && <div className="itinerary-sidebar-title">Itinerary</div>}
        <button className="itinerary-sidebar-toggle-btn" onClick={onToggleSidebar}>
          ⋮
        </button>
      </div>

      {sidebarVisible && (
        <div className="itinerary-content">
          <Itinerary 
            key={itineraryData ? JSON.stringify(itineraryData[0]?.date) : 'no-itinerary'}
            days={itineraryData ?? undefined} 
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
      )}
    </div>
  );
}