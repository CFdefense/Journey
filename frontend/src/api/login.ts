const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;

export interface LoginRequest {
    email: string; 
    password: string;
}

export interface LoginResponse {
    id: number; 
    token: string; 
}

/**
 * Calls the Rust backend's /account/login endpoint to attempt user authentication.
 *
 * @param payload { email, password } The user's credentials.
 * @returns A promise resolving to LoginResponse if successful, or throws an error if not.
 */

export async function login(payload: LoginRequest): Promise<LoginResponse> {
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
      } else if (response.status === 501) {
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