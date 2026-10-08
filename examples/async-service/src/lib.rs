//! Library crate for the async service example.
//!
//! Exposes the building blocks so integration tests (and the binary in
//! `main.rs`) can drive workers, the scheduler, and the runtime directly.

pub mod cancellation;
pub mod runtime;
pub mod scheduler;
pub mod worker;
