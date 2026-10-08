use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ValidationError {
    #[error("Field '{field}' is required")]
    Required { field: String },

    #[error("Field '{field}' must be at least {min} characters")]
    TooShort { field: String, min: usize },

    #[error("Field '{field}' must be at most {max} characters")]
    TooLong { field: String, max: usize },

    #[error("Field '{field}' has invalid format")]
    InvalidFormat { field: String },

    #[error("Field '{field}' must be a valid email")]
    InvalidEmail { field: String },

    #[error("Field '{field}' must be a valid URL")]
    InvalidUrl { field: String },

    #[error("Field '{field}' must be at least {min}")]
    BelowMinimum { field: String, min: i64 },

    #[error("Field '{field}' must be at most {max}")]
    AboveMaximum { field: String, max: i64 },
}
