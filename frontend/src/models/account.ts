export type LoginRequest = {
	email: string;
	/// Plaintext password
	password: string;
};

export type SignUpRequest = {
	email: string;
	first_name: string;
	last_name: string;
	/// Plaintext password
	password: string;
};

export type UpdateRequest = {
	/// Optional new email
	email: string | null;
	/// Optional new first name
	first_name: string | null;
	/// Optional new last name
	last_name: string | null;
	/// Optional new plaintext password
	password: string | null;
	/// Current password (required if password is being updated)
	current_password: string | null;
	/// Optional new budget enum
	budget_preference: BudgetBucket | null;
	/// Optional new risk enum
	risk_preference: RiskTolerence | null;
	/// Optional new food and allergies preferences
	/// * String is a comma-separated list of preferences
	food_allergies: string | null;
	/// Optional new disabilites
	/// * String is a comma-separated list of preferences
	disabilities: string | null;
};

/// API route response for POST `/api/account/update`.
/// - Contains full updated account profile for convenience.
export type UpdateResponse = {
	/// Current email
	email: string;
	/// Current first name
	first_name: string;
	/// Current last name
	last_name: string;
	/// Optional budget enum
	budget_preference: BudgetBucket | null;
	/// Optional risk enum
	risk_preference: RiskTolerence | null;
	/// Optional food and allergies preferences
	/// * String is a comma-separated list of preferences
	food_allergies: string | null;
	/// Optional disabilites
	/// * String is a comma-separated list of preferences
	disabilities: string | null;
};

/// API route response for GET `/api/account/current`.
/// - Safe-to-return account profile for current user
export type CurrentResponse = {
	/// Email
	email: string;
	/// First name
	first_name: string;
	/// Last name
	last_name: string;
	/// Optional budget enum
	budget_preference: BudgetBucket | null;
	/// Optional risk enum
	risk_preference: RiskTolerence | null;
	/// Optional food and allergies preferences
	food_allergies: string | null;
	/// Optional food and allergies preferences
	disabilities: string | null;
};

export enum BudgetBucket {
	VeryLowBudget = "VeryLowBudget",
	LowBudget = "LowBudget",
	MediumBudget = "MediumBudget",
	HighBudget = "HighBudget",
	LuxuryBudget = "LuxuryBudget"
}

export enum RiskTolerence {
	ChillVibes = "ChillVibes",
	LightFun = "LightFun",
	Adventurer = "Adventurer",
	RiskTaker = "RiskTaker"
}
