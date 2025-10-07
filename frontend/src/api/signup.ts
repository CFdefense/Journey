const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;

export interface SignUpRequest {
    email: string; 
	  first_name: string;
	  last_name: string; 
    password: string;
}

export interface SignUpResponse {
    id: number; 
    email: string; 
}

/// Handles user registration via the Rust backend API.
///
/// # Method
/// Sends a `POST /account/signup` request to create a new user account.
///
/// # Behavior
/// - Accepts a `SignUpRequest` payload containing `email`, `first_name`, `last_name`, and `password`.
/// - Sends the data to the backend as JSON.
/// - On success: returns a `SignUpResponse` containing the created user’s info or token.
/// - On failure:
///   - Returns a user-friendly error if:
///     - `409 Conflict` → an account already exists with this email.
///     - `500 Internal Server Error` → a server-side issue occurred.
///   - Throws a generic error for any other unexpected status code.
///
/// # Returns
/// A `Promise<SignUpResponse>` resolving to the backend response if successful,  
/// or throws an error if registration fails.

export async function apiSignUp(payload: SignUpRequest): Promise<SignUpResponse> {
  console.log("Calling signup API with payload:", payload);
  
    try {
    const response = await fetch(`${API_BASE_URL}/account/signup`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include", // needed param for dealing with cookies
      body: JSON.stringify(payload),
    });

    // handle all errors from backend
    if(!response.ok) {
        if (response.status === 409) {
        throw new Error("An account was already created with this email address.");
      } else if (response.status === 500) {
        throw new Error("Server error.");
      } else {
        throw new Error(`Unexpected error: ${response.status}`);
      }
    }
    const res: SignUpResponse = await response.json();
    return res;
    
    } catch (error) {
        console.error("Sign Up API error: ", error);
        throw error;
    }

}