/*
 * src/models/Account.rs
 *
 * File for Account table models
 *
 * Purpose:
 *   Models for the account table and payloads which interact with it.
 *
 * Include:
 *   Account            - Model representing an instance of the Account table
 *   LoginPayload       - Model representing the payload for a login
 *   SignupPayload      - Model representing the payload for a signup
 */

use serde::{Deserialize, Serialize};
use regex::Regex;

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    // TODO: More Preferences...
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SignupPayload {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
}

impl SignupPayload {
    /// Validate email format using regex
    fn validate_email(email: &str) -> bool {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        email_regex.is_match(email)
    }

    /// Validate password strength
    /// - Minimum 8 characters
    /// - Maximum 128 characters
    /// - At least one uppercase letter
    /// - At least one lowercase letter
    /// - At least one number
    /// - Only ASCII characters allowed (for security and compatibility)
    fn validate_password(password: &str) -> Result<(), String> {
        if password.len() < 8 {
            return Err("Password must be at least 8 characters long".to_string());
        }
        
        if password.len() > 128 {
            return Err("Password must be 128 characters or less".to_string());
        }
        
        // Only allow ASCII characters (prevents potential encoding issues)
        if !password.is_ascii() {
            return Err("Password must contain only ASCII characters".to_string());
        }
        
        if !password.chars().any(|c| c.is_uppercase()) {
            return Err("Password must contain at least one uppercase letter".to_string());
        }
        
        if !password.chars().any(|c| c.is_lowercase()) {
            return Err("Password must contain at least one lowercase letter".to_string());
        }
        
        if !password.chars().any(|c| c.is_numeric()) {
            return Err("Password must contain at least one number".to_string());
        }
        
        Ok(())
    }

    /// Validate the entire signup payload
    pub fn validate(&self) -> Result<(), String> {
        // Validate email (trim before checking)
        let email_trimmed = self.email.trim();
        if email_trimmed.is_empty() {
            return Err("Email is required".to_string());
        }
        
        if !Self::validate_email(email_trimmed) {
            return Err("Invalid email format".to_string());
        }
        
        // Validate first name (trim before checking)
        let first_name_trimmed = self.first_name.trim();
        if first_name_trimmed.is_empty() {
            return Err("First name is required".to_string());
        }
        
        if first_name_trimmed.len() > 50 {
            return Err("First name must be 50 characters or less".to_string());
        }
        
        // Validate last name (trim before checking)
        let last_name_trimmed = self.last_name.trim();
        if last_name_trimmed.is_empty() {
            return Err("Last name is required".to_string());
        }
        
        if last_name_trimmed.len() > 50 {
            return Err("Last name must be 50 characters or less".to_string());
        }
        
        // Validate password
        Self::validate_password(&self.password)?;
        
        Ok(())
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub id: i32,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupResponse {
    pub id: i32,
    pub email: String,
}

// TODO: More Payloads...

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(result.unwrap_err(), "First name must be 50 characters or less");
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
        assert_eq!(result.unwrap_err(), "Last name must be 50 characters or less");
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
}
