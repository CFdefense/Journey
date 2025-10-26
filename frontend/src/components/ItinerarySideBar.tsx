import "../styles/ItinerarySideBar.css";
import Itinerary from "./Itinerary";
import type { DayItinerary } from "../helpers/itinerary";

interface ItinerarySideBarProps {
  onToggleSidebar: () => void;
  sidebarVisible: boolean;
  itineraryData: DayItinerary[] | null;
}

export default function ItinerarySideBar({
  onToggleSidebar,
  sidebarVisible,
  itineraryData
}: ItinerarySideBarProps) {
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
        </div>
      )}
    </div>
  );
}