import { describe, expect, test } from "vitest";
import { checkIfValidName, checkIfValidPassword, checkIfPasswordsMatch } from "../src/helpers/account";

describe("Unit Tests", () => {
	test("Test Names", () => {
		expect(checkIfValidName("First", "Last")).toBe(null);
		expect(checkIfValidName("", "Last")).toBe("First and last name are required.");
		expect(checkIfValidName("First", "")).toBe("First and last name are required.");
		expect(checkIfValidName("F".repeat(51), "Last")).toBe("Names must be 50 characters or fewer.");
		expect(checkIfValidName("First", "L".repeat(51))).toBe("Names must be 50 characters or fewer.");
	});
	test("Test Passwords", () => {
		expect(checkIfValidPassword("1234567")).toBe("Password must be at least 8 characters long.");
		expect(checkIfValidPassword("a".repeat(129))).toBe("Password must be 128 characters or fewer.");
		expect(checkIfValidPassword("12345678\u{1F600}")).toBe("Password must contain only ASCII characters.");
		expect(checkIfValidPassword("abcdefgh")).toBe("Password must contain at least one uppercase letter.");
		expect(checkIfValidPassword("ABCDEFGH")).toBe("Password must contain at least one lowercase letter.");
		expect(checkIfValidPassword("ABCDefgh")).toBe("Password must contain at least one number.");
		expect(checkIfValidPassword("ABCdef123")).toBe(null);
	});
	test("Test Passwords Match", () => {
		expect(checkIfPasswordsMatch("ABCdef123", "abcDEF123")).toBe("Passwords do not match.");
		expect(checkIfPasswordsMatch("ABCdef123", "ABCdef123")).toBe(null);
	});
});