//! Shared infrastructure. `common` must never depend on `modules` —
//! adding a feature module must not require editing anything here.

pub mod config;
pub mod error;
