# error-envelope

[![Blackwell Systems™](https://raw.githubusercontent.com/blackwell-systems/blackwell-docs-theme/main/badge-trademark.svg)](https://github.com/blackwell-systems)
[![Crates.io](https://img.shields.io/crates/v/error-envelope.svg)](https://crates.io/crates/error-envelope)
[![Docs.rs](https://docs.rs/error-envelope/badge.svg)](https://docs.rs/error-envelope)
[![CI](https://github.com/blackwell-systems/error-envelope/actions/workflows/ci.yml/badge.svg)](https://github.com/blackwell-systems/error-envelope/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Sponsor](https://img.shields.io/badge/Sponsor-Buy%20Me%20a%20Coffee-yellow?logo=buy-me-a-coffee&logoColor=white)](https://buymeacoffee.com/blackwellsystems)

Structured, traceable, retry-aware HTTP error responses for Rust APIs. Features anyhow and Axum integration with a framework-agnostic core.

## Overview

- **anyhow integration**: Automatic conversion from anyhow::Error to structured HTTP responses
- **Axum support**: Implements IntoResponse for seamless API error handling
- **Consistent error format**: One predictable JSON structure for all HTTP errors
- **Typed error codes**: 18 standard codes as a type-safe enum
- **Traceability**: Built-in support for trace IDs and retry hints
- **Framework-agnostic core**: Works standalone; integrations are opt-in via features

**The stack:** anyhow for error propagation → error-envelope for HTTP boundary → Axum for responses

```rust
use axum::{extract::Path, Json};
use error_envelope::Error;

async fn get_user(Path(id): Path<String>) -> Result<Json<User>, Error> {
    let user = db::find_user(&id).await?; // anyhow error converts automatically
    Ok(Json(user))
}

// On error, returns structured HTTP response:
// {
//   "code": "TIMEOUT",
//   "message": "database query timeout",
//   "trace_id": "abc-123",
//   "retryable": true
// }
```

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [API Reference](#api-reference) - Common constructors and patterns
- [Error Codes](#error-codes) - Standard error codes reference

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
error-envelope = "0.2"
```

With optional features:
```toml
[dependencies]
error-envelope = { version = "0.2", features = ["axum-support", "anyhow-support"] }
```

You can enable either or both features depending on your use case.

📖 **Full API documentation**: [docs.rs/error-envelope](https://docs.rs/error-envelope)

## Crate Features

| Feature | Description |
|---------|-------------|
| `default` | Core error envelope with no framework dependencies |
| `axum-support` | Adds `IntoResponse` implementation for Axum framework integration |
| `anyhow-support` | Enables `From<anyhow::Error>` conversion for seamless interop with anyhow |

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

## Anyhow Integration

With the `anyhow-support` feature, `anyhow::Error` automatically converts to `error_envelope::Error`:

```rust
use error_envelope::Error;

async fn handler() -> Result<String, Error> {
    // anyhow::Error converts automatically via ?
    let result = do_work().await?;
    Ok(result)
}

fn do_work() -> anyhow::Result<String> {
    anyhow::bail!("something went wrong");
}
```

This makes error-envelope a drop-in replacement for anyhow at HTTP boundaries:

```rust
use axum::{Json, Router, routing::get};
use error_envelope::Error;

async fn api_handler() -> Result<Json<Response>, Error> {
    let data = fetch_data().await?; // anyhow error converts automatically
    Ok(Json(Response { data }))
}
```

## Framework Integration

### Axum

With the `axum-support` feature, `Error` implements `IntoResponse`:

```rust
use axum::{Json, routing::get, Router};
use error_envelope::Error;

async fn handler() -> Result<Json<User>, Error> {
    let user = db::find_user("123").await?;
    Ok(Json(user))
}

// Error automatically converts to HTTP response with:
// - Correct status code
// - JSON body with error envelope
// - X-Request-ID header (if trace_id set)
// - Retry-After header (if retry_after set)
```

## API Reference

Common constructors for typical scenarios:

```rust
use error_envelope::Error;

// Most common
Error::internal("Database connection failed");      // 500
Error::not_found("User not found");                 // 404
Error::unauthorized("Missing token");               // 401
Error::forbidden("Insufficient permissions");       // 403

// Validation
Error::bad_request("Invalid JSON");                 // 400
use error_envelope::validation;
let err = validation(field_errors);                 // 400 with field details

// Infrastructure
Error::rate_limited("Too many requests");           // 429
Error::timeout("Query timeout");                    // 504
```

**Builder pattern:**
```rust
let err = Error::rate_limited("too many requests")
    .with_details(serde_json::json!({"limit": 100}))
    .with_trace_id("trace-123")
    .with_retry_after(Duration::from_secs(30));
```

📖 **Full API documentation:** [API.md](API.md) - Complete constructor reference, formatted helpers, advanced patterns

## Error Codes

18 standard codes as a type-safe enum. Most common:

| Code | HTTP Status | Use Case |
|------|-------------|----------|
| `Internal` | 500 | Unexpected server errors |
| `NotFound` | 404 | Resource doesn't exist |
| `Unauthorized` | 401 | Missing/invalid auth |
| `ValidationFailed` | 400 | Invalid input data |
| `Timeout` | 504 | Gateway timeout (retryable) |

📖 **Complete reference:** [ERROR_CODES.md](ERROR_CODES.md) - All 18 codes with detailed descriptions, use cases, and retryable behavior


## Design Principles

Minimal, framework-agnostic core (~500 lines); integrations behind feature flags. See [ARCHITECTURE.md](ARCHITECTURE.md) for design rationale.

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

## License

MIT
