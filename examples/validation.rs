/// Example: Field-level validation with structured error details
///
/// This demonstrates how to return validation errors with field-specific
/// messages that clients can use to highlight form fields.
///
/// Run with: cargo run --example validation --features axum-support
use axum::{extract::Json, routing::post, Router};
use error_envelope::{validation, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
struct CreateUserRequest {
    email: String,
    password: String,
    age: u8,
    username: String,
}

#[derive(Serialize)]
struct User {
    id: String,
    email: String,
    username: String,
}

async fn create_user(Json(req): Json<CreateUserRequest>) -> Result<Json<User>, Error> {
    // Collect all validation errors
    let mut errors = HashMap::new();

    // Email validation
    if !req.email.contains('@') {
        errors.insert(
            "email".to_string(),
            "must be a valid email address".to_string(),
        );
    }
    if req.email.len() < 5 {
        errors.insert(
            "email".to_string(),
            "must be at least 5 characters".to_string(),
        );
    }

    // Password validation
    if req.password.len() < 8 {
        errors.insert(
            "password".to_string(),
            "must be at least 8 characters".to_string(),
        );
    }
    if !req.password.chars().any(|c| c.is_numeric()) {
        errors.insert(
            "password".to_string(),
            "must contain at least one number".to_string(),
        );
    }

    // Age validation
    if req.age < 18 {
        errors.insert("age".to_string(), "must be 18 or older".to_string());
    }
    if req.age > 120 {
        errors.insert("age".to_string(), "must be a valid age".to_string());
    }

    // Username validation
    if req.username.len() < 3 {
        errors.insert(
            "username".to_string(),
            "must be at least 3 characters".to_string(),
        );
    }
    if !req
        .username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_')
    {
        errors.insert(
            "username".to_string(),
            "can only contain letters, numbers, and underscores".to_string(),
        );
    }

    // Return validation error if any fields failed
    if !errors.is_empty() {
        return Err(validation(errors).with_trace_id("req-123"));
    }

    // All validations passed
    Ok(Json(User {
        id: "user-456".to_string(),
        email: req.email,
        username: req.username,
    }))
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/user", post(create_user));

    println!("Starting server on http://localhost:3000");
    println!("\nTest validation errors:");
    println!("\n  # Invalid email and short password:");
    println!(
        r#"  curl -X POST http://localhost:3000/user \
    -H "Content-Type: application/json" \
    -d '{{"email":"bad","password":"short","age":15,"username":"ab"}}'"#
    );

    println!("\n  # Valid request:");
    println!(
        r#"  curl -X POST http://localhost:3000/user \
    -H "Content-Type: application/json" \
    -d '{{"email":"user@example.com","password":"secure123","age":25,"username":"john_doe"}}'"#
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
