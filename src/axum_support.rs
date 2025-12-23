//! Axum framework integration for error-envelope.
//!
//! Enable this module with the `axum-support` feature.
//!
//! # Example
//!
//! ```rust,no_run
//! use axum::{routing::get, Router};
//! use error_envelope::Error;
//!
//! async fn handler() -> Result<String, Error> {
//!     Err(Error::not_found("User not found")
//!         .with_trace_id("abc-123"))
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = Router::new().route("/", get(handler));
//!     let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
//!         .await
//!         .unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```

use crate::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        // Clone fields we need before moving self into JSON
        let retry_after = self.retry_after;
        let trace_id = self.trace_id.clone();

        // Create base response with JSON body
        let mut response = (status, Json(self)).into_response();

        // Add Retry-After header if specified
        if let Some(duration) = retry_after {
            let seconds = duration.as_secs().max(1);
            response
                .headers_mut()
                .insert("Retry-After", seconds.to_string().parse().unwrap());
        }

        // Add X-Request-Id header if trace ID is present
        if let Some(trace_id) = trace_id {
            if let Ok(header_value) = trace_id.parse() {
                response.headers_mut().insert("X-Request-Id", header_value);
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Code;
    use std::time::Duration;

    #[tokio::test]
    async fn test_into_response() {
        let err = Error::new(Code::NotFound, 404, "user not found");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_with_retry_after() {
        let err =
            Error::rate_limited("too many requests").with_retry_after(Duration::from_secs(30));
        let response = err.into_response();

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(response.headers().contains_key("Retry-After"));
    }

    #[tokio::test]
    async fn test_with_trace_id() {
        let err = Error::not_found("user not found").with_trace_id("abc-123");
        let response = err.into_response();

        assert!(response.headers().contains_key("X-Request-Id"));
    }
}
