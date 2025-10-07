export interface LoginRequest {
    email: string; 
    password: string;
}

export interface LoginResponse {
    id: number; 
    token: string; 
}

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