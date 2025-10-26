const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;

import type { ApiResult } from "../helpers/global";
import type {
	CurrentResponse,
	LoginRequest,
	SignUpRequest
} from "../models/account";

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
