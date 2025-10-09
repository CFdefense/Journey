/*
 * tests/models/account.rs
 *
 * Unit tests for Account Models
 *
 * Purpose:
 *   Test account model validation logic including email validation,
 *   password strength requirements, and signup payload validation.
 */

use capping2025::models::account::SignupPayload;

// ===== Email Validation Tests =====

#[test]
fn test_validate_email_valid() {
    assert!(SignupPayload::validate_email("user@example.com"));
    assert!(SignupPayload::validate_email("test.user@domain.co.uk"));
    assert!(SignupPayload::validate_email("name+tag@company.org"));
    assert!(SignupPayload::validate_email("user123@test-domain.com"));
}

#[test]
fn test_validate_email_invalid() {
    assert!(!SignupPayload::validate_email(""));
    assert!(!SignupPayload::validate_email("notanemail"));
    assert!(!SignupPayload::validate_email("@example.com"));
    assert!(!SignupPayload::validate_email("user@"));
    assert!(!SignupPayload::validate_email("user@.com"));
    assert!(!SignupPayload::validate_email("user @example.com"));
    assert!(!SignupPayload::validate_email("user@exam ple.com"));
}

// ===== Password Validation Tests =====

#[test]
fn test_validate_password_valid() {
    assert!(SignupPayload::validate_password("Password1").is_ok());
    assert!(SignupPayload::validate_password("MySecure123").is_ok());
    assert!(SignupPayload::validate_password("Passw0rd!@#").is_ok());
    assert!(SignupPayload::validate_password("LongerPassword123").is_ok());
}

#[test]
fn test_validate_password_too_short() {
    let result = SignupPayload::validate_password("Pass1");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Password must be at least 8 characters long"
    );
}

#[test]
fn test_validate_password_no_uppercase() {
    let result = SignupPayload::validate_password("password123");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Password must contain at least one uppercase letter"
    );
}

#[test]
fn test_validate_password_no_lowercase() {
    let result = SignupPayload::validate_password("PASSWORD123");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Password must contain at least one lowercase letter"
    );
}

#[test]
fn test_validate_password_no_number() {
    let result = SignupPayload::validate_password("PasswordOnly");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Password must contain at least one number"
    );
}

#[test]
fn test_validate_password_too_long() {
    let password = "A".repeat(129) + "1a";
    let result = SignupPayload::validate_password(&password);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Password must be 128 characters or less"
    );
}

#[test]
fn test_validate_password_non_ascii() {
    let result = SignupPayload::validate_password("Password1パスワード");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Password must contain only ASCII characters"
    );
}

#[test]
fn test_validate_password_emoji() {
    let result = SignupPayload::validate_password("Password1🔒");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Password must contain only ASCII characters"
    );
}

#[test]
fn test_validate_password_max_length_allowed() {
    // Test that exactly 128 characters is okay (with required chars)
    let password = "A".to_string() + &"a".repeat(126) + "1";
    assert_eq!(password.len(), 128);
    assert!(SignupPayload::validate_password(&password).is_ok());
}

// ===== Signup Payload Validation Tests =====

#[test]
fn test_validate_signup_payload_valid() {
    let payload = SignupPayload {
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "Password123".to_string(),
    };
    assert!(payload.validate().is_ok());
}

#[test]
fn test_validate_signup_payload_empty_email() {
    let payload = SignupPayload {
        email: "".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "Password123".to_string(),
    };
    let result = payload.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Email is required");
}

#[test]
fn test_validate_signup_payload_invalid_email() {
    let payload = SignupPayload {
        email: "not-an-email".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "Password123".to_string(),
    };
    let result = payload.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Invalid email format");
}

#[test]
fn test_validate_signup_payload_empty_first_name() {
    let payload = SignupPayload {
        email: "test@example.com".to_string(),
        first_name: "".to_string(),
        last_name: "Doe".to_string(),
        password: "Password123".to_string(),
    };
    let result = payload.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "First name is required");
}

#[test]
fn test_validate_signup_payload_empty_last_name() {
    let payload = SignupPayload {
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "".to_string(),
        password: "Password123".to_string(),
    };
    let result = payload.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Last name is required");
}

#[test]
fn test_validate_signup_payload_first_name_too_long() {
    let payload = SignupPayload {
        email: "test@example.com".to_string(),
        first_name: "a".repeat(51),
        last_name: "Doe".to_string(),
        password: "Password123".to_string(),
    };
    let result = payload.validate();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "First name must be 50 characters or less"
    );
}

#[test]
fn test_validate_signup_payload_last_name_too_long() {
    let payload = SignupPayload {
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "a".repeat(51),
        password: "Password123".to_string(),
    };
    let result = payload.validate();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Last name must be 50 characters or less"
    );
}

#[test]
fn test_validate_signup_payload_weak_password() {
    let payload = SignupPayload {
        email: "test@example.com".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        password: "weak".to_string(),
    };
    let result = payload.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Password"));
}

#[test]
fn test_validate_signup_payload_whitespace_trimming() {
    let payload = SignupPayload {
        email: "  test@example.com  ".to_string(),
        first_name: "  John  ".to_string(),
        last_name: "  Doe  ".to_string(),
        password: "Password123".to_string(), // Valid ASCII password
    };
    // Email and names should be validated after trimming whitespace
    assert!(payload.validate().is_ok());
}