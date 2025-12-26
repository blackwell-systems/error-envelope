/// Example: Trace ID propagation for distributed tracing
///
/// This demonstrates how to propagate trace IDs through your application
/// for correlation across services and log entries.
///
/// Run with: cargo run --example tracing --features axum-support

use axum::{
    extract::{Path, Request},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Json, Router,
};
use error_envelope::Error;
use serde::Serialize;
use uuid::Uuid;

// Middleware to extract or generate trace ID
async fn trace_id_middleware(mut req: Request, next: Next) -> Response {
    // Extract trace ID from X-Request-ID header or generate new one
    let trace_id = req
        .headers()
        .get("X-Request-ID")
        .and_then(|h| h.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Store trace ID in request extensions for handlers to access
    req.extensions_mut().insert(TraceId(trace_id));

    next.run(req).await
}

#[derive(Clone)]
struct TraceId(String);

#[derive(Serialize)]
struct User {
    id: String,
    email: String,
}

async fn get_user(
    Path(id): Path<String>,
    axum::Extension(trace_id): axum::Extension<TraceId>,
) -> Result<Json<User>, Error> {
    println!("[TRACE: {}] Looking up user: {}", trace_id.0, id);

    // Simulate database call that might fail
    let user = fetch_from_db(&id, &trace_id.0).await?;

    println!("[TRACE: {}] User found: {}", trace_id.0, user.id);
    Ok(Json(user))
}

async fn fetch_from_db(id: &str, trace_id: &str) -> Result<User, Error> {
    println!("[TRACE: {}] Database query for user {}", trace_id, id);

    // Simulate not found error with trace ID
    if id == "404" {
        return Err(Error::not_found("User not found").with_trace_id(trace_id));
    }

    // Simulate internal error with trace ID
    if id == "500" {
        return Err(Error::internal("Database connection failed")
            .with_trace_id(trace_id)
            .with_details(serde_json::json!({
                "operation": "user_lookup",
                "database": "primary"
            })));
    }

    Ok(User {
        id: id.to_string(),
        email: format!("user{}@example.com", id),
    })
}

async fn list_users(
    axum::Extension(trace_id): axum::Extension<TraceId>,
) -> Result<Json<Vec<User>>, Error> {
    println!("[TRACE: {}] Listing all users", trace_id.0);

    // Simulate downstream service call
    let users = call_downstream_service(&trace_id.0).await?;

    println!("[TRACE: {}] Found {} users", trace_id.0, users.len());
    Ok(Json(users))
}

async fn call_downstream_service(trace_id: &str) -> Result<Vec<User>, Error> {
    println!("[TRACE: {}] Calling downstream user service", trace_id);

    // Simulate downstream timeout
    Err(Error::timeout("Downstream service timeout")
        .with_trace_id(trace_id)
        .with_details(serde_json::json!({
            "service": "user-service",
            "endpoint": "/users/list"
        })))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/user/:id", get(get_user))
        .route("/users", get(list_users))
        .layer(middleware::from_fn(trace_id_middleware));

    println!("Starting server on http://localhost:3000");
    println!("\nTrace ID examples:");
    println!("\n  # Request with trace ID:");
    println!(r#"  curl -H "X-Request-ID: my-trace-123" http://localhost:3000/user/1"#);
    println!("\n  # Request without trace ID (will generate UUID):");
    println!("  curl http://localhost:3000/user/1");
    println!("\n  # Trigger not found (with trace in error):");
    println!("  curl http://localhost:3000/user/404");
    println!("\n  # Trigger internal error (with trace in error):");
    println!("  curl http://localhost:3000/user/500");
    println!("\n  # Trigger downstream timeout (with trace in error):");
    println!("  curl http://localhost:3000/users");
    println!("\nWatch console output to see trace ID propagation through the call chain.");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
