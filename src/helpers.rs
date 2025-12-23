use crate::{Code, Error};

/// Helper constructors for common error types.
impl Error {
    // Generic errors

    /// Creates an internal server error (500).
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(Code::Internal, 500, message).with_retryable(false)
    }

    /// Creates a bad request error (400).
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(Code::BadRequest, 400, message).with_retryable(false)
    }

    /// Creates a validation error (400).
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(Code::ValidationFailed, 400, message).with_retryable(false)
    }

    /// Creates an unauthorized error (401).
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(Code::Unauthorized, 401, message).with_retryable(false)
    }

    /// Creates a forbidden error (403).
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(Code::Forbidden, 403, message).with_retryable(false)
    }

    /// Creates a not found error (404).
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(Code::NotFound, 404, message).with_retryable(false)
    }

    /// Creates a method not allowed error (405).
    pub fn method_not_allowed(message: impl Into<String>) -> Self {
        Self::new(Code::MethodNotAllowed, 405, message).with_retryable(false)
    }

    /// Creates a request timeout error (408).
    pub fn request_timeout(message: impl Into<String>) -> Self {
        Self::new(Code::RequestTimeout, 408, message).with_retryable(true)
    }

    /// Creates a conflict error (409).
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(Code::Conflict, 409, message).with_retryable(false)
    }

    /// Creates a gone error (410).
    pub fn gone(message: impl Into<String>) -> Self {
        Self::new(Code::Gone, 410, message).with_retryable(false)
    }

    /// Creates a payload too large error (413).
    pub fn payload_too_large(message: impl Into<String>) -> Self {
        Self::new(Code::PayloadTooLarge, 413, message).with_retryable(false)
    }

    /// Creates an unprocessable entity error (422).
    pub fn unprocessable_entity(message: impl Into<String>) -> Self {
        Self::new(Code::UnprocessableEntity, 422, message).with_retryable(false)
    }

    /// Creates a rate limited error (429).
    pub fn rate_limited(message: impl Into<String>) -> Self {
        Self::new(Code::RateLimited, 429, message).with_retryable(true)
    }

    /// Creates a timeout error (504).
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(Code::Timeout, 504, message).with_retryable(true)
    }

    /// Creates an unavailable error (503).
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new(Code::Unavailable, 503, message).with_retryable(true)
    }

    /// Creates a downstream error (502).
    pub fn downstream(service: impl Into<String>, cause: impl std::error::Error) -> Self {
        let service = service.into();
        let mut err = Self::wrap(Code::DownstreamError, 502, "", cause);
        if !service.is_empty() {
            err = err.with_details(serde_json::json!({"service": service}));
        }
        err.with_retryable(true)
    }

    /// Creates a downstream timeout error (504).
    pub fn downstream_timeout(service: impl Into<String>, cause: impl std::error::Error) -> Self {
        let service = service.into();
        let mut err = Self::wrap(Code::DownstreamTimeout, 504, "", cause);
        if !service.is_empty() {
            err = err.with_details(serde_json::json!({"service": service}));
        }
        err.with_retryable(true)
    }
}

// Formatted constructors (using format! macro)

/// Creates an internal server error with formatted message.
pub fn internalf(message: impl Into<String>) -> Error {
    Error::internal(message)
}

/// Creates a bad request error with formatted message.
pub fn bad_requestf(message: impl Into<String>) -> Error {
    Error::bad_request(message)
}

/// Creates a not found error with formatted message.
pub fn not_foundf(message: impl Into<String>) -> Error {
    Error::not_found(message)
}

/// Creates an unauthorized error with formatted message.
pub fn unauthorizedf(message: impl Into<String>) -> Error {
    Error::unauthorized(message)
}

/// Creates a forbidden error with formatted message.
pub fn forbiddenf(message: impl Into<String>) -> Error {
    Error::forbidden(message)
}

/// Creates a conflict error with formatted message.
pub fn conflictf(message: impl Into<String>) -> Error {
    Error::conflict(message)
}

/// Creates a timeout error with formatted message.
pub fn timeoutf(message: impl Into<String>) -> Error {
    Error::timeout(message)
}

/// Creates an unavailable error with formatted message.
pub fn unavailablef(message: impl Into<String>) -> Error {
    Error::unavailable(message)
}

// Additional helpers

use serde_json::json;
use std::collections::HashMap;

/// Field-level validation errors.
pub type FieldErrors = HashMap<String, String>;

/// Creates a validation error with field-level details.
pub fn validation(fields: FieldErrors) -> Error {
    Error::new(Code::ValidationFailed, 400, "")
        .with_details(json!({"fields": fields}))
        .with_retryable(false)
}

/// Maps arbitrary errors into an Error.
///
/// Handles common error types and wraps unknown errors as Internal.
pub fn from(err: impl std::error::Error + 'static) -> Error {
    let err_str = err.to_string().to_lowercase();

    // Check for timeout errors
    if err_str.contains("timeout") || err_str.contains("timed out") {
        return Error::timeout("");
    }

    // Check for canceled errors
    if err_str.contains("cancel") {
        return Error::new(Code::Canceled, 499, "").with_retryable(false);
    }

    // Default to wrapping as internal error
    Error::wrap(Code::Internal, 500, "", err).with_retryable(false)
}

/// Checks if an error has the given code.
pub fn is(err: &Error, code: Code) -> bool {
    err.code == code
}
