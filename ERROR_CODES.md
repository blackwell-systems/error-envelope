# Error Codes Reference

Complete reference of all error codes in error-envelope.

## Overview

error-envelope provides 18 standard error codes as a type-safe enum. Each code has:
- **Default HTTP status** - The standard status code for this error type
- **Default retryable behavior** - Whether clients should automatically retry
- **Default message** - Fallback message if none provided

---

## Complete Error Codes Table

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

---

## Detailed Descriptions

### Generic Errors

#### Internal (500)
Unexpected server errors that aren't the client's fault.

**When to use:**
- Database connection failures
- Unexpected panics or bugs
- File system errors
- Memory allocation failures

**Example:**
```rust
Error::internal("Database connection pool exhausted")
```

#### BadRequest (400)
Malformed requests that can't be processed.

**When to use:**
- Invalid JSON syntax
- Missing required headers
- Malformed URLs
- Invalid content types

**Example:**
```rust
Error::bad_request("Invalid JSON in request body")
```

#### ValidationFailed (400)
Input data is well-formed but violates business rules.

**When to use:**
- Email format invalid
- Age out of range
- Field too long/short
- Invalid enum values

**Example:**
```rust
use error_envelope::validation;
use std::collections::HashMap;

let mut fields = HashMap::new();
fields.insert("email".to_string(), "Invalid email format".to_string());
fields.insert("age".to_string(), "Must be between 18 and 120".to_string());

let err = validation(fields);
```

---

### Auth Errors

#### Unauthorized (401)
Authentication required or credentials invalid.

**When to use:**
- Missing authentication token
- Expired token
- Invalid credentials
- Token signature verification failed

**Example:**
```rust
Error::unauthorized("JWT token expired")
```

#### Forbidden (403)
Authenticated but not authorized to access resource.

**When to use:**
- Insufficient permissions
- Wrong role
- Resource belongs to different user
- Feature not enabled for account

**Example:**
```rust
Error::forbidden("Admin role required")
```

---

### Resource Errors

#### NotFound (404)
Resource doesn't exist at the requested path.

**When to use:**
- User/record not in database
- Endpoint doesn't exist
- File not found
- Invalid ID

**Example:**
```rust
Error::not_found("User not found")
```

#### MethodNotAllowed (405)
HTTP method not supported for this endpoint.

**When to use:**
- POST to GET-only endpoint
- DELETE not allowed
- Method not implemented

**Example:**
```rust
Error::method_not_allowed("POST not allowed on this endpoint")
```

#### Gone (410)
Resource permanently deleted or removed.

**When to use:**
- Soft-deleted records
- Expired offers
- Deprecated endpoints
- Permanently removed content

**Example:**
```rust
Error::gone("This endpoint was deprecated in v2.0")
```

#### Conflict (409)
Request conflicts with current resource state.

**When to use:**
- Duplicate email registration
- Concurrent modification
- Version mismatch
- Unique constraint violation

**Example:**
```rust
Error::conflict("Email already registered")
```

---

### Request Errors

#### RequestTimeout (408)
Client-side timeout before request completed.

**When to use:**
- Client closed connection early
- Long-running client operations
- Upload timeouts

**Example:**
```rust
Error::request_timeout("Client closed connection")
```

**Note:** Retryable by default.

#### PayloadTooLarge (413)
Request body exceeds size limit.

**When to use:**
- File upload too large
- JSON payload exceeds limit
- Request headers too large

**Example:**
```rust
Error::payload_too_large("Upload exceeds 10MB limit")
```

#### UnprocessableEntity (422)
Request is well-formed but semantically invalid.

**When to use:**
- Business rule violations
- Dates in wrong order
- Amounts exceed limits
- Complex validation failures

**Example:**
```rust
Error::unprocessable_entity("Check-out date must be after check-in")
```

---

### Infrastructure Errors

#### RateLimited (429)
Too many requests from client.

**When to use:**
- Rate limit exceeded
- Quota exhausted
- Throttling applied

