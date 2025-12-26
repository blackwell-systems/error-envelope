/// Example: Mapping thiserror domain errors to HTTP errors
///
/// This demonstrates the recommended pattern for integrating thiserror-based
/// domain errors with error-envelope at the HTTP boundary.
///
/// Run with: cargo run --example domain_errors --features axum-support

use axum::{routing::get, Json, Router};
use error_envelope::{Code, Error};
use serde::Serialize;
use thiserror::Error as ThisError;

// Domain errors defined with thiserror
#[derive(ThisError, Debug)]
pub enum DomainError {
    #[error("user not found")]
    UserNotFound,

    #[error("email already exists")]
    EmailConflict,

    #[error("insufficient permissions")]
    Forbidden,

    #[error("database error: {0}")]
    Database(String),
}

// Map domain errors to HTTP errors via From trait
impl From<DomainError> for Error {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::UserNotFound => Error::new(Code::NotFound, 404, "User not found"),

            DomainError::EmailConflict => {
                Error::new(Code::Conflict, 409, "Email already exists")
            }

            DomainError::Forbidden => {
                Error::new(Code::Forbidden, 403, "Insufficient permissions")
            }

            // Preserve cause message for debugging
            DomainError::Database(cause) => {
                Error::wrap(Code::Internal, 500, "Database operation failed", cause)
            }
        }
    }
}

#[derive(Serialize)]
struct User {
    id: String,
    email: String,
}

// Handlers use ? for automatic error conversion
async fn get_user() -> Result<Json<User>, Error> {
    // Domain error converts automatically via From
    find_user("123").await?;
    Ok(Json(User {
        id: "123".to_string(),
        email: "user@example.com".to_string(),
    }))
}

async fn create_user() -> Result<Json<User>, Error> {
    check_email_available("test@example.com").await?;
    Ok(Json(User {
        id: "456".to_string(),
        email: "test@example.com".to_string(),
    }))
}

async fn delete_user() -> Result<Json<()>, Error> {
    check_permissions("user123", "delete").await?;
    Ok(Json(()))
}

// Domain layer functions that return domain errors
async fn find_user(_id: &str) -> Result<User, DomainError> {
    // Simulate not found
    Err(DomainError::UserNotFound)
}

async fn check_email_available(_email: &str) -> Result<(), DomainError> {
    // Simulate conflict
    Err(DomainError::EmailConflict)
}

async fn check_permissions(_user: &str, _action: &str) -> Result<(), DomainError> {
    // Simulate forbidden
    Err(DomainError::Forbidden)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/user", get(get_user))
        .route("/user/create", get(create_user))
        .route("/user/delete", get(delete_user));

    println!("Starting server on http://localhost:3000");
    println!("\nTry these endpoints:");
    println!("  curl http://localhost:3000/user         # 404 User not found");
    println!("  curl http://localhost:3000/user/create  # 409 Email conflict");
    println!("  curl http://localhost:3000/user/delete  # 403 Forbidden");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
