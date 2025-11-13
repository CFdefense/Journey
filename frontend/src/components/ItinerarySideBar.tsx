import { useNavigate } from "react-router-dom";
import { useState } from "react";
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
  const [showSaveModal, setShowSaveModal] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  const handleSaveItinerary = async () => {
    if (selectedItineraryId === null || !itineraryData) {
      console.log("No itinerary selected to save");
      return;
    }

    setIsSaving(true);
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
      setShowSaveModal(false);
    } catch (error) {
      console.error("Failed to save itinerary:", error);
    } finally {
      setIsSaving(false);
    }
  };

  const handleViewItinerary = () => {
    if (selectedItineraryId !== null) {
      navigate("/view", { state: { itineraryId: selectedItineraryId } });
    }
  };

 return (
    <>
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
              onClick={() => setShowSaveModal(true)}
              disabled={selectedItineraryId === null}
            >
              Save
            </button>
          </div>
        </div>
      </div>

      {/* Save Confirmation Modal */}
      {showSaveModal && (
        <div className="itinerary-modal-overlay" onClick={() => setShowSaveModal(false)}>
          <div className="itinerary-modal-content" onClick={(e) => e.stopPropagation()}>
            <h2>Save Itinerary?</h2>
            <p>This will save all changes to your itinerary.</p>
            <div className="itinerary-modal-actions">
              <button
                className="itinerary-btn-secondary"
                onClick={() => setShowSaveModal(false)}
                disabled={isSaving}
              >
                Cancel
              </button>
              <button
                className="itinerary-btn-primary"
                onClick={handleSaveItinerary}
                disabled={isSaving}
              >
                {isSaving ? "Saving..." : "Save"}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}