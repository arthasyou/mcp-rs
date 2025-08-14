use axum::{
    extract::State,
    response::sse::{Event, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::stream::{self, Stream};
use serde_json::{json, Value};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tokio_stream::StreamExt as _;

#[derive(Clone)]
struct AppState {
    notification_count: Arc<Mutex<u32>>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        notification_count: Arc::new(Mutex::new(0)),
    };

    let app = Router::new()
        .route("/sse", get(sse_handler))
        .route("/message", post(message_handler))
        .with_state(state.clone());

    // Spawn a task to send periodic notifications
    let state_clone = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let mut count = state_clone.notification_count.lock().await;
            *count += 1;
            println!("Server: Preparing to send notification #{}", *count);
        }
    });

    println!("Starting SSE test server on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("SSE client connected");
    
    let initial_event = Event::default()
        .event("endpoint")
        .data("/message");

    let notification_stream = stream::unfold(state, |state| async move {
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        let count = {
            let mut c = state.notification_count.lock().await;
            *c += 1;
            *c
        };
        
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "progress",
            "params": {
                "message": format!("Progress update #{}", count),
                "percentage": (count * 10) % 100
            }
        });
        
        println!("Server: Sending notification: {}", notification);
        
        let event = Event::default()
            .event("message")
            .data(notification.to_string());
            
        Some((Ok(event), state))
    });

    let combined_stream = stream::once(async { Ok(initial_event) })
        .chain(notification_stream);

    Sse::new(combined_stream)
}

async fn message_handler(
    Json(payload): Json<Value>,
) -> Json<Value> {
    println!("Server received message: {}", payload);
    
    // Extract request ID
    let id = payload.get("id").cloned().unwrap_or(json!(null));
    
    // Simple echo response
    let response = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "echo": payload.get("params").cloned().unwrap_or(json!({})),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    });
    
    println!("Server sending response: {}", response);
    Json(response)
}