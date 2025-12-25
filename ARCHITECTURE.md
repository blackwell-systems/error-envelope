# error-envelope Architecture Guide

Visual guide to understanding when and how to use error-envelope in your Rust API.

## Table of Contents

- [The Three-Layer Error Model](#the-three-layer-error-model)
- [When to Use error-envelope](#when-to-use-error-envelope)
- [Error Flow Through Your Application](#error-flow-through-your-application)
- [Integration Patterns](#integration-patterns)
- [Client Benefits](#client-benefits)

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

## When to Use error-envelope

Use error-envelope when you need **consistent, structured HTTP error responses** across your API.

```mermaid
flowchart TB
    subgraph problems["Without error-envelope"]
        p1["Inconsistent formats<br/>across endpoints"]
        p2["No trace IDs<br/>for debugging"]
        p3["No retry signals<br/>for clients"]
        p4["Manual JSON<br/>serialization"]
    end

    arrow["error-envelope"]

    subgraph solution["With error-envelope"]
        s1["One JSON structure<br/>everywhere"]
        s2["Automatic trace ID<br/>propagation"]
        s3["Built-in retryable<br/>signals"]
        s4["Zero boilerplate<br/>via traits"]
    end

    problems --> arrow
    arrow --> solution

    style problems fill:#4C3A3C,stroke:#6b7280,color:#f0f0f0
    style solution fill:#3A4C43,stroke:#6b7280,color:#f0f0f0
    style arrow fill:#CC8F00,stroke:#6b7280,color:#f0f0f0
```

### Use Cases

**Perfect for:**
- REST APIs with multiple endpoints
- Services consumed by mobile apps
- Microservices with distributed tracing
- APIs with validation requirements
- Services that call downstream APIs

**Skip if:**
- Internal-only services (no external clients)
- gRPC or binary protocols (not HTTP/JSON)
- Single-endpoint services with simple errors
- Existing error standard you must maintain

---

## Error Flow Through Your Application

Here's how errors flow from your domain logic to HTTP responses:

```mermaid
sequenceDiagram
    participant Client
    participant Handler as Axum Handler
    participant Service as Business Logic
    participant Domain as Domain Layer
    participant DB as Database

    Client->>Handler: POST /bookings
    Handler->>Service: create_booking(dto)
    
    Service->>Domain: BookingRequest::try_from(dto)
    Domain-->>Service: ValidationError
    
    Service-->>Handler: anyhow::Error
    Handler->>Handler: Convert to error-envelope::Error
    Handler-->>Client: 400 Bad Request<br/>{code, message, details, trace_id}

    Note over Handler,Client: error-envelope creates<br/>structured JSON response

    rect rgb(58, 74, 92)
        Note over Handler: IntoResponse trait<br/>automatically converts<br/>Error to HTTP response
    end
```

### Code Example

```rust
use axum::Json;
use error_envelope::Error;
use anyhow::Result as AnyResult;

// Domain layer: typed errors
#[derive(thiserror::Error, Debug)]
enum BookingError {
    #[error("Check-in date must be in the future")]
    InvalidCheckIn,
    
    #[error("No rooms available for {0} guests")]
    NoAvailability(u8),
}

// Service layer: anyhow for flexibility
async fn create_booking(dto: BookingDto) -> AnyResult<Booking> {
    let booking = BookingRequest::try_from(dto)?;  // ValidationError -> anyhow
    let availability = check_availability(&booking).await?;  // BookingError -> anyhow
    Ok(save_booking(booking).await?)
}

// HTTP layer: error-envelope for structure
async fn handler(
    Json(dto): Json<BookingDto>
) -> Result<Json<Booking>, Error> {
    let booking = create_booking(dto).await
        .map_err(|e| Error::from(e))?;  // anyhow -> error-envelope
    Ok(Json(booking))
}
```

**Key transitions:**
1. Domain error → `anyhow` (via `?` operator)
2. `anyhow` → `error-envelope` (via `From<anyhow::Error>` with `anyhow-support` feature)
3. `error-envelope` → HTTP response (via `IntoResponse` with `axum-support` feature)

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

---

## Client Benefits

error-envelope makes life easier for API clients (web apps, mobile apps, CLI tools).

### Before: Inconsistent Error Handling

```mermaid
flowchart TB
    subgraph client["Client Code"]
        parse["Parse Response"]
    end

    subgraph endpoints["API Endpoints"]
        e1["POST /users<br/>returns {error: string}"]
        e2["GET /bookings<br/>returns {message: string}"]
        e3["PUT /profile<br/>returns {code: int, msg: string}"]
    end

    subgraph parsing["Parsing Strategies"]
        p1["Strategy 1:<br/>error field"]
        p2["Strategy 2:<br/>message field"]
        p3["Strategy 3:<br/>code + msg fields"]
    end

    e1 --> p1
    e2 --> p2
    e3 --> p3
    p1 --> parse
    p2 --> parse
    p3 --> parse

    style client fill:#4C3A3C,stroke:#6b7280,color:#f0f0f0
    style endpoints fill:#3A4A5C,stroke:#6b7280,color:#f0f0f0
    style parsing fill:#4C4538,stroke:#6b7280,color:#f0f0f0
```

### After: One Parsing Strategy

```mermaid
flowchart TB
    subgraph client["Client Code"]
        parse["Parse Error Envelope<br/>(one strategy)"]
        handle["Smart Error Handling"]
    end

    subgraph endpoints["API Endpoints"]
        e1["POST /users"]
        e2["GET /bookings"]
        e3["PUT /profile"]
    end

    subgraph response["Consistent Response"]
        envelope["error_envelope::Error<br/>{code, message, details,<br/>trace_id, retryable}"]
    end

    subgraph actions["Client Actions"]
        a1["Highlight form fields<br/>(from details)"]
        a2["Show retry button<br/>(if retryable)"]
        a3["Log trace ID<br/>(for support)"]
        a4["Match on code<br/>(for logic)"]
    end

    e1 --> envelope
    e2 --> envelope
    e3 --> envelope
    envelope --> parse
    parse --> handle
    handle --> a1
    handle --> a2
    handle --> a3
    handle --> a4

    style client fill:#3A4A5C,stroke:#6b7280,color:#f0f0f0
    style endpoints fill:#3A4C43,stroke:#6b7280,color:#f0f0f0
    style response fill:#4C4538,stroke:#6b7280,color:#f0f0f0
    style actions fill:#4C3A3C,stroke:#6b7280,color:#f0f0f0
```

### Example: Mobile App Error Handler

```typescript
// TypeScript/React Native - one error handler for entire API
interface ErrorEnvelope {
  code: string;
  message: string;
  details?: { fields?: Record<string, string> };
  trace_id?: string;
  retryable: boolean;
}

function handleApiError(error: ErrorEnvelope, formRef: FormRef) {
  // Log trace ID for debugging
  if (error.trace_id) {
    console.error(`API Error [${error.trace_id}]:`, error.message);
  }

  // Handle validation errors
  if (error.code === "VALIDATION_FAILED" && error.details?.fields) {
    Object.entries(error.details.fields).forEach(([field, msg]) => {
      formRef.setFieldError(field, msg);  // Highlight specific inputs
    });
    return;
  }

  // Show retry for transient failures
  if (error.retryable) {
    showRetryDialog(error.message);
    return;
  }

  // Handle specific error codes
  switch (error.code) {
    case "UNAUTHORIZED":
      redirectToLogin();
      break;
    case "RATE_LIMITED":
      showRateLimitMessage(error.retry_after);
      break;
    default:
      showGenericError(error.message);
  }
}
```

**One function handles every API error** with field-level validation, trace IDs, and smart retry logic.

---

## Decision Tree: Do You Need error-envelope?

```mermaid
flowchart TD
    start["Building a Rust API?"]
    
    start -->|Yes| external["External clients?<br/>(web, mobile, CLI)"]
    start -->|No| skip1["Skip error-envelope"]
    
    external -->|Yes| multiple["Multiple endpoints?"]
    external -->|No| skip2["Skip error-envelope<br/>(unless you want structure)"]
    
    multiple -->|Yes| validation["Need field-level<br/>validation errors?"]
    multiple -->|No| skip3["Skip error-envelope<br/>(unless you want consistency)"]
    
    validation -->|Yes| use1["Use error-envelope<br/>+ validation library"]
    validation -->|No| simple["Just need consistent<br/>error structure?"]
    
    simple -->|Yes| use2["Use error-envelope<br/>(basic integration)"]
    simple -->|No| consider["Consider error-envelope<br/>for future-proofing"]

    style start fill:#3A4A5C,stroke:#6b7280,color:#f0f0f0
    style use1 fill:#2A9F66,stroke:#6b7280,color:#f0f0f0
    style use2 fill:#2A9F66,stroke:#6b7280,color:#f0f0f0
    style consider fill:#CC8F00,stroke:#6b7280,color:#f0f0f0
    style skip1 fill:#4C3A3C,stroke:#6b7280,color:#f0f0f0
    style skip2 fill:#4C3A3C,stroke:#6b7280,color:#f0f0f0
    style skip3 fill:#4C3A3C,stroke:#6b7280,color:#f0f0f0
    style external fill:#3A4A5C,stroke:#6b7280,color:#f0f0f0
    style multiple fill:#3A4A5C,stroke:#6b7280,color:#f0f0f0
    style validation fill:#3A4A5C,stroke:#6b7280,color:#f0f0f0
    style simple fill:#3A4A5C,stroke:#6b7280,color:#f0f0f0
```

---

## Error Flow Through Your Application

### Example: Booking API

```mermaid
flowchart TB
    subgraph client["Client Layer"]
        mobile["Mobile App"]
        web["Web App"]
    end

    subgraph api["API Layer (Axum)"]
        handler["Handler:<br/>create_booking()"]
        extractor["JSON Extractor"]
    end

    subgraph service["Service Layer"]
        service_fn["Service:<br/>process_booking()"]
        validation["Validation"]
        business["Business Logic"]
    end

    subgraph external["External Services"]
        payment["Payment Gateway"]
        inventory["Inventory Service"]
    end

    subgraph response["Error Response"]
        envelope["error_envelope::Error"]
        json["JSON:<br/>{code: 'VALIDATION_FAILED',<br/>message: 'Invalid input',<br/>details: {fields: {...}},<br/>trace_id: 'abc-123',<br/>retryable: false}"]
    end

    mobile --> handler
    web --> handler
    handler --> extractor
    extractor -->|"Serde error"| envelope
    extractor -->|"Valid JSON"| service_fn
    
    service_fn --> validation
    validation -->|"Invalid"| envelope
    validation -->|"Valid"| business
    
    business --> payment
    payment -->|"Timeout"| envelope
    business --> inventory
    inventory -->|"No rooms"| envelope

    envelope --> json
    json --> mobile
    json --> web

    style client fill:#3A4A5C,stroke:#6b7280,color:#f0f0f0
    style api fill:#3A4C43,stroke:#6b7280,color:#f0f0f0
    style service fill:#4C4538,stroke:#6b7280,color:#f0f0f0
    style external fill:#4C3A3C,stroke:#6b7280,color:#f0f0f0
    style response fill:#CC8F00,stroke:#6b7280,color:#f0f0f0
```

**Every error path converges to error-envelope** before reaching clients.

---

## Integration Patterns

### Pattern: Microservices with Distributed Tracing

```mermaid
flowchart LR
    subgraph gateway["API Gateway"]
        gw["Generate<br/>Trace ID"]
    end

    subgraph service1["Booking Service"]
        s1["Handler receives<br/>X-Request-ID"]
        s1_err["Error with<br/>trace_id"]
    end

    subgraph service2["Payment Service"]
        s2["Downstream call<br/>with trace_id"]
        s2_err["Error propagates<br/>trace_id"]
    end

    subgraph logging["Observability"]
        logs["Structured Logs<br/>(trace_id in context)"]
        traces["Distributed Traces<br/>(span correlation)"]
    end

    gw -->|"X-Request-ID: abc-123"| s1
    s1 --> s1_err
    s1 -->|"Forward trace_id"| s2
    s2 --> s2_err
    s2_err -->|"with_trace_id('abc-123')"| s1_err
    
    s1_err --> logs
    s2_err --> logs
    s1_err --> traces
    s2_err --> traces

    style gateway fill:#3A4A5C,stroke:#6b7280,color:#f0f0f0
    style service1 fill:#3A4C43,stroke:#6b7280,color:#f0f0f0
    style service2 fill:#4C4538,stroke:#6b7280,color:#f0f0f0
    style logging fill:#4C3A3C,stroke:#6b7280,color:#f0f0f0
```

**Benefits:**
- Correlate errors across service boundaries
- Find failed requests in logs by trace ID
- Debug distributed transactions end-to-end
- Clients include trace IDs in bug reports

### Pattern: Rate Limiting with Retry-After

```mermaid
sequenceDiagram
    participant Client
    participant API
    participant RateLimiter

    Client->>API: POST /bookings (attempt 1)
    API->>RateLimiter: Check rate limit
    RateLimiter-->>API: Limit exceeded
    API-->>Client: 429 Rate Limited<br/>{code: "RATE_LIMITED",<br/>retryable: true,<br/>retry_after: "30s"}

    Note over Client: Wait 30 seconds<br/>(from retry_after)

    Client->>API: POST /bookings (attempt 2)
    API->>RateLimiter: Check rate limit
    RateLimiter-->>API: OK
    API-->>Client: 200 OK

    rect rgb(58, 74, 92)
        Note over API,Client: error-envelope provides<br/>retry_after duration<br/>and retryable flag
    end
```

**Code:**

```rust
use error_envelope::Error;
use std::time::Duration;

async fn handler() -> Result<Json<Data>, Error> {
    if is_rate_limited() {
        return Err(
            Error::rate_limited("Too many requests")
                .with_retry_after(Duration::from_secs(30))
                .with_trace_id(request_id)
        );
    }
    
    Ok(Json(process_request().await?))
}
```

**Client sees:**
```json
{
  "code": "RATE_LIMITED",
  "message": "Too many requests",
  "retryable": true,
  "retry_after": "30s",
  "trace_id": "abc-123"
}
```

---

## Comparison: Manual vs error-envelope

### Manual Error Handling (Before)

```rust
async fn handler() -> Result<Json<User>, StatusCode> {
    match find_user("123").await {
        Ok(user) => Ok(Json(user)),
        Err(e) => {
            // Lost all error context!
            // No trace ID, no structured details, no retry hint
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
```

**Problems:**
- No structured error details
- No trace IDs for debugging
- No retry signals for clients
- Loses error context
- Inconsistent across endpoints

### With error-envelope (After)

```rust
use error_envelope::Error;

async fn handler() -> Result<Json<User>, Error> {
    let user = find_user("123").await?;  // anyhow converts automatically
    Ok(Json(user))
}

// Automatic error response:
// {
//   "code": "INTERNAL",
//   "message": "Database connection failed",
//   "trace_id": "abc-123",
//   "retryable": false
// }
```

**Benefits:**
- Structured JSON with stable codes
- Automatic trace ID propagation
- Retry signals for transient failures
- Consistent format across all endpoints
- One line of code (`?` operator)

```mermaid
flowchart LR
    subgraph manual["Manual (Before)"]
        m1["StatusCode::500"]
        m2["Lost context"]
        m3["No trace IDs"]
        m4["Client guesses"]
    end

    subgraph envelope["error-envelope (After)"]
        e1["Structured JSON"]
        e2["Error context"]
        e3["Trace IDs"]
        e4["Smart clients"]
    end

    manual -.->|"10 lines of<br/>boilerplate per<br/>endpoint"| envelope

    style manual fill:#4C3A3C,stroke:#6b7280,color:#f0f0f0
    style envelope fill:#2A9F66,stroke:#6b7280,color:#f0f0f0
```

---

## Summary

**error-envelope is a thin layer at the HTTP boundary** that gives you:

1. **Consistency** - One error format across all endpoints
2. **Structure** - Machine-readable codes, human-readable messages, structured details
3. **Observability** - Trace IDs for distributed debugging
4. **Resilience** - Retry signals for transient failures
5. **Zero boilerplate** - `IntoResponse` trait handles serialization

**Use it when:**
- Building REST APIs with external clients
- Multiple endpoints need consistent errors
- You want field-level validation feedback
- Distributed tracing is important
- Client retry logic matters

**Skip it when:**
- Internal-only services (no external clients)
- Single-endpoint APIs with simple errors
- Using non-HTTP protocols (gRPC, etc.)
- Existing error standard you must maintain

**The key insight:** error-envelope doesn't replace thiserror or anyhow--it complements them by handling the HTTP boundary. Your domain still uses thiserror for typed errors, your application layer still uses anyhow for flexibility, and error-envelope handles the final conversion to structured HTTP responses.
