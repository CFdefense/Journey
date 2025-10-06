const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;

export interface LoginRequest {
    email: string; 
    password: string;
}

export interface LoginResponse {
    id: number; 
    token: string; 
}

/// Handles user login via the Rust backend API.
///
/// # Method
/// Calls `POST /account/login` through `apiLogin`.
///
/// # Behavior
/// - Collects email and password input from the user.
/// - Sends login request to backend.
/// - On success: stores auth token and navigates to `/create`.
/// - On failure: displays the error message received from backend.
///
/// # Returns
/// A rendered login form component that manages authentication flow.

export async function apiLogin(payload: LoginRequest): Promise<LoginResponse> {
  console.log("Calling login API with payload:", payload);
  
    try {
    const response = await fetch(`${API_BASE_URL}/account/login`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include", // needed param for dealing with cookies
      body: JSON.stringify(payload),
    });

    // handle all errors from backend
    if(!response.ok) {
        if (response.status === 400) {
        throw new Error("Invalid email or password.");
      } else if (response.status === 500) {
        throw new Error("Server error.");
      } else {
        throw new Error(`Unexpected error: ${response.status}`);
      }
    }
    const res: LoginResponse = await response.json();
    return res;
    
    } catch (error) {
        console.error("Login API error: ", error);
        throw error;
    }

}