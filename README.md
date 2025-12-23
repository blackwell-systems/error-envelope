# error-envelope

[![Blackwell Systems™](https://raw.githubusercontent.com/blackwell-systems/blackwell-docs-theme/main/badge-trademark.svg)](https://github.com/blackwell-systems)
[![Crates.io](https://img.shields.io/crates/v/error-envelope.svg)](https://crates.io/crates/error-envelope)
[![Docs.rs](https://docs.rs/error-envelope/badge.svg)](https://docs.rs/error-envelope)
[![CI](https://github.com/blackwell-systems/error-envelope/actions/workflows/ci.yml/badge.svg)](https://github.com/blackwell-systems/error-envelope/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Sponsor](https://img.shields.io/badge/Sponsor-Buy%20Me%20a%20Coffee-yellow?logo=buy-me-a-coffee&logoColor=white)](https://buymeacoffee.com/blackwellsystems)

A tiny Rust crate for consistent HTTP error responses across services.

This is a Rust port of [`err-envelope` (Go)](https://github.com/blackwell-systems/err-envelope), providing feature parity with the Go implementation.

## Why

Without a standard, every endpoint returns errors differently:
- `{"error": "bad request"}`
- `{"message": "invalid email"}`  
- `{"code": "E123", "details": {...}}`

This forces clients to handle each endpoint specially. `error-envelope` provides a single, predictable error shape.

## What You Get

```json
{
  "code": "VALIDATION_FAILED",
  "message": "Invalid input",
  "details": {
    "fields": {
      "email": "must be a valid email"
    }
  },
  "trace_id": "a1b2c3d4e5f6",
  "retryable": false
}
```

Every field has a purpose: stable codes for logic, messages for humans, details for context, trace IDs for debugging, and retry signals for resilience.

**Rate limiting example:**
```json
{
  "code": "RATE_LIMITED",
  "message": "Too many requests",
  "trace_id": "a1b2c3d4e5f6",
  "retryable": true,
  "retry_after": "30s"
}
```

The `retry_after` field (human-readable duration) appears when `with_retry_after()` is used.

## Installation

```toml
[dependencies]
error-envelope = "0.1"
```

For Axum integration:
```toml
[dependencies]
error-envelope = { version = "0.1", features = ["axum-support"] }
```

📖 **Full API documentation**: [docs.rs/error-envelope](https://docs.rs/error-envelope)

## Crate Features

- **`default`**: Core error envelope with no framework dependencies
- **`axum-support`**: Adds `IntoResponse` implementation for Axum framework integration

## Quick Start

```rust
use error_envelope::Error;

fn main() {
    let err = Error::not_found("User not found")
        .with_details(serde_json::json!({"user_id": "123"}))
        .with_trace_id("abc-123");

    println!("{}", serde_json::to_string_pretty(&err).unwrap());
}
```

## API

### Common Constructors

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

### Formatted Constructors

Use the `format!` macro for dynamic error messages:

```rust
use error_envelope::{not_foundf, internalf};

// Using format! macro
let user_id = 123;
let err = not_foundf(format!("user {} not found", user_id));

let db_name = "postgres";
let err = internalf(format!("database {} connection failed", db_name));
```

### Custom Errors

```rust
use error_envelope::{Error, Code};
use std::time::Duration;

// Low-level constructor
let err = Error::new(
    Code::Internal,
    500,
    "Database connection failed"
);

// Add details
let err = err.with_details(serde_json::json!({
    "database": "postgres",
    "host": "db.example.com"
}));

// Add trace ID
let err = err.with_trace_id("abc123");

// Override retryable
let err = err.with_retryable(true);

// Set retry-after duration
let err = err.with_retry_after(Duration::from_secs(60));
```

### Builder Pattern

All `with_*` methods consume and return `Self`, enabling fluent chaining:

```rust
let err = Error::rate_limited("too many requests")
    .with_details(serde_json::json!({"limit": 100}))
    .with_trace_id("trace-123")
    .with_retry_after(Duration::from_secs(30));
```

The builder pattern is **immutable by default** in Rust (unlike the Go version which had to implement copy-on-modify).

## Error Codes

| Code | HTTP Status | Retryable | Use Case |
|------|-------------|-----------|----------|
| `Internal` | 500 | No | Unexpected server errors |
| `BadRequest` | 400 | No | Malformed requests |
| `ValidationFailed` | 400 | No | Invalid input data |
| `Unauthorized` | 401 | No | Missing/invalid auth |
| `Forbidden` | 403 | No | Insufficient permissions |
| `NotFound` | 404 | No | Resource doesn't exist |
| `MethodNotAllowed` | 405 | No | Invalid HTTP method |
| `RequestTimeout` | 408 | Yes | Client timeout |
| `Conflict` | 409 | No | State conflict (duplicate) |
| `Gone` | 410 | No | Resource permanently deleted |
| `PayloadTooLarge` | 413 | No | Request body too large |
| `UnprocessableEntity` | 422 | No | Semantic validation failed |
| `RateLimited` | 429 | Yes | Too many requests |
| `Canceled` | 499 | No | Client canceled request |
| `Unavailable` | 503 | Yes | Service temporarily down |
| `Timeout` | 504 | Yes | Gateway timeout |
| `DownstreamError` | 502 | Yes | Upstream service failed |
| `DownstreamTimeout` | 504 | Yes | Upstream service timeout |

## Framework Integration

### Axum (Optional Feature)

Enable the `axum-support` feature and implement `IntoResponse`:

```toml
[dependencies]
error-envelope = { version = "0.1", features = ["axum-support"] }
```

```rust
use axum::response::IntoResponse;
use error_envelope::Error;

async fn handler() -> Result<String, Error> {
    Err(Error::not_found("User not found"))
}
```

## Design Principles

**Minimal**: ~500 lines, minimal dependencies, single responsibility.

**Framework-Agnostic**: Works standalone, optional integrations available.

**Predictable**: Error codes are stable (never change). Messages may evolve for clarity.

**Observable**: Trace IDs for request correlation. Structured details for logging.

## Examples

See [`examples/axum_server.rs`](examples/axum_server.rs) for a complete Axum server demonstrating:
- Validation errors with field details
- Rate limiting with retry-after
- Downstream error handling
- Trace ID propagation

Run it:
```bash
cargo run --example axum_server --features axum-support
```

Test endpoints:
```bash
curl http://localhost:3000/user?id=123
curl http://localhost:3000/rate-limit
curl http://localhost:3000/validation
```

## Testing

```bash
cargo test --all-features
```

All 17 tests pass (15 unit tests + 2 doc tests):
- Constructors and builders
- JSON serialization  
- Error trait implementation
- Retry-after formatting
- Immutability guarantees
- Axum integration

## License

MIT
