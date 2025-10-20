import { useState, useEffect } from "react";
import Itinerary from "../components/Itinerary";
import UnassignedEvents from "../components/UnassignedEvents";
import type { Event } from "../components/UnassignedEvents";

import "../styles/Itinerary.css";
import { apiItineraryDetails } from "../api/itinerary";

export default function ViewItineraryPage() {
  const [itineraryData, setItineraryData] = useState<any>(null);

  const unassignedEvents: Event[] = [
    { id: "1", title: "Breakfast", desc: "Saxbys coffee and bagel" },
    { id: "2", title: "Meeting", desc: "Capping discussion" },
  ];

  const onDragStart = (e: React.DragEvent, event: Event) => {
    e.dataTransfer.setData("eventId", event.id);
    e.dataTransfer.setData("eventTitle", event.title);
    e.dataTransfer.setData("eventDesc", event.desc || "");
  };

  useEffect(() => {
    const testId = 3;
    apiItineraryDetails(testId)
    .then((data) => {
      console.log("Fetched itinerary:", data);
      setItineraryData(data);
    })
    .catch((err) => console.error("Error fetching itinerary", err));
  }, []);

  return (
    <div className="view-page">
      <UnassignedEvents events={unassignedEvents} onDragStart={onDragStart} />
      <Itinerary />
      <button>Edit with AI</button>
    </div>
  );
}
