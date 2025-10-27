const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;

import type { ApiResult } from "../helpers/global";
import type {
	CurrentResponse,
	LoginRequest,
	SignUpRequest
} from "../models/account";

export interface UpdateRequest {
    email?: string;
    first_name?: string;
    last_name?: string;
    password?: string;
    budget_preference?: string; // BudgetBucket enum
    risk_preference?: string;   // RiskTolerance enum
    food_allergies?: string;
    disabilities?: string;
}

export interface UpdateResponse {
    id: number;
    email: string;
    first_name: string;
    last_name: string;
    budget_preference?: string;
    risk_preference?: string;
    food_allergies?: string;
    disabilities?: string;
}
export interface Event {
    id?: number;
    event_name: string;
    event_description: string;
    event_type: string;
    street_address: string;
    city: string;
    postal_code: number;
}

export interface EventDay {
    date: string;
    morning_events: Event[];
    noon_events: Event[];
    afternoon_events: Event[];
    evening_events: Event[];
}

// The API returns event days directly, not full itinerary objects
export interface SavedItinerariesResponse {
    itineraries: EventDay[];
}

export interface Itinerary {
    id: number;
    chat_session_id: number;
    title: string;
    start_date: string;
    end_date: string;
    event_days: EventDay[];
}


/// Calls login
///
/// # Method
/// Calls `POST /api/account/login` through `apiLogin`.
///
/// # Returns
/// Status of login call.
/// * 200: successful login
/// * -1: fetch call threw an exception
///
/// # Exceptions
/// Never throws an exception
export async function apiLogin(payload: LoginRequest): Promise<ApiResult<void>> {
    try {
        const response = await fetch(`${API_BASE_URL}/api/account/login`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            credentials: import.meta.env.DEV ? "include" : "same-origin",
            body: JSON.stringify(payload)
        });
        return { result: null, status: response.status };
    } catch (error) {
        console.error("Login API error: ", error);
        return { result: null, status: -1 };
    }
}

/// Calls signup
///
/// # Method
/// Sends a `POST /api/account/signup` request to create a new user account.
///
/// # Returns
/// Status of signup call.
/// * 200: successful signup
/// * -1: fetch call threw an exception
///
/// # Exceptions
/// Never throws an exception
export async function apiSignUp(payload: SignUpRequest): Promise<ApiResult<void>> {
    try {
        const response = await fetch(`${API_BASE_URL}/api/account/signup`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            credentials: import.meta.env.DEV ? "include" : "same-origin",
            body: JSON.stringify(payload)
        });
        return { result: null, status: response.status };
    } catch (error) {
        console.error("Sign Up API error: ", error);
        return { result: null, status: -1 };
    }
}

/// Calls logout
///
/// # Method
/// Sends a `GET /api/account/logout` request set cookie as expired.
///
/// # Returns
/// Status of logout call.
/// * 200: successful logout
/// * -1: fetch call threw an exception
///
/// # Exceptions
/// Never throws an exception
export async function apiLogout(): Promise<ApiResult<void>> {
    try {
        const response = await fetch(`${API_BASE_URL}/api/account/logout`, {
            method: "GET",
            headers: {
                "Content-Type": "application/json"
            },
            credentials: import.meta.env.DEV ? "include" : "same-origin"
        });
        return { result: null, status: response.status };
    } catch (error) {
        console.error("Logout Up API error: ", error);
        return { result: null, status: -1 };
    }
}


/// Calls validate
///
/// # Method
/// Sends a `GET /api/account/validate` request set cookie as expired.
///
/// # Returns
/// Whether session is currently valid.
///
/// # Exceptions
/// Never throws an exception
export async function apiValidate(): Promise<ApiResult<void>> {
    try {
        const response = await fetch(`${API_BASE_URL}/api/account/validate`, {
            method: "GET",
            headers: {
                "Content-Type": "application/json"
            },
            credentials: import.meta.env.DEV ? "include" : "same-origin"
        });
        return { result: null, status: response.status };
    } catch (error) {
        console.error("Validate API error: ", error);
        return { result: null, status: -1 };
    }
}

/// Calls current
///
/// # Method
/// Sends a `GET /api/account/current` request set cookie as expired.
///
/// # Returns whether current user has filled out preferences or not.
/// # Throws Error with message.
export async function apiCheckIfPreferencesPopulated(): Promise<boolean> {
	console.log("Calling validate API");

	try {

		const response = await fetch(`${API_BASE_URL}/api/account/current`, {
			method: "GET",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: "include" // needed param for dealing with cookies
		});

		// handle all errors from backend
		if (!response.ok) {
			if (response.status === 401) {
				throw new Error("Invalid Credentials.")
			} else if (response.status === 500) {
				throw new Error("Server error.");
			} else {
				throw new Error(`Unexpected error: ${response.status}`);
			}
		}

		// Read the response body as JSON if possible
		const data = await response.json().catch(() => null);
		console.log("Response body:", data);
		
		// check if any preferences were not yet filled out 
		if(data.budget_preference === null || data.disabilities === null || data.food_allergies === null || data.risk_preference === null) {
			return false;
		}

		return true;
	
	} catch (error) {
		console.error("Current API error: ", error);
		return false;
	}
}
/// Calls update account
///
/// # Method
/// Sends a `POST /api/account/update` request to update user account details.
///
/// # Returns updated account information if successful.
/// # Throws Error with message to be displayed.
export async function apiUpdateAccount(payload: UpdateRequest): Promise<ApiResult<CurrentResponse>> {
    try {
        const response = await fetch(`${API_BASE_URL}/api/account/update`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            credentials: "include",
            body: JSON.stringify(payload)
        });

        if (!response.ok) {
            if (response.status === 400) {
                throw new Error("Invalid input data.");
            } else if (response.status === 401) {
                throw new Error("Not authenticated.");
            } else if (response.status === 500) {
                throw new Error("Server error.");
            } else {
                throw new Error(`Unexpected error: ${response.status}`);
            }
        }

        return { result: await response.json(), status: response.status };
    } catch (error) {
        console.error("Update Account API error: ", error);
        return { result: null, status: -1 };
    }
}

/// Get current account information
///
/// # Method
/// Sends a `GET /api/account/profile` request to fetch user account details.
///
/// # Returns current account information if successful.
/// # Throws Error with message to be displayed.
export async function apiGetProfile(): Promise<ApiResult<CurrentResponse>> {
    try {
        const response = await fetch(`${API_BASE_URL}/api/account/current`, {
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
        console.error("Get Profile API error: ", error);
        return { result: null, status: -1 };
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
/// Calls current
///
/// # Method
/// Sends a `GET /api/account/current` request set cookie as expired.
///
/// # Returns
/// Non-sensitive account info and the status of the API call
/// * 200: successful fetch
/// * -1: fetch call threw an exception
///
/// # Exceptions
/// Never throws an exception
export async function apiCurrent(): Promise<ApiResult<CurrentResponse>> {
	// TODO: get account data from cache if it exists
	try {
		const response = await fetch(`${API_BASE_URL}/api/account/current`, {
			method: "GET",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: import.meta.env.DEV ? "include" : "same-origin"
		});
		if (!response.ok) {
			return { result: null, status: response.status };
		}
		return { result: await response.json(), status: response.status };
	} catch (error) {
		console.error("Current API error: ", error);
		return { result: null, status: -1 };
	}
}