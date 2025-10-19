const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;
import type { Itinerary, Event} from "../models/itinerary";


export async function apiItineraryDetails(itinerary_id: number): Promise<Itinerary> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/itinerary/${itinerary_id}`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: import.meta.env.DEV ? "include" : "same-origin",
    });

    if (response.ok) {
      const data = await response.json();
      return data ?? placeholderItinerary(itinerary_id); 
    }

    switch (response.status) {
      case 401:
        console.error("Unauthorized: User is not authenticated.");
        break;
      case 404:
        console.error("Not Found: Itinerary does not exist or does not belong to user.");
        break;
      case 500:
        console.error("Internal Server Error: Something went wrong on the server.");
        break;
      default:
        console.warn(`Unexpected response status: ${response.status}`);
    }

    return placeholderItinerary(itinerary_id);

  } catch (error) {
    console.error("apiItineraryDetails network error:", error);
    return placeholderItinerary(itinerary_id);
  }
}

// default itinerary for dealing with errors or when a placeholder is needed
function placeholderItinerary(itinerary_id: number): Itinerary {
  const emptyEvents: Event[] = [];
  return {
    id: itinerary_id,
    start_date: new Date().toISOString().split("T")[0], // today's date
    end_date: new Date().toISOString().split("T")[0],   // same as start
    morning_events: emptyEvents,
    noon_events: emptyEvents,
    afternoon_events: emptyEvents,
    evening_events: emptyEvents,
    chat_session_id: -1,
  };
}