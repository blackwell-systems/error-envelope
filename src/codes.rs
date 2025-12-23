use serde::{Deserialize, Serialize};

/// Machine-readable error codes that remain stable across releases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Code {
    // Generic errors
    Internal,
    BadRequest,
    NotFound,
    MethodNotAllowed,
    Gone,
    Conflict,
    PayloadTooLarge,
    RequestTimeout,
    RateLimited,
    Unavailable,

    // Validation / auth
    ValidationFailed,
    Unauthorized,
    Forbidden,
    UnprocessableEntity,

    // Timeouts / cancellations
    Timeout,
    Canceled,

    // Downstream
    DownstreamError,
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
