import "../styles/ItinerarySideBar.css";
import Itinerary from "./Itinerary";

interface ItinerarySideBarProps {
  onToggleSidebar: () => void;
  sidebarVisible: boolean;
}

export default function ItinerarySideBar({
  onToggleSidebar,
  sidebarVisible
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
          <Itinerary />
        </div>
      )}
    </div>
  );
}