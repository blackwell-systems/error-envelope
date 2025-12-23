#[cfg(test)]
mod tests {
    use crate::{Code, Error};
    use std::time::Duration;

    #[test]
    fn test_new() {
        let err = Error::new(Code::NotFound, 404, "user not found");
        assert_eq!(err.code, Code::NotFound);
        assert_eq!(err.status, 404);
        assert_eq!(err.message, "user not found");
        assert!(!err.retryable);
    }

    #[test]
    fn test_builder_pattern() {
        let err = Error::not_found("user not found")
            .with_details(serde_json::json!({"user_id": "123"}))
            .with_trace_id("abc-123")
            .with_retryable(true);

        assert_eq!(err.code, Code::NotFound);
        assert_eq!(err.trace_id, Some("abc-123".to_string()));
        assert!(err.retryable);
        assert!(err.details.is_some());
    }

    #[test]
    fn test_helpers() {
        let err = Error::internal("database error");
        assert_eq!(err.code, Code::Internal);
        assert_eq!(err.status, 500);

        let err = Error::bad_request("invalid json");
        assert_eq!(err.code, Code::BadRequest);
        assert_eq!(err.status, 400);

        let err = Error::unauthorized("missing token");
        assert_eq!(err.code, Code::Unauthorized);
        assert_eq!(err.status, 401);
    }

    #[test]
    fn test_json_serialization() {
        let err = Error::not_found("user not found")
            .with_trace_id("abc-123");

        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"code\":\"NOT_FOUND\""));
        assert!(json.contains("\"message\":\"user not found\""));
        assert!(json.contains("\"trace_id\":\"abc-123\""));
        assert!(json.contains("\"retryable\":false"));
    }

    #[test]
    fn test_retry_after_serialization() {
        let err = Error::rate_limited("too many requests")
            .with_retry_after(Duration::from_secs(30));

        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"retry_after\":\"30s\""));

        let err = Error::unavailable("maintenance")
            .with_retry_after(Duration::from_secs(300));

        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"retry_after\":\"5m0s\""));
    }

    #[test]
    fn test_wrap_with_cause() {
        let cause = std::io::Error::new(std::io::ErrorKind::Other, "connection refused");
        let err = Error::wrap(Code::Internal, 500, "database connection failed", cause);

        assert_eq!(err.code, Code::Internal);
        assert!(err.cause().is_some());
        assert!(err.cause().unwrap().contains("connection refused"));
    }

    #[test]
    fn test_downstream_errors() {
        let cause = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
        let err = Error::downstream("payments", cause);

        assert_eq!(err.code, Code::DownstreamError);
        assert_eq!(err.status, 502);
        assert!(err.retryable);
        assert!(err.details.is_some());
    }

    #[test]
    fn test_display() {
        let err = Error::not_found("user not found");
        let display = format!("{}", err);
        assert!(display.contains("NotFound"));
        assert!(display.contains("user not found"));
    }

    #[test]
    fn test_error_trait() {
        let err = Error::internal("test error");
        let _: &dyn std::error::Error = &err;  // Should compile
    }

    #[test]
    fn test_formatted_helpers() {
        let user_id = 123;
        let err = crate::not_foundf(format!("user {} not found", user_id));
        assert_eq!(err.code, Code::NotFound);
        assert_eq!(err.message, "user 123 not found");
    }

    #[test]
    fn test_default_message() {
        let err = Error::new(Code::Internal, 500, "");
        assert_eq!(err.message, "Internal error");
    }

    #[test]
    fn test_immutability() {
        let original = Error::not_found("not found");
        let modified = original.clone()
            .with_details(serde_json::json!({"id": "123"}))
            .with_trace_id("trace-456")
            .with_retryable(true);

        // Original should be unchanged
        assert!(original.details.is_none());
        assert!(original.trace_id.is_none());
        assert!(!original.retryable);

        // Modified should have new values
        assert!(modified.details.is_some());
        assert_eq!(modified.trace_id, Some("trace-456".to_string()));
        assert!(modified.retryable);
    }
}
