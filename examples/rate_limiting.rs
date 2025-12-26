/// Example: Rate limiting with retry-after hints
///
/// This demonstrates how to return rate limit errors with retry-after
/// hints that tell clients when to retry.
///
/// Run with: cargo run --example rate_limiting --features axum-support
use axum::{extract::Path, routing::get, Json, Router};
use error_envelope::Error;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    fn check(&self, user_id: &str) -> Result<(), Error> {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();

        // Get or create request history for this user
        let history = requests.entry(user_id.to_string()).or_insert_with(Vec::new);

        // Remove old requests outside the time window
        history.retain(|&time| now.duration_since(time) < self.window);

        // Check if limit exceeded
        if history.len() >= self.max_requests {
            let oldest = history.first().unwrap();
            let retry_after = self.window - now.duration_since(*oldest);

            return Err(Error::rate_limited("Too many requests")
                .with_retry_after(retry_after)
                .with_details(serde_json::json!({
                    "limit": self.max_requests,
                    "window": format!("{}s", self.window.as_secs()),
                    "reset_at": format!("{}s", retry_after.as_secs())
                }))
                .with_trace_id("rate-limit-check"));
        }

        // Record this request
        history.push(now);
        Ok(())
    }
}

#[derive(Serialize)]
struct ApiResponse {
    message: String,
    user_id: String,
}

async fn api_endpoint(
    Path(user_id): Path<String>,
    limiter: axum::extract::State<RateLimiter>,
) -> Result<Json<ApiResponse>, Error> {
    // Check rate limit
    limiter.check(&user_id)?;

    // Process request
    Ok(Json(ApiResponse {
        message: "Success".to_string(),
        user_id,
    }))
}

#[tokio::main]
async fn main() {
    // Allow 3 requests per 10 seconds
    let limiter = RateLimiter::new(3, Duration::from_secs(10));

    let app = Router::new()
        .route("/api/:user_id", get(api_endpoint))
        .with_state(limiter);

    println!("Starting server on http://localhost:3000");
    println!("\nRate limit: 3 requests per 10 seconds");
    println!("\nTest rate limiting:");
    println!("  # Make 4 requests quickly (4th will be rate limited):");
    println!("  for i in {{1..4}}; do");
    println!(r#"    curl http://localhost:3000/api/user123"#);
    println!("    echo");
    println!("  done");
    println!("\nThe 4th request will return:");
    println!(r#"  {{"code":"RATE_LIMITED","retry_after":"10s",...}}"#);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
