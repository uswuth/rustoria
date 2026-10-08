use std::sync::LazyLock;

use regex::Regex;

use crate::error::ValidationError;

static EMAIL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());

static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^https?://[a-zA-Z0-9][-a-zA-Z0-9]*(\.[a-zA-Z0-9][-a-zA-Z0-9]*)+(/.*)?$").unwrap()
});

static SLUG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap());

pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    if email.is_empty() {
        return Err(ValidationError::Required {
            field: "email".to_string(),
        });
    }
    if !EMAIL_REGEX.is_match(email) {
        return Err(ValidationError::InvalidEmail {
            field: "email".to_string(),
        });
    }
    Ok(())
}

pub fn validate_url(url: &str) -> Result<(), ValidationError> {
    if url.is_empty() {
        return Err(ValidationError::Required {
            field: "url".to_string(),
        });
    }
    if !URL_REGEX.is_match(url) {
        return Err(ValidationError::InvalidUrl {
            field: "url".to_string(),
        });
    }
    Ok(())
}

pub fn validate_age(age: u8) -> Result<(), ValidationError> {
    if age < 1 {
        return Err(ValidationError::BelowMinimum {
            field: "age".to_string(),
            min: 1,
        });
    }
    if age > 150 {
        return Err(ValidationError::AboveMaximum {
            field: "age".to_string(),
            max: 150,
        });
    }
    Ok(())
}

/// Validates a URL-safe slug: lowercase alphanumeric words separated by
/// single hyphens (e.g. `hello-world`). Uses the generic `InvalidFormat`
/// error since a slug has no dedicated error variant.
pub fn validate_slug(slug: &str, field: &str) -> Result<(), ValidationError> {
    if slug.is_empty() {
        return Err(ValidationError::Required {
            field: field.to_string(),
        });
    }
    if !SLUG_REGEX.is_match(slug) {
        return Err(ValidationError::InvalidFormat {
            field: field.to_string(),
        });
    }
    Ok(())
}

pub fn validate_required(value: &str, field: &str) -> Result<(), ValidationError> {
    if value.is_empty() {
        return Err(ValidationError::Required {
            field: field.to_string(),
        });
    }
    Ok(())
}

pub fn validate_min_length(value: &str, field: &str, min: usize) -> Result<(), ValidationError> {
    if value.len() < min {
        return Err(ValidationError::TooShort {
            field: field.to_string(),
            min,
        });
    }
    Ok(())
}

pub fn validate_max_length(value: &str, field: &str, max: usize) -> Result<(), ValidationError> {
    if value.len() > max {
        return Err(ValidationError::TooLong {
            field: field.to_string(),
            max,
        });
    }
    Ok(())
}
