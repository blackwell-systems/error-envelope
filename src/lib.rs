//! A tiny, framework-agnostic error envelope for HTTP APIs.
//!
//! This crate provides a structured error envelope that standardizes
//! error responses across HTTP services with fields for:
//! - Stable machine-readable codes
//! - Human-readable messages
//! - Structured details
//! - Trace IDs for debugging
//! - Retry signals for resilience
//!
//! # Example
//!
//! ```rust
//! use error_envelope::{Error, Code};
//!
//! let err = Error::not_found("User not found")
//!     .with_details(serde_json::json!({"user_id": "123"}))
//!     .with_trace_id("abc-def-123");
//!
//! assert_eq!(err.code, Code::NotFound);
//! assert_eq!(err.status, 404);
//! ```

mod codes;
mod error;
mod helpers;
mod tests;

pub use codes::Code;
pub use error::Error;
pub use helpers::*;

#[cfg(feature = "axum-support")]
pub mod axum_support;

#[cfg(feature = "anyhow-support")]
mod anyhow_support;
