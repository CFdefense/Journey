const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;
import type { ApiResult } from "../helpers/global";
import type {
	Itinerary,
	SavedItinerariesResponse,
	SaveResponse,
	SearchEventRequest,
	SearchEventResponse,
	UnsaveRequest,
	UserEventRequest,
	UserEventResponse
} from "../models/itinerary";

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
				credentials: import.meta.env.DEV ? "include" : "same-origin"
			}
		);
		return {
			result: response.ok ? await response.json() : null,
			status: response.status
		};
	} catch (error) {
		console.error("apiItineraryDetails error:", error);
		return { result: null, status: -1 };
	}
}

/// Saves or updates an itinerary for the authenticated user
///
/// # Method
/// Sends a `POST /api/itinerary/save` request to insert a new itinerary
/// or update an existing one in the database.
///
/// # Parameters
/// - `payload`: The complete `Itinerary` object to save. If the itinerary ID
///   already exists for this user, it will be updated. Otherwise, a new
///   itinerary will be created.
///
/// # Returns
/// - On success: A `SaveResponse` object containing the ID of the saved itinerary.
/// - On failure: Throws an error with details about the failure.
///
/// # Exceptions
/// Never throws an exception
export async function apiSaveItineraryChanges(
	payload: Itinerary
): Promise<ApiResult<SaveResponse>> {
): Promise<ApiResult<SaveResponse>> {
	try {
		const response = await fetch(`${API_BASE_URL}/api/itinerary/save`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: import.meta.env.DEV ? "include" : "same-origin",
			body: JSON.stringify(payload)
		});

		return { result: await response.json(), status: response.status };
	} catch (error) {
		console.error("apiSaveItineraryChanges error:", error);
		return { result: null, status: -1 };
	}
}

/// Unsaves an existing itinerary for the authenticated user
///
/// # Method
/// Sends a `POST /api/itinerary/unsave` request to set the saved field
/// to false for the specified itinerary.
///
/// # Parameters
/// - `payload`: The itinerary id to unsave. The itinerary must
///   belong to the authenticated user.
///
/// # Returns
/// - On success: status code of 200
/// - On failure: non-200 status code
///
/// # Exceptions
/// Never throws an exception
export async function apiUnsaveItinerary(
	payload: UnsaveRequest
): Promise<ApiResult<void>> {
	try {
		const response = await fetch(`${API_BASE_URL}/api/itinerary/unsave`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: import.meta.env.DEV ? "include" : "same-origin",
			body: JSON.stringify(payload)
		});
		return { result: null, status: response.status };
	} catch (error) {
		console.error("apiUnsaveItinerary error:", error);
		return { result: null, status: -1 };
	}
}

/// Inserts or updates a user-created event
///
/// # Method
/// Sends a `POST /api/itinerary/userEvent` request to insert or
/// update a user-created event in the db
///
/// # Body
/// - `UserEventRequest`: If the id is specified, it will try to update that event.
/// Otherwise it will insert a new one. Event name is always required.
///
/// # Returns
/// - On success: The `UserEventResponse` object returned by the backend.
/// - On failure: A non-200 status code.
///
/// # Exceptions
/// Never throws an exception
export async function apiUserEvent(
	userEvent: UserEventRequest
): Promise<ApiResult<UserEventResponse>> {
	try {
		const response = await fetch(
			`${API_BASE_URL}/api/itinerary/userEvent`,
			{
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				credentials: import.meta.env.DEV ? "include" : "same-origin",
				body: JSON.stringify(userEvent)
			}
		);
		return {
			result: response.ok ? await response.json() : null,
			status: response.status
		};
	} catch (error) {
		console.error("apiUserEvent error:", error);
		return { result: null, status: -1 };
	}
}

/// Searches the db for events based on the search filters
///
/// # Method
/// Sends a `POST /api/itinerary/searchEvent` request to fetch a list
/// of events that match the query parameters.
///
/// # Body
/// - `SearchEventRequest`: Optional search parameters to query with.
///
/// # Returns
/// - On success: The `SearchEventResponse` object returned by the backend.
/// - On failure: A non-200 status code.
///
/// # Exceptions
/// Never throws an exception
export async function apiSearchEvent(
	query: SearchEventRequest
): Promise<ApiResult<SearchEventResponse>> {
	// TODO: get events from cache if possible
	try {
		const response = await fetch(
			`${API_BASE_URL}/api/itinerary/searchEvent`,
			{
				method: "POST",
				headers: {
					"Content-Type": "application/json"
				},
				credentials: import.meta.env.DEV ? "include" : "same-origin",
				body: JSON.stringify(query)
			}
		);
		return {
			result: response.ok ? await response.json() : null,
			status: response.status
		};
	} catch (error) {
		console.error("apiSearchEvent error:", error);
		return { result: null, status: -1 };
	}
}

/// Deletes a user-created event from the db
///
/// # Method
/// Sends a `DELETE /api/itinerary/userEvent/{id}` request to delete the event
/// with the provided id.
///
/// # Returns
/// - On success: 200 status code.
/// - On failure: A non-200 status code.
///
/// # Exceptions
/// Never throws an exception
export async function apiDeleteUserEvent(id: number): Promise<ApiResult<void>> {
	//TODO remove event from cache
	try {
		const response = await fetch(
			`${API_BASE_URL}/api/itinerary/userEvent/${id}`,
			{
				method: "DELETE",
				credentials: import.meta.env.DEV ? "include" : "same-origin"
			}
		);
		return { result: null, status: response.status };
	} catch (error) {
		console.error("apiDeleteUserEvent error:", error);
		return { result: null, status: -1 };
	}
}

/// Sends a `GET /api/itinerary/saved` request to fetch all saved itineraries.
///
/// # Returns list of saved itineraries if successful.
/// # Throws Error with message to be displayed.
export async function apiGetSavedItineraries(): Promise<
	ApiResult<SavedItinerariesResponse>
> {
	try {
		const response = await fetch(`${API_BASE_URL}/api/itinerary/saved`, {
			method: "GET",
			credentials: "include"
		});
		return {
			result: response.ok ? await response.json() : null,
			status: response.status
		};
	} catch (error) {
		console.error("Get Saved Itineraries API error: ", error);
		return { result: null, status: -1 };
	}
}
