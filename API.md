# API Reference

Complete API documentation for error-envelope.

## Table of Contents

- [Common Constructors](#common-constructors)
- [Formatted Constructors](#formatted-constructors)
- [Custom Errors](#custom-errors)
- [Builder Pattern](#builder-pattern)

---

## Common Constructors

Pre-built constructors for standard HTTP error scenarios.

```rust
use error_envelope::Error;

// Generic errors
Error::internal("Database connection failed");   // 500
Error::bad_request("Invalid JSON in body");       // 400

// Auth errors
Error::unauthorized("Missing token");             // 401
Error::forbidden("Insufficient permissions");     // 403

// Resource errors
Error::not_found("User not found");                // 404
Error::method_not_allowed("POST not allowed");      // 405
Error::request_timeout("Client timeout");          // 408
Error::conflict("Email already exists");           // 409
Error::gone("Resource permanently deleted");      // 410
Error::payload_too_large("Upload exceeds 10MB");    // 413
Error::unprocessable_entity("Invalid data format"); // 422

// Infrastructure errors
Error::rate_limited("Too many requests");          // 429
Error::unavailable("Service temporarily down");   // 503
Error::timeout("Database query timed out");       // 504

// Downstream errors
Error::downstream("payments", err);               // 502
Error::downstream_timeout("payments", err);        // 504
```

---

## Formatted Constructors

Use the `format!` macro for dynamic error messages.

```rust
use error_envelope::{not_foundf, internalf, unauthorizedf, forbiddenf, 
                      conflictf, timeoutf, unavailablef};

// Dynamic messages with format! macro
let user_id = 123;
let err = not_foundf(format!("user {} not found", user_id));

let db_name = "postgres";
let err = internalf(format!("database {} connection failed", db_name));

let role = "admin";
let err = forbiddenf(format!("requires {} role", role));

// All common constructors have 'f' variants
let err = unauthorizedf(format!("token expired at {}", timestamp));
let err = conflictf(format!("email {} already registered", email));
let err = timeoutf(format!("query took {}ms (limit: 5000ms)", duration));
```

---

## Custom Errors

Low-level constructor for full control.

```rust
use error_envelope::{Error, Code};
use std::time::Duration;

// Low-level constructor
let err = Error::new(
    Code::Internal,
    500,
    "Database connection failed"
);

// Add structured details
let err = err.with_details(serde_json::json!({
    "database": "postgres",
    "host": "db.example.com",
    "connection_pool": "primary"
}));

// Add trace ID for distributed tracing
let err = err.with_trace_id("abc-123-def-456");

// Override default retryable behavior
let err = err.with_retryable(true);

// Override HTTP status code
let err = err.with_status(503);

// Set retry-after duration (for rate limiting)
let err = err.with_retry_after(Duration::from_secs(60));
```

### Helper Constructor: `newf`

For formatted messages without separate `*f` functions:

```rust
use error_envelope::{Error, Code};

let user_id = 123;
let err = Error::newf(
    Code::NotFound, 
    404, 
    format!("user {} not found", user_id)
);

// Equivalent to:
let err = not_foundf(format!("user {} not found", user_id));
```

---

## Builder Pattern

All `with_*` methods consume and return `Self`, enabling fluent chaining.

```rust
use error_envelope::Error;
use std::time::Duration;

// Chain multiple modifiers
let err = Error::rate_limited("too many requests")
    .with_details(serde_json::json!({
        "limit": 100,
        "window": "1m",
        "reset_at": "2025-12-25T12:00:00Z"
    }))
    .with_trace_id("trace-123")
    .with_retry_after(Duration::from_secs(30));

// Example output:
// {
//   "code": "RATE_LIMITED",
//   "message": "too many requests",
//   "details": {
//     "limit": 100,
//     "window": "1m",
//     "reset_at": "2025-12-25T12:00:00Z"
//   },
//   "trace_id": "trace-123",
//   "retryable": true,
//   "retry_after": "30s"
// }
```

### Available Builder Methods

| Method | Purpose | Example |
|--------|---------|---------|
| `with_details(value)` | Add structured context | `.with_details(json!({"user_id": "123"}))` |
| `with_trace_id(id)` | Add trace ID for debugging | `.with_trace_id(request_id)` |
| `with_retryable(bool)` | Override retry behavior | `.with_retryable(true)` |
| `with_status(u16)` | Override HTTP status | `.with_status(503)` |
| `with_retry_after(Duration)` | Set retry duration | `.with_retry_after(Duration::from_secs(30))` |

### Immutability

The builder pattern is **immutable by default** in Rust:

```rust
let err1 = Error::internal("base error");
let err2 = err1.with_trace_id("abc-123");

// err1 and err2 are different instances
// err1 does NOT have trace_id (it was moved into err2)
```

This is unlike the Go version (`err-envelope`), which had to implement copy-on-modify explicitly.

---

## Helper Functions

### Validation Errors

```rust
use error_envelope::{validation, FieldErrors};
use std::collections::HashMap;

let mut fields = HashMap::new();
fields.insert("email".to_string(), "Invalid email format".to_string());
fields.insert("age".to_string(), "Must be between 18 and 120".to_string());

let err = validation(fields);

// Response:
// {
//   "code": "VALIDATION_FAILED",
//   "message": "Invalid input",
//   "details": {
//     "fields": {
//       "email": "Invalid email format",
//       "age": "Must be between 18 and 120"
//     }
//   },
//   "retryable": false
// }
```

### Error Mapping

```rust
use error_envelope::from;

// Map arbitrary errors to Error
fn process() -> Result<Data, error_envelope::Error> {
    let result = external_call()
        .map_err(|e| from(e))?;  // Automatically detects timeout/cancel patterns
    Ok(result)
}

// The from() helper inspects error messages and maps to appropriate codes:
// - "timeout" or "timed out" → Timeout (504, retryable)
// - "cancel" → Canceled (499, not retryable)
// - Everything else → Internal (500, not retryable)
```

### Error Code Checking

```rust
use error_envelope::{is, Code};

let err = Error::not_found("User not found");

if is(&err, Code::NotFound) {
    // Handle not found specifically
}
```

---

## Framework Integration

### Axum (axum-support feature)

```rust
use axum::Json;
use error_envelope::Error;

async fn handler() -> Result<Json<User>, Error> {
    let user = find_user("123").await?;
    Ok(Json(user))
}

// Error automatically converts to HTTP response with:
// - Correct status code
// - JSON body with error envelope
// - X-Request-ID header (if trace_id set)
// - Retry-After header (if retry_after set)
```

### anyhow Integration (anyhow-support feature)

```rust
use error_envelope::Error;
use anyhow::Result;

async fn handler() -> Result<Json<Data>, Error> {
    // anyhow::Error converts automatically via From trait
    let config = load_config().await?;
    let data = fetch_data(&config).await?;
    Ok(Json(data))
}

// Any anyhow::Error becomes error_envelope::Error (Internal/500)
// with the error message preserved
```

---

## Advanced Usage

### Wrapping Errors with Context

```rust
use error_envelope::Error;

// Wrap an underlying error with context
let cause = std::io::Error::new(std::io::ErrorKind::NotFound, "config.toml");
let err = Error::wrap(
    Code::Internal,
    500,
    "Failed to load configuration",
    cause
);

// The cause message is stored internally and included in Display output
println!("{}", err);
// Output: Internal: Failed to load configuration (config.toml)

// Access cause via method
if let Some(cause_msg) = err.cause() {
    println!("Root cause: {}", cause_msg);
}
```

### Rate Limiting with Retry-After

```rust
use error_envelope::Error;
use std::time::Duration;

fn check_rate_limit(user_id: &str) -> Result<(), Error> {
    if exceeded_limit(user_id) {
        return Err(
            Error::rate_limited("Too many requests")
                .with_retry_after(Duration::from_secs(30))
                .with_details(serde_json::json!({
                    "limit": 100,
                    "window": "1m"
                }))
        );
    }
    Ok(())
}

// Response includes Retry-After header (with axum-support)
// and retry_after field in JSON body ("30s")
```

### Distributed Tracing

```rust
use axum::extract::Request;
use error_envelope::Error;

async fn handler(req: Request) -> Result<Json<Data>, Error> {
    // Extract trace ID from incoming request
    let trace_id = req.headers()
        .get("X-Request-ID")
        .and_then(|h| h.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Use trace ID in all errors
    let data = fetch_data().await
        .map_err(|e| Error::from(e).with_trace_id(&trace_id))?;
    
    Ok(Json(data))
}

// All errors in the call chain will include the same trace_id
// for correlation across services
```

---

## Domain Error Mapping (thiserror)

When using `thiserror` for domain errors, implement `From<DomainError> for Error` to map domain semantics to HTTP responses.

This gives you:
- Explicit HTTP status codes and error codes
- Zero boilerplate in handlers via `?`
- No accidental 500s for domain failures

### Pattern

```rust
use error_envelope::{Code, Error};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum DomainError {
    #[error("user not found")]
    NotFound,

    #[error("email already exists")]
    EmailConflict,

    #[error("database error")]
    Database(#[from] anyhow::Error),
}

impl From<DomainError> for Error {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::NotFound =>
                Error::new(Code::NotFound, 404, "User not found"),

            DomainError::EmailConflict =>
                Error::new(Code::Conflict, 409, "Email already exists"),

            // Preserve cause message for debugging
            DomainError::Database(cause) =>
                Error::wrap(Code::Internal, 500, "Database failure", cause),
        }
    }
}
```

### Usage in Handlers

```rust
use axum::Json;
use error_envelope::Error;

#[derive(Debug)]
struct User;

async fn handler() -> Result<Json<User>, Error> {
    // DomainError -> Error automatically via From
    let user = get_user().await?;
    Ok(Json(user))
}

async fn get_user() -> Result<User, DomainError> {
    Err(DomainError::NotFound)
}
```

### Notes

- `Error::wrap(...)` stores the cause's string representation for debugging
- This is **not** an error chain — `Error::source()` returns `None`
- Use this pattern for **domain errors** where you know the HTTP semantics
- Use anyhow integration for **unknown/unexpected errors** at boundaries

---

For complete error code reference, see [ERROR_CODES.md](ERROR_CODES.md).
