const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;
import type { Itinerary, EventDay, Event } from "../models/itinerary";

/// Calls itinerary details
///
/// # Method
/// Sends a `GET /api/itinerary/:itinerary_id` request to fetch the full details
/// of a specific itinerary for the currently authenticated user.
///
/// # Parameters
/// - `itinerary_id`: The numeric ID of the itinerary to retrieve.
///
/// # Returns
/// - On success: The `Itinerary` object returned by the backend.
/// - On failure: A placeholder itinerary with default values.
///
/// # Errors
/// - Logs detailed error messages based on the response status code:
///   - `401`: User is not authenticated.
///   - `404`: The itinerary does not exist or does not belong to the user.
///   - `500`: Internal server error.
/// - Returns a placeholder itinerary if the request fails or encounters a network error.
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
      // The backend returns { itinerary: Itinerary }
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

export async function saveItinerary(itinerary:Itinerary): Promise<Itinerary> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/itinerary/save`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: import.meta.env.DEV ? "include" : "same-origin",
      body: JSON.stringify(itinerary),
    });
    if (response.ok) {
      const data = await response.json();
      // The backend responds with { id: number } inside SaveResponse
      // Attach this id back to the itinerary we just saved
      return {...itinerary, id: data.id};
    }

    switch (response.status) {
      case 400:
        console.error("Bad Request: Invalid itinerary data.");
        break;
      case 401:
        console.error("Unauthorized: User is not authenticated.");
        break;
      case 405:
        console.error("Method Not Allowed: Must use POST.");
        break;
      case 408:
        console.error("Request Timed Out.");
        break;
      case 500:
        console.error("Internal Server Error: Something went wrong on the server.");
        break;
      default:
        console.warn(`Unexpected response status: ${response.status}`);
    }

    throw new Error(`Failed to save itinerary (status ${response.status})`);

  } catch (error) {
    console.error("saveItinerary network or parsing error:", error);
    throw error;
  }
}

// default itinerary for dealing with errors or when a placeholder is needed
function placeholderItinerary(itinerary_id: number): Itinerary {
  const emptyEvents: Event[] = [];

  const emptyDay: EventDay = {
    date: new Date().toISOString().split("T")[0],
    morning_events: emptyEvents,
    noon_events: emptyEvents,
    afternoon_events: emptyEvents,
    evening_events: emptyEvents,
  };

  return {
    id: itinerary_id,
    start_date: emptyDay.date,
    end_date: emptyDay.date,
    event_days: [emptyDay],
    chat_session_id: -1,
    title: "Untitled Itinerary",
  };
}
