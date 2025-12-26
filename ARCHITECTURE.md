# error-envelope Architecture Guide

Visual guide to understanding when and how to use error-envelope in your Rust API.

## Table of Contents

- [The Three-Layer Error Model](#the-three-layer-error-model)
- [Integration Patterns](#integration-patterns)
- [Summary](#summary)

---

## The Three-Layer Error Model

Most Rust APIs have three distinct error handling layers. Each layer has different requirements:

```mermaid
flowchart TB
    subgraph domain["Domain Layer (Business Logic)"]
        domain_code["Domain Code<br/>───────────<br/>• Typed errors<br/>• Pattern matching<br/>• Exhaustive checking<br/>• Unit testing"]
        domain_lib["thiserror"]
    end

    subgraph app["Application Layer (Handlers/Services)"]
        app_code["Application Code<br/>───────────<br/>• Error propagation<br/>• Context chaining<br/>• Conversion<br/>• Logging"]
        app_lib["anyhow"]
    end

    subgraph http["HTTP Boundary (API Responses)"]
        http_code["HTTP Responses<br/>───────────<br/>• Structured JSON<br/>• Status codes<br/>• Trace IDs<br/>• Client contracts"]
        http_lib["error-envelope"]
    end

    domain_code --> app_code
    app_code --> http_code
    domain_lib -.->|"used by"| domain_code
    app_lib -.->|"used by"| app_code
    http_lib -.->|"used by"| http_code

    style domain fill:#3A4A5C,stroke:#6b7280,color:#f0f0f0
    style app fill:#3A4C43,stroke:#6b7280,color:#f0f0f0
    style http fill:#4C4538,stroke:#6b7280,color:#f0f0f0
```

**error-envelope lives at the HTTP boundary.** It's the last stop before errors become JSON responses that clients consume.



---

## Integration Patterns

### Pattern 1: Direct Usage (No Features)

Framework-agnostic. Manually serialize and write responses.

```rust
use error_envelope::Error;

fn handler() -> Result<String, Error> {
    if missing_auth() {
        return Err(Error::unauthorized("Missing token"));
    }
    Ok("success".to_string())
}

// Manual serialization
match handler() {
    Ok(data) => send_json(200, data),
    Err(e) => {
        let json = serde_json::to_string(&e)?;
        send_json(e.status, json)
    }
}
```

### Pattern 2: Axum Integration (axum-support feature)

Automatic HTTP response conversion via `IntoResponse` trait.

```rust
use axum::Json;
use error_envelope::Error;

async fn handler() -> Result<Json<User>, Error> {
    let user = find_user("123").await?;  // anyhow::Error converts automatically
    Ok(Json(user))
}

// Error automatically becomes:
// - HTTP response with correct status
// - JSON body with error envelope
// - X-Request-ID header (if trace_id set)
// - Retry-After header (if retry_after set)
```

```mermaid
flowchart LR
    subgraph handler["Handler Returns Result<T, Error>"]
        result["Result<Json<User>, Error>"]
    end

    subgraph axum["Axum Framework"]
        into_response["IntoResponse trait"]
    end

    subgraph http["HTTP Response"]
        status["Status: 404"]
        headers["Headers:<br/>Content-Type: application/json<br/>X-Request-ID: abc-123"]
        body["Body: {code, message, ...}"]
    end

    result --> into_response
    into_response --> status
    into_response --> headers
    into_response --> body

    style handler fill:#3A4A5C,stroke:#6b7280,color:#f0f0f0
    style axum fill:#3A4C43,stroke:#6b7280,color:#f0f0f0
    style http fill:#4C4538,stroke:#6b7280,color:#f0f0f0
```

### Pattern 3: With anyhow (anyhow-support feature)

Seamless conversion from application-layer errors to HTTP errors.

```rust
use error_envelope::Error;
use anyhow::{Result, Context};

async fn handler() -> Result<Json<Data>, Error> {
    // All anyhow errors convert automatically via ?
    let config = load_config().await?;
    let data = fetch_data(&config).await
        .context("Failed to fetch data")?;
    Ok(Json(data))
}

// Any anyhow::Error becomes error-envelope::Error (Internal/500)
// with the error message preserved
```

```mermaid
flowchart TB
    subgraph sources["Error Sources"]
        db["Database Error<br/>(sqlx)"]
        io["I/O Error<br/>(std::io)"]
        parse["Parse Error<br/>(serde)"]
        custom["Custom Error<br/>(thiserror)"]
    end

    subgraph anyhow["anyhow Layer"]
        anyerr["anyhow::Error<br/>(type-erased)"]
    end

    subgraph envelope["error-envelope Layer"]
        env["error_envelope::Error<br/>(structured HTTP)"]
    end

    subgraph response["HTTP Response"]
        json["JSON body with<br/>code, message, trace_id"]
    end

    db --> anyerr
    io --> anyerr
    parse --> anyerr
    custom --> anyerr

    anyerr -->|"From<anyhow::Error>"| env
    env -->|"IntoResponse"| json

    style sources fill:#3A4A5C,stroke:#6b7280,color:#f0f0f0
    style anyhow fill:#3A4C43,stroke:#6b7280,color:#f0f0f0
    style envelope fill:#4C4538,stroke:#6b7280,color:#f0f0f0
    style response fill:#4C3A3C,stroke:#6b7280,color:#f0f0f0
```

### Pattern 4: Domain Errors with thiserror (Recommended)

Explicit mapping from typed domain errors to HTTP semantics.

```rust
use error_envelope::{Code, Error};
use thiserror::Error as ThisError;

// Define domain errors with thiserror
#[derive(ThisError, Debug)]
pub enum DomainError {
    #[error("user not found")]
    NotFound,
    
    #[error("email already exists")]
    EmailConflict,
    
    #[error("database error")]
    Database(#[from] anyhow::Error),
}

// Map domain errors to HTTP errors
impl From<DomainError> for Error {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::NotFound =>
                Error::new(Code::NotFound, 404, "User not found"),
            DomainError::EmailConflict =>
                Error::new(Code::Conflict, 409, "Email already exists"),
            DomainError::Database(cause) =>
                Error::wrap(Code::Internal, 500, "Database failure", cause),
        }
    }
}

// Handlers use ? for automatic conversion
async fn handler() -> Result<Json<User>, Error> {
    let user = get_user().await?; // DomainError -> Error via From
    Ok(Json(user))
}
```

```mermaid
flowchart TB
    subgraph domain["Domain Layer"]
        derr["DomainError<br/>───────────<br/>• NotFound<br/>• EmailConflict<br/>• Database"]
    end
    
    subgraph mapping["From Trait"]
        from["impl From&lt;DomainError&gt;<br/>───────────<br/>• Explicit HTTP semantics<br/>• Preserves cause messages<br/>• No accidental 500s"]
    end
    
    subgraph envelope["error-envelope"]
        env["Error<br/>───────────<br/>• Code: NotFound/Conflict<br/>• Status: 404/409<br/>• Message: Human-readable"]
    end
    
    subgraph response["HTTP Response"]
        json["JSON Response<br/>───────────<br/>• {code, message, ...}<br/>• Proper status code<br/>• Structured details"]
    end
    
    derr -->|"?"| from
    from --> env
    env -->|"IntoResponse"| json
    
    style domain fill:#3A4A5C,stroke:#6b7280,color:#f0f0f0
    style mapping fill:#3A4C43,stroke:#6b7280,color:#f0f0f0
    style envelope fill:#4C4538,stroke:#6b7280,color:#f0f0f0
    style response fill:#4C3A3C,stroke:#6b7280,color:#f0f0f0
```

**Why this pattern?**

- **Explicit HTTP semantics** - You decide which domain errors map to which HTTP codes
- **No accidental 500s** - NotFound becomes 404, not Internal (unlike anyhow path)
- **Zero boilerplate** - Handlers use `?` just like with anyhow
- **Type safety** - Compile-time guarantee all domain errors are mapped

See [`examples/domain_errors.rs`](examples/domain_errors.rs) for complete example.




---

## Summary

**error-envelope is a thin layer at the HTTP boundary** that gives you:

1. **Consistency** - One error format across all endpoints
2. **Structure** - Machine-readable codes, human-readable messages, structured details
3. **Observability** - Trace IDs for distributed debugging
4. **Resilience** - Retry signals for transient failures
5. **Zero boilerplate** - `IntoResponse` trait handles serialization

error-envelope doesn't replace thiserror or anyhow--it complements them by handling the HTTP boundary. Your domain still uses thiserror for typed errors, your application layer still uses anyhow for flexibility, and error-envelope handles the final conversion to structured HTTP responses.
