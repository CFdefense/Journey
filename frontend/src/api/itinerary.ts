const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;
import type { ApiResult } from "../helpers/global";
import type { Itinerary } from "../models/itinerary";

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
		console.error("apiItineraryDetails network error:", error);
		return { result: null, status: -1 };
	}
}
