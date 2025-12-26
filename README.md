# error-envelope

[![Blackwell Systems™](https://raw.githubusercontent.com/blackwell-systems/blackwell-docs-theme/main/badge-trademark.svg)](https://github.com/blackwell-systems)
[![Crates.io](https://img.shields.io/crates/v/error-envelope.svg)](https://crates.io/crates/error-envelope)
[![Docs.rs](https://docs.rs/error-envelope/badge.svg)](https://docs.rs/error-envelope)
[![CI](https://github.com/blackwell-systems/error-envelope/actions/workflows/ci.yml/badge.svg)](https://github.com/blackwell-systems/error-envelope/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Sponsor](https://img.shields.io/badge/Sponsor-Buy%20Me%20a%20Coffee-yellow?logo=buy-me-a-coffee&logoColor=white)](https://buymeacoffee.com/blackwellsystems)

Structured, traceable, retry-aware HTTP error responses for Rust APIs. Features anyhow and Axum integration with a framework-agnostic core.

## Overview

- **anyhow integration**: Automatic conversion from anyhow::Error into error_envelope::Error at the HTTP boundary
- **Axum support**: Implements IntoResponse for seamless API error handling
- **Consistent error format**: One predictable JSON structure for all HTTP errors
- **Typed error codes**: 18 standard codes as a type-safe enum
- **Traceability**: Built-in support for trace IDs and retry hints
- **Framework-agnostic core**: Works standalone; integrations are opt-in via features

**The stack:** anyhow for propagation → error-envelope at the HTTP boundary → Axum integration (optional via feature flag)

```rust
use axum::{extract::Path, Json};
use error_envelope::{Error, validation};
use std::collections::HashMap;

#[derive(serde::Deserialize)]
struct CreateUser { email: String, age: u8 }

#[derive(serde::Serialize)]
struct User { id: String, email: String }

// Automatic conversion from anyhow:
async fn get_user(Path(id): Path<String>) -> Result<Json<User>, Error> {
    let user = db::find_user(&id).await?; // anyhow error converts automatically
    Ok(Json(user))
}

// Structured validation errors:
async fn create_user(Json(data): Json<CreateUser>) -> Result<Json<User>, Error> {
    let mut errors = HashMap::new();
    
    if !data.email.contains('@') {
        errors.insert("email".to_string(), "must be a valid email".to_string());
    }
    if data.age < 18 {
        errors.insert("age".to_string(), "must be 18 or older".to_string());
    }
    
    if !errors.is_empty() {
        return Err(validation(errors).with_trace_id("abc-123"));
    }
    
    Ok(Json(User { id: "123".to_string(), email: data.email }))
}

// On validation error, returns HTTP 400:
// {
//   "code": "VALIDATION_FAILED",
//   "message": "Invalid input",
//   "details": {
//     "fields": {
//       "email": "must be a valid email",
//       "age": "must be 18 or older"
//     }
//   },
//   "trace_id": "abc-123",
//   "retryable": false
// }
```

## Table of Contents

- [Why error-envelope](#why-error-envelope)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [API Reference](API.md) - Complete API documentation
- [Error Codes](ERROR_CODES.md) - All 18 error codes with descriptions

## Why error-envelope

APIs need a formal contract for errors. Without one, clients can't predict error structure:

```json
{"error": "bad request"}
```
String field, no structure.

```json
{"message": "invalid", "code": 400}
```
Different field names, ad-hoc.

```json
{"errors": [{"field": "email"}]}
```
Array structure, incompatible.

Every endpoint becomes a special case. `error-envelope` establishes a predictable contract: same structure, same fields, every time.

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
use std::time::Duration;

// Create errors with builder pattern:
let err = Error::rate_limited("too many requests")
    .with_trace_id("abc-123")
    .with_retry_after(Duration::from_secs(30));
```

That's it. See the hero example above for Axum integration and validation patterns.

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

Complete working examples in the [`examples/`](examples/) directory:

- **[`domain_errors.rs`](examples/domain_errors.rs)** - Map thiserror domain errors to HTTP errors (From pattern)
- **[`validation.rs`](examples/validation.rs)** - Field-level validation with structured error details
- **[`rate_limiting.rs`](examples/rate_limiting.rs)** - Rate limiting with retry-after hints
- **[`tracing.rs`](examples/tracing.rs)** - Trace ID propagation through middleware
- **[`axum_server.rs`](examples/axum_server.rs)** - Complete Axum server with all patterns

Run any example:
```bash
cargo run --example domain_errors --features axum-support
cargo run --example validation --features axum-support
cargo run --example rate_limiting --features axum-support
cargo run --example tracing --features axum-support
```

## Testing

```bash
cargo test --all-features
```

## License

MIT
