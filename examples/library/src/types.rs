use serde::{Deserialize, Serialize};

use crate::error::ValidationError;

/// A validated email address. The `try_from` attribute makes deserialization
/// run the same validation as `EmailAddress::new`, so an invalid value can
/// never be constructed — not even from untrusted input.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(try_from = "String")]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn new(email: &str) -> Result<Self, ValidationError> {
        crate::validator::validate_email(email)?;
        Ok(Self(email.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for EmailAddress {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        crate::validator::validate_email(&value)?;
        Ok(Self(value))
    }
}

impl std::fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A validated http(s) URL. Deserialization is validated, see [`EmailAddress`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(try_from = "String")]
pub struct Url(String);

impl Url {
    pub fn new(url: &str) -> Result<Self, ValidationError> {
        crate::validator::validate_url(url)?;
        Ok(Self(url.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Url {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        crate::validator::validate_url(&value)?;
        Ok(Self(value))
    }
}

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A validated age in the range 1..=150. Deserialization is validated via
/// `try_from` on the raw `u8`, matching the serialized form.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(try_from = "u8")]
pub struct Age(u8);

impl Age {
    pub fn new(age: u8) -> Result<Self, ValidationError> {
        crate::validator::validate_age(age)?;
        Ok(Self(age))
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for Age {
    type Error = ValidationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl std::fmt::Display for Age {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
