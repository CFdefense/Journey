const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;

export interface Event {
	id: string;
	title: string;
	desc?: string;
	time?: string;
	address?: string;
	postal_code?: number;
	city?: string;
	type?: string;
}

/// Calls `GET /api/itinerary/saved`
///
/// Fetches all saved itineraries from the backend database for authenticated user.
///
/// # Returns
/// A list of itineraries.
/// # Throws
/// Error with descriptive message if fetching fails.
export async function apiGetItinerary(): Promise<Event[]> {
	try {
		const response = await fetch(`${API_BASE_URL}/api/itinerary/saved`, {
			method: "GET",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: "include"
		});

		if (!response.ok) {
			if (response.status === 401) {
				throw new Error("Unauthorized. Please log in again.");
			} else if (response.status === 404) {
				throw new Error("No events found.");
			} else if (response.status === 500) {
				throw new Error("Server error while fetching events.");
			} else {
				throw new Error(`Unexpected error: ${response.status}`);
			}
		}

		const data = await response.json();
		return data as Event[];
	} catch (error) {
		console.error("Get Events API error: ", error);
		throw error;
	}
}
