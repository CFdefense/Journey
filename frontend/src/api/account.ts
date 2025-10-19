const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;

import type { LoginRequest, SignUpRequest } from "../models/account";

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
			credentials: import.meta.env.DEV? "include" : "same-origin",
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
			credentials: import.meta.env.DEV? "include" : "same-origin",
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
			credentials: import.meta.env.DEV? "include" : "same-origin",
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
			credentials: import.meta.env.DEV? "include" : "same-origin",
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
			credentials: import.meta.env.DEV? "include" : "same-origin",
		});


		// Read the response body as JSON if possible
		const data = await response.json().catch(() => null);

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