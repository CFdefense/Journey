export interface LoginRequest {
	email: string;
	/// Plaintext password
	password: string;
}

export interface SignUpRequest {
	email: string;
	first_name: string;
	last_name: string;
	/// Plaintext password
	password: string;
}
