//! A minimal Axum server demonstrating error-envelope usage.
//!
//! Run this example with:
//! ```bash
//! cargo run --example axum_server --features axum-support
//! ```
//!
//! Then test endpoints:
//! ```bash
//! curl http://localhost:3000/user?id=123
//! curl http://localhost:3000/user?id=
//! curl http://localhost:3000/rate-limit
//! curl http://localhost:3000/validation
//! ```

use axum::{extract::Query, routing::get, Router};
use error_envelope::{Error, FieldErrors};
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
struct UserQuery {
    id: Option<String>,
}

async fn get_user(Query(params): Query<UserQuery>) -> Result<String, Error> {
    match params.id {
        Some(id) if !id.is_empty() => Ok(format!("User: {}", id)),
        Some(_) => Err(Error::bad_request("User ID cannot be empty")),
        None => Err(Error::bad_request("User ID is required")
            .with_details(serde_json::json!({"parameter": "id"}))),
    }
}

async fn rate_limit_example() -> Result<String, Error> {
    Err(Error::rate_limited("Too many requests")
        .with_retry_after(Duration::from_secs(30))
        .with_trace_id("abc-123-def"))
}

async fn validation_example() -> Result<String, Error> {
    let mut fields = FieldErrors::new();
    fields.insert("email".to_string(), "must be a valid email".to_string());
    fields.insert("age".to_string(), "must be 18 or older".to_string());

    Err(error_envelope::validation(fields).with_trace_id("validation-error-456"))
}

async fn downstream_example() -> Result<String, Error> {
    let io_err = std::io::Error::new(std::io::ErrorKind::TimedOut, "connection timeout");
    Err(Error::downstream_timeout("payments-api", io_err).with_trace_id("downstream-789"))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/user", get(get_user))
        .route("/rate-limit", get(rate_limit_example))
        .route("/validation", get(validation_example))
        .route("/downstream", get(downstream_example));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running on http://localhost:3000");
    println!("Try:");
    println!("  curl http://localhost:3000/user?id=123");
    println!("  curl http://localhost:3000/user?id=");
    println!("  curl http://localhost:3000/rate-limit");
    println!("  curl http://localhost:3000/validation");
    println!("  curl http://localhost:3000/downstream");

    axum::serve(listener, app).await.unwrap();
}
