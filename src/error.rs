use crate::Code;
use serde::{Serialize, Serializer};
use std::fmt;
use std::time::Duration;

/// Structured error envelope for HTTP APIs.
#[derive(Debug, Clone)]
pub struct Error {
    pub code: Code,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub trace_id: Option<String>,
    pub retryable: bool,

    // Not serialized directly
    pub status: u16,
    pub retry_after: Option<Duration>,

    // Cause is not clonable, so we store it as a string
    cause_message: Option<String>,
}

impl Error {
    /// Creates a new error with the given code, status, and message.
    pub fn new(code: Code, status: u16, message: impl Into<String>) -> Self {
        let message = message.into();
        let message = if message.is_empty() {
            code.default_message().to_string()
        } else {
            message
        };

        let status = if status == 0 {
            code.default_status()
        } else {
            status
        };

        Self {
            code,
            message,
            details: None,
            trace_id: None,
            retryable: code.is_retryable_default(),
            status,
            retry_after: None,
            cause_message: None,
        }
    }

    /// Creates a new error with a formatted message.
    /// 
    /// This is a semantic alias for `new()` that signals the message
    /// is typically constructed with `format!()`.
    ///
    /// # Example
    /// ```
    /// use error_envelope::{Error, Code};
    /// let user_id = 123;
    /// let err = Error::newf(Code::NotFound, 404, format!("user {} not found", user_id));
    /// ```
    pub fn newf(code: Code, status: u16, message: impl Into<String>) -> Self {
        Self::new(code, status, message)
    }

    /// Creates a new error that wraps an underlying cause.
    pub fn wrap(
        code: Code,
        status: u16,
        message: impl Into<String>,
        cause: impl std::error::Error,
    ) -> Self {
        let mut err = Self::new(code, status, message);
        err.cause_message = Some(cause.to_string());
        err
    }

    /// Adds structured details to the error.
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Adds a trace ID for distributed tracing.
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Sets whether the error is retryable.
    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    /// Overrides the HTTP status code.
    pub fn with_status(mut self, status: u16) -> Self {
        if status != 0 {
            self.status = status;
        }
        self
    }

    /// Sets the retry-after duration for rate-limited responses.
    pub fn with_retry_after(mut self, duration: Duration) -> Self {
        self.retry_after = Some(duration);
        self
    }

    /// Returns the cause message if available.
    pub fn cause(&self) -> Option<&str> {
        self.cause_message.as_deref()
    }

    /// Returns the HTTP status code.
    pub fn status(&self) -> u16 {
        self.status
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref cause) = self.cause_message {
            write!(f, "{:?}: {} ({})", self.code, self.message, cause)
        } else {
            write!(f, "{:?}: {}", self.code, self.message)
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Since we only store the cause message, we can't return the original error
        None
    }
}

// Custom serialization to include retry_after as human-readable duration
impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        // Count actual fields that will be serialized
        let mut field_count = 3; // code, message, retryable (always present)
        if self.details.is_some() {
            field_count += 1;
        }
        if self.trace_id.is_some() {
            field_count += 1;
        }
        if self.retry_after.is_some() {
            field_count += 1;
        }

        let mut state = serializer.serialize_struct("Error", field_count)?;

        state.serialize_field("code", &self.code)?;
        state.serialize_field("message", &self.message)?;

        if self.details.is_some() {
            state.serialize_field("details", &self.details)?;
        }

        if self.trace_id.is_some() {
            state.serialize_field("trace_id", &self.trace_id)?;
        }

        state.serialize_field("retryable", &self.retryable)?;

        if let Some(ref duration) = self.retry_after {
            let secs = duration.as_secs();
            let formatted = if secs < 60 {
                format!("{}s", secs)
            } else {
                format!("{}m{}s", secs / 60, secs % 60)
            };
            state.serialize_field("retry_after", &formatted)?;
        }

        state.end()
    }
}
