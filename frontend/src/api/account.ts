const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;

import type { LoginRequest, SignUpRequest } from "../models/account";

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
/// # Returns if login was successful.
/// # Throws Error with message to be displayed.
export async function apiLogin(payload: LoginRequest): Promise<void> {
	try {
		const response = await fetch(`${API_BASE_URL}/api/account/login`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: "include", // needed param for dealing with cookies
			body: JSON.stringify(payload)
		});

		// handle all errors from backend
		if (!response.ok) {
			if (response.status === 400) {
				throw new Error("Invalid email or password.");
			} else if (response.status === 500) {
				throw new Error("Server error.");
			} else {
				throw new Error(`Unexpected error: ${response.status}`);
			}
		}
		return;
	} catch (error) {
		console.error("Login API error: ", error);
		throw error;
	}
}

/// Calls signup
///
/// # Method
/// Sends a `POST /api/account/signup` request to create a new user account.
///
/// # Returns if account creation was successful.
/// # Throws Error with message to be displayed.
export async function apiSignUp(payload: SignUpRequest): Promise<void> {
	try {
		const response = await fetch(`${API_BASE_URL}/api/account/signup`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: "include", // needed param for dealing with cookies
			body: JSON.stringify(payload)
		});

		// handle all errors from backend
		if (!response.ok) {
			if (response.status === 409) {
				throw new Error(
					"An account was already created with this email address."
				);
			} else if (response.status === 400) {
				throw new Error("Bad Request");
			} else if (response.status === 500) {
				throw new Error("Server error.");
			} else {
				throw new Error(`Unexpected error: ${response.status}`);
			}
		}
		return;
	} catch (error) {
		console.error("Sign Up API error: ", error);
		throw error;
	}
}

/// Calls logout
///
/// # Method
/// Sends a `GET /api/account/logout` request set cookie as expired.
///
/// # Returns if account creation was successful.
/// # Throws Error with message to be displayed.
export async function apiLogout(): Promise<void> {
	console.log("Calling logout API");

	try {
		const response = await fetch(`${API_BASE_URL}/api/account/logout`, {
			method: "GET",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: "include" // needed param for dealing with cookies
		});

		// handle all errors from backend
		if (!response.ok) {
			if (response.status === 401) {
				throw new Error("Already logged out");
			} else if (response.status === 500) {
				throw new Error("Server error.");
			} else {
				throw new Error(`Unexpected error: ${response.status}`);
			}
		}
		return;
	} catch (error) {
		console.error("Logout Up API error: ", error);
		throw error;
	}
}

/// Calls validate
///
/// # Method
/// Sends a `GET /api/account/validate` request set cookie as expired.
///
/// # Returns whether session is currently valid.
/// # Throws Error with message.
export async function apiValidate(): Promise<boolean> {
	console.log("Calling validate API");

	try {
		const response = await fetch(`${API_BASE_URL}/api/account/validate`, {
			method: "GET",
			headers: {
				"Content-Type": "application/json"
			},
			credentials: "include" // needed param for dealing with cookies
		});

		// handle all errors from backend
		if (!response.ok) {
			if (response.status === 401) {
				return false;
			} else if (response.status === 500) {
				throw new Error("Server error.");
			} else {
				throw new Error(`Unexpected error: ${response.status}`);
			}
		}
		return true;
	} catch (error) {
		console.error("Validate API error: ", error);
		throw error;
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


		// Read the response body as JSON if possible
		const data = await response.json().catch(() => null);
		console.log("Response body:", data);
		
		// check if any preferences were not yet filled out 
		if(data.budget_preference === null || data.disabilities === null || data.food_allergies === null || data.risk_preference === null) {
			return false;
		}

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
		
		return true;
	
	} catch (error) {
		console.error("Current API error: ", error);
		throw error;
	}
}
/// Calls update account
///
/// # Method
/// Sends a `POST /api/account/update` request to update user account details.
///
/// # Returns updated account information if successful.
/// # Throws Error with message to be displayed.
export async function apiUpdateAccount(payload: UpdateRequest): Promise<UpdateResponse> {
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

        return await response.json();
    } catch (error) {
        console.error("Update Account API error: ", error);
        throw error;
    }
}

/// Get current account information
///
/// # Method
/// Sends a `GET /api/account/profile` request to fetch user account details.
///
/// # Returns current account information if successful.
/// # Throws Error with message to be displayed.
export async function apiGetProfile(): Promise<UpdateResponse> {
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

        return await response.json();
    } catch (error) {
        console.error("Get Profile API error: ", error);
        throw error;
    }
}

/// Sends a `GET /api/itinerary/saved` request to fetch all saved itineraries.
///
/// # Returns list of saved itineraries if successful.
/// # Throws Error with message to be displayed.
export async function apiGetSavedItineraries(): Promise<SavedItinerariesResponse> {
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

        return await response.json();
    } catch (error) {
        console.error("Get Saved Itineraries API error: ", error);
        throw error;
    }
}