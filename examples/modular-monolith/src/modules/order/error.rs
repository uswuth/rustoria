use thiserror::Error;
use uuid::Uuid;

/// Domain errors of the order module. Handlers map these onto HTTP
/// responses; the module itself never talks about status codes.
#[derive(Debug, Error, PartialEq)]
pub enum OrderError {
    #[error("order must contain at least one item")]
    NoItems,

    #[error("item {0} must have a quantity of at least 1")]
    ZeroQuantity(Uuid),

    #[error("user {0} does not exist")]
    UnknownUser(Uuid),
}
