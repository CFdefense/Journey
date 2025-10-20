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

  //properly gets JSON
  useEffect(() => {
    async function fetchItinerary() {
      try {
        const testId = 3;
        const itinerary = await apiItineraryDetails(testId);
        console.log(itinerary)
        console.log("meow")
      } catch {
          console.log("Error")
      }
    }
    fetchItinerary();
  }, []);

  return (
    <div className="view-page">
      <UnassignedEvents events={unassignedEvents} onDragStart={onDragStart} />
      <Itinerary />
      <button>Edit with AI</button>
    </div>
  );
}
