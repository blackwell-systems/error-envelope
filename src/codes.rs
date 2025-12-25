use serde::{Deserialize, Serialize};

/// Machine-readable error codes that remain stable across releases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Code {
    /// Internal server error (500).
    Internal,
    /// Bad request error (400).
    BadRequest,
    /// Resource not found (404).
    NotFound,
    /// HTTP method not allowed (405).
    MethodNotAllowed,
    /// Resource permanently deleted (410).
    Gone,
    /// Resource conflict, often due to concurrent modification (409).
    Conflict,
    /// Request payload exceeds size limit (413).
    PayloadTooLarge,
    /// Request timed out before completion (408).
    RequestTimeout,
    /// Too many requests, client should retry later (429).
    RateLimited,
    /// Service temporarily unavailable (503).
    Unavailable,

    /// Input validation failed (400).
    ValidationFailed,
    /// Authentication required or credentials invalid (401).
    Unauthorized,
    /// Authenticated but not permitted to access resource (403).
    Forbidden,
    /// Request is well-formed but semantically invalid (422).
    UnprocessableEntity,

    /// Gateway or processing timeout (504).
    Timeout,
    /// Request was canceled by client (499).
    Canceled,

    /// Downstream service returned an error (502).
    DownstreamError,
    /// Downstream service timed out (504).
    DownstreamTimeout,
}

impl Code {
    /// Returns the default HTTP status code for this error code.
    pub fn default_status(&self) -> u16 {
        match self {
            Code::Internal => 500,
            Code::BadRequest => 400,
            Code::NotFound => 404,
            Code::MethodNotAllowed => 405,
            Code::Gone => 410,
            Code::Conflict => 409,
            Code::PayloadTooLarge => 413,
            Code::RequestTimeout => 408,
            Code::RateLimited => 429,
            Code::Unavailable => 503,
            Code::ValidationFailed => 400,
            Code::Unauthorized => 401,
            Code::Forbidden => 403,
            Code::UnprocessableEntity => 422,
            Code::Timeout => 504,
            Code::Canceled => 499,
            Code::DownstreamError => 502,
            Code::DownstreamTimeout => 504,
        }
    }

    /// Returns whether this error is retryable by default.
    pub fn is_retryable_default(&self) -> bool {
        matches!(
            self,
            Code::Timeout
                | Code::DownstreamTimeout
                | Code::Unavailable
                | Code::RateLimited
                | Code::RequestTimeout
        )
    }

    /// Returns a default human-readable message for this code.
    pub fn default_message(&self) -> &'static str {
        match self {
            Code::Internal => "Internal error",
            Code::BadRequest => "Bad request",
            Code::ValidationFailed => "Invalid input",
            Code::Unauthorized => "Unauthorized",
            Code::Forbidden => "Forbidden",
            Code::NotFound => "Not found",
            Code::Gone => "Resource no longer exists",
            Code::Conflict => "Conflict",
            Code::PayloadTooLarge => "Payload too large",
            Code::UnprocessableEntity => "Unprocessable entity",
            Code::RateLimited => "Rate limited",
            Code::RequestTimeout | Code::Timeout | Code::DownstreamTimeout => "Request timed out",
            Code::Unavailable => "Service unavailable",
            Code::Canceled => "Request canceled",
            Code::DownstreamError => "Downstream service error",
            Code::MethodNotAllowed => "Method not allowed",
        }
    }
}
