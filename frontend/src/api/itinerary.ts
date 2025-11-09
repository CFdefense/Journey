const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;
import type { ApiResult } from "../helpers/global";
import type { Itinerary, SaveResponse } from "../models/itinerary";

// The API returns event days directly, not full itinerary objects
export interface SavedItinerariesResponse {
    itineraries: Itinerary[];
}
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
/// - On failure: A null itinerary with a non-200 status code.
///
/// # Exceptions
/// Never throws an exception
export async function apiItineraryDetails(
	itinerary_id: number
): Promise<ApiResult<Itinerary>> {
	// TODO: get itinerary from cache if it exists
	try {
		const response = await fetch(
			`${API_BASE_URL}/api/itinerary/${itinerary_id}`,
			{
				method: "GET",
				headers: {
					"Content-Type": "application/json"
				},
				credentials: import.meta.env.DEV ? "include" : "same-origin"
			}
		);
		return { result: await response.json(), status: response.status };
	} catch (error) {
		console.error("apiItineraryDetails error:", error);
		return { result: null, status: -1 };
	}
}

export async function saveItineraryChanges(
	payload: Itinerary
): Promise<SaveResponse> {
	try {
		const response = await fetch(`${API_BASE_URL}/api/itinerary/save`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: import.meta.env.DEV ? "include" : "same-origin",
			body: JSON.stringify(payload)
		});

		if (!response.ok) {
			const errorText = await response.text();
			throw new Error(
				`Failed to save itinerary: ${response.status} ${errorText}`
			);
		}

		// Parse and return the SaveResponse
		const data: SaveResponse = await response.json();
		return data;
	} catch (error) {
		console.error("Save API error:", error);
		throw error;
	}
}

/// Sends a `GET /api/itinerary/saved` request to fetch all saved itineraries.
///
/// # Returns list of saved itineraries if successful.
/// # Throws Error with message to be displayed.
export async function apiGetSavedItineraries(): Promise<ApiResult<SavedItinerariesResponse>> {
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
				throw new Error("Not authenticated.");
			} else if (response.status === 500) {
				throw new Error("Server error.");
			} else {
				throw new Error(`Unexpected error: ${response.status}`);
			}
		}

		return { result: await response.json(), status: response.status };
	} catch (error) {
		console.error("Get Saved Itineraries API error: ", error);
		return { result: null, status: -1 };
	}
}