**Example:**
```rust
use std::time::Duration;

Error::rate_limited("Too many requests")
    .with_retry_after(Duration::from_secs(60))
```

**Note:** Retryable by default. Always include `retry_after` to tell clients when to retry.

#### Unavailable (503)
Service temporarily unavailable.

**When to use:**
- Maintenance mode
- Circuit breaker open
- Overloaded
- Starting up

**Example:**
```rust
Error::unavailable("Service in maintenance mode")
```

**Note:** Retryable by default.

#### Timeout (504)
Gateway or processing timeout.

**When to use:**
- Database query timeout
- HTTP client timeout
- Long-running operations
- Background job timeout

**Example:**
```rust
Error::timeout("Database query exceeded 30s timeout")
```

**Note:** Retryable by default.

#### Canceled (499)
Client canceled the request.

**When to use:**
- Client closed connection
- Request aborted
- User navigation away from page

**Example:**
```rust
Error::new(Code::Canceled, 499, "Request canceled by client")
```

**Note:** Not retryable (client intentionally canceled).

---

### Downstream Errors

#### DownstreamError (502)
Downstream service returned an error.

**When to use:**
- Payment gateway failures
- External API errors
- Third-party service issues
- Microservice failures

**Example:**
```rust
let payment_err = call_payment_service().await.unwrap_err();
Error::downstream("payments", payment_err)
```

**Note:** Retryable by default. Service name automatically added to details.

#### DownstreamTimeout (504)
Downstream service timed out.

**When to use:**
- External API timeout
- Microservice timeout
- Third-party service slow
- Network issues

**Example:**
```rust
let inventory_err = call_inventory_service().await.unwrap_err();
Error::downstream_timeout("inventory", inventory_err)
```

**Note:** Retryable by default. Service name automatically added to details.

---

## Choosing the Right Code

### Quick Reference

**Client made a mistake:**
- `BadRequest` - Malformed request
- `ValidationFailed` - Invalid data
- `Unauthorized` - Not authenticated
- `Forbidden` - Not authorized
- `NotFound` - Resource doesn't exist
- `Conflict` - Duplicate/state conflict

**Server had a problem:**
- `Internal` - Unexpected error
- `Unavailable` - Temporarily down
- `Timeout` - Operation took too long

**Downstream service failed:**
- `DownstreamError` - Service returned error
- `DownstreamTimeout` - Service timed out

**Rate limiting:**
- `RateLimited` - Too many requests (include retry_after)

---

## Retryable vs Non-Retryable

### Retryable by Default (Yes)

These errors are transient—retrying might succeed:
- `RequestTimeout` (408)
- `RateLimited` (429)
- `Unavailable` (503)
- `Timeout` (504)
- `DownstreamError` (502)
- `DownstreamTimeout` (504)

### Not Retryable by Default (No)

These errors are permanent—retrying won't help:
- `Internal` (500) - Server bug, not transient
- `BadRequest` (400) - Malformed request
- `ValidationFailed` (400) - Invalid data
- `Unauthorized` (401) - Auth failure
- `Forbidden` (403) - Permission denied
- `NotFound` (404) - Doesn't exist
- `MethodNotAllowed` (405) - Wrong method
- `Conflict` (409) - State conflict
- `Gone` (410) - Permanently deleted
- `PayloadTooLarge` (413) - Too large
- `UnprocessableEntity` (422) - Invalid semantics
- `Canceled` (499) - Client canceled

### Overriding Retryable Behavior

```rust
// Make Internal retryable (unusual, but possible)
Error::internal("Transient cache issue")
    .with_retryable(true);

// Make Timeout non-retryable (if you know it won't help)
Error::timeout("Query complexity limit exceeded")
    .with_retryable(false);
```

---

## See Also

- [README.md](README.md) - Quick start and overview
- [ARCHITECTURE.md](ARCHITECTURE.md) - Visual architecture guide with diagrams
- [examples/](examples/) - Runnable code examples
- [docs.rs/error-envelope](https://docs.rs/error-envelope) - Full API documentation
