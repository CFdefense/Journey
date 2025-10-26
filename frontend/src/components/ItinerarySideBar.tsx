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
  
  const handleSaveItinerary = () => {
    if (selectedItineraryId !== null) {
      console.log("Saving itinerary with ID:", selectedItineraryId);
    } else {
      console.log("No itinerary selected to save");
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
          
          <button 
            className="save-itinerary-btn"
            onClick={handleSaveItinerary}
            disabled={selectedItineraryId === null}
          >
            Save Itinerary
          </button>
        </div>
      )}
    </div>
  );
}