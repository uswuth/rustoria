use library::error::ValidationError;
use library::types::{Age, EmailAddress, Url};
use library::validator;

#[test]
fn validate_email_accepts_valid() {
    assert!(validator::validate_email("user@example.com").is_ok());
    assert!(validator::validate_email("test.user+tag@domain.co.uk").is_ok());
}

#[test]
fn validate_email_rejects_invalid() {
    assert_eq!(
        validator::validate_email(""),
        Err(ValidationError::Required {
            field: "email".to_string()
        })
    );
    assert_eq!(
        validator::validate_email("not-an-email"),
        Err(ValidationError::InvalidEmail {
            field: "email".to_string()
        })
    );
    assert_eq!(
        validator::validate_email("missing@domain"),
        Err(ValidationError::InvalidEmail {
            field: "email".to_string()
        })
    );
}

#[test]
fn validate_url_accepts_valid() {
    assert!(validator::validate_url("https://example.com").is_ok());
    assert!(validator::validate_url("http://sub.domain.org/path").is_ok());
}

#[test]
fn validate_url_rejects_invalid() {
    assert_eq!(
        validator::validate_url(""),
        Err(ValidationError::Required {
            field: "url".to_string()
        })
    );
    assert_eq!(
        validator::validate_url("not-a-url"),
        Err(ValidationError::InvalidUrl {
            field: "url".to_string()
        })
    );
}

#[test]
fn validate_age_accepts_valid() {
    // Age must be at least 1: the contract is 1..=150.
    assert!(validator::validate_age(1).is_ok());
    assert!(validator::validate_age(25).is_ok());
    assert!(validator::validate_age(150).is_ok());
}

#[test]
fn validate_age_rejects_invalid() {
    assert_eq!(
        validator::validate_age(0),
        Err(ValidationError::BelowMinimum {
            field: "age".to_string(),
            min: 1
        })
    );
    assert_eq!(
        validator::validate_age(151),
        Err(ValidationError::AboveMaximum {
            field: "age".to_string(),
            max: 150
        })
    );
}

#[test]
fn validate_required_rejects_empty() {
    assert_eq!(
        validator::validate_required("", "name"),
        Err(ValidationError::Required {
            field: "name".to_string()
        })
    );
    assert!(validator::validate_required("value", "name").is_ok());
}

#[test]
fn validate_min_length() {
    assert_eq!(
        validator::validate_min_length("ab", "field", 3),
        Err(ValidationError::TooShort {
            field: "field".to_string(),
            min: 3
        })
    );
    assert!(validator::validate_min_length("abc", "field", 3).is_ok());
}

#[test]
fn validate_max_length() {
    assert_eq!(
        validator::validate_max_length("abcd", "field", 3),
        Err(ValidationError::TooLong {
            field: "field".to_string(),
            max: 3
        })
    );
    assert!(validator::validate_max_length("abc", "field", 3).is_ok());
}

#[test]
fn email_address_type() {
    let email = EmailAddress::new("user@example.com").unwrap();
    assert_eq!(email.as_str(), "user@example.com");
    assert_eq!(email.to_string(), "user@example.com");
}

#[test]
fn email_address_rejects_invalid() {
    assert!(EmailAddress::new("invalid").is_err());
}

#[test]
fn url_type() {
    let url = Url::new("https://example.com").unwrap();
    assert_eq!(url.as_str(), "https://example.com");
}

#[test]
fn age_type() {
    let age = Age::new(25).unwrap();
    assert_eq!(age.value(), 25);
}

#[test]
fn age_rejects_invalid() {
    assert!(Age::new(0).is_err());
    assert!(Age::new(200).is_err());
}

#[test]
fn validate_slug_accepts_valid() {
    assert!(validator::validate_slug("hello-world", "slug").is_ok());
    assert!(validator::validate_slug("rust", "slug").is_ok());
}

#[test]
fn validate_slug_rejects_invalid() {
    assert_eq!(
        validator::validate_slug("", "slug"),
        Err(ValidationError::Required {
            field: "slug".to_string()
        })
    );
    assert_eq!(
        validator::validate_slug("Not A Slug!", "slug"),
        Err(ValidationError::InvalidFormat {
            field: "slug".to_string()
        })
    );
    assert_eq!(
        validator::validate_slug("--leading-hyphens", "slug"),
        Err(ValidationError::InvalidFormat {
            field: "slug".to_string()
        })
    );
}

#[test]
fn deserialize_valid_newtypes() {
    let email: EmailAddress = serde_json::from_str("\"user@example.com\"").unwrap();
    assert_eq!(email.as_str(), "user@example.com");

    let url: Url = serde_json::from_str("\"https://example.com\"").unwrap();
    assert_eq!(url.as_str(), "https://example.com");

    let age: Age = serde_json::from_str("25").unwrap();
    assert_eq!(age.value(), 25);
}

#[test]
fn deserialize_rejects_invalid_input() {
    // Deserialization must run the validators, not bypass them.
    let email: Result<EmailAddress, _> = serde_json::from_str("\"not-an-email\"");
    assert!(email.is_err(), "invalid email must not deserialize");

    let url: Result<Url, _> = serde_json::from_str("\"not-a-url\"");
    assert!(url.is_err(), "invalid URL must not deserialize");

    let age: Result<Age, _> = serde_json::from_str("0");
    assert!(age.is_err(), "out-of-range age must not deserialize");

    let age: Result<Age, _> = serde_json::from_str("200");
    assert!(age.is_err(), "out-of-range age must not deserialize");
}
