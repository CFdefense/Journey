import { useEffect, useState } from "react";
import Itinerary from "../components/Itinerary";
import UnassignedEvents from "../components/UnassignedEvents";
import type { Event } from "../components/UnassignedEvents";
import { apiItineraryDetails } from "../api/itinerary";
import { fetchItinerary } from "../helpers/populate_itinerary";
import type { TimeBlock } from "../helpers/populate_itinerary";
import "../styles/Itinerary.css";

export default function ViewItineraryPage() {
  const [timeBlocks, setTimeBlocks] = useState<TimeBlock[]>([]);

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
    async function load() {
      const data = await fetchItinerary(3); //give the itinerary ID of the itinerary you want to display
      setTimeBlocks(data);
    }
    load();
  }, []);

  return (
    <div className="view-page">
      <UnassignedEvents events={unassignedEvents} onDragStart={onDragStart} />
      <Itinerary initialTimeBlocks={timeBlocks}/>
      <button>Edit with AI</button>
    </div>
  );
}
