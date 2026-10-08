use thiserror::Error;

/// Domain errors of the user module. Handlers map these onto HTTP responses;
/// the module itself never talks about status codes.
#[derive(Debug, Error, PartialEq)]
pub enum UserError {
    #[error("user with email {0} already exists")]
    EmailTaken(String),

    #[error("invalid user input: {0}")]
    InvalidInput(String),
}
