export function checkIfValidName(
	firstName: string,
	lastName: string
): string | null {
	if (firstName.length === 0 || lastName.length === 0) {
		return "First and last name are required.";
	}
	if (firstName.length > 50 || lastName.length > 50) {
		return "Names must be 50 characters or fewer.";
	}
	return null;
}

export function checkIfValidPassword(password: string): string | null {
	if (password.length < 8) {
		return "Password must be at least 8 characters long.";
	}

	if (password.length > 128) {
		return "Password must be 128 characters or fewer.";
	}

	// eslint-disable-next-line no-control-regex
	if (!/^[\x00-\x7F]+$/.test(password)) {
		return "Password must contain only ASCII characters.";
	}

	if (!/[A-Z]/.test(password)) {
		return "Password must contain at least one uppercase letter.";
	}

	if (!/[a-z]/.test(password)) {
		return "Password must contain at least one lowercase letter.";
	}

	if (!/[0-9]/.test(password)) {
		return "Password must contain at least one number.";
	}

	return null;
}

export function checkIfPasswordsMatch(
	password: string,
	confirmPassword: string
): string | null {
	if (password !== confirmPassword) {
		return "Passwords do not match.";
	}
	return null;
}
