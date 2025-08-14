use mcp_client_rust::transport::impls::sse::SseTransport;
use mcp_client_rust::transport::traits::ClientTransport;
use mcp_core::protocol::message::{JsonRpcMessage, JsonRpcRequest};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("Starting SSE client test...");
    
    // Initialize SSE transport
    let transport = SseTransport::new("http://localhost:3000/sse");
    
    // Counter for received notifications
    let notification_count = Arc::new(AtomicU32::new(0));
    let count_clone = notification_count.clone();
    
    // Method 1: Set a synchronous callback handler
    transport.set_message_handler(move |msg| {
        let count = count_clone.fetch_add(1, Ordering::SeqCst) + 1;
        println!("\n[SYNC HANDLER] Notification #{} received:", count);
        match &msg {
            JsonRpcMessage::Notification(notification) => {
                println!("  Method: {}", notification.method);
                println!("  Params: {}", serde_json::to_string_pretty(&notification.params).unwrap());
            }
            _ => {
                println!("  Unexpected message type: {:?}", msg);
            }
        }
    }).await;
    
    // Method 2: Get the async receiver for handling messages in a separate task
    if let Some(mut receiver) = transport.take_message_receiver().await {
        let count_clone2 = notification_count.clone();
        tokio::spawn(async move {
            println!("Async notification handler started");
            while let Some(msg) = receiver.recv().await {
                let count = count_clone2.load(Ordering::SeqCst);
                match msg {
                    JsonRpcMessage::Notification(notification) => {
                        println!("\n[ASYNC HANDLER] Processing notification #{}:", count);
                        println!("  Method: {}", notification.method);
                        
                        // Handle different notification types
                        match notification.method.as_str() {
                            "progress" => {
                                if let Some(params) = notification.params {
                                    if let Some(message) = params.get("message") {
                                        println!("  Progress: {}", message);
                                    }
                                    if let Some(percentage) = params.get("percentage") {
                                        println!("  Percentage: {}%", percentage);
                                    }
                                }
                            }
                            "log" => {
                                println!("  Log message: {:?}", notification.params);
                            }
                            _ => {
                                println!("  Unknown notification type");
                            }
                        }
                    }
                    _ => {
                        println!("  Unexpected message in async handler: {:?}", msg);
                    }
                }
            }
            println!("Async notification handler ended");
        });
    }
    
    // Start the transport
    println!("Starting transport...");
    transport.start().await?;
    println!("Transport started successfully!");
    
    // Send a test request
    println!("\nSending test request...");
    let request = JsonRpcMessage::Request(JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "test_method".to_string(),
        params: Some(json!({
            "test": "data",
            "number": 42
        })),
        id: Some(1),
    });
    
    match transport.send(request).await {
        Ok(response) => {
            println!("Received response:");
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
        Err(e) => {
            println!("Error sending request: {}", e);
        }
    }
    
    // Wait for notifications
    println!("\nWaiting for server notifications (press Ctrl+C to exit)...");
    tokio::signal::ctrl_c().await?;
    
    // Print final stats
    let final_count = notification_count.load(Ordering::SeqCst);
    println!("\nTotal notifications received: {}", final_count);
    
    // Clean up
    println!("Closing transport...");
    transport.close().await?;
    println!("Client shutdown complete");
    
    Ok(())
}