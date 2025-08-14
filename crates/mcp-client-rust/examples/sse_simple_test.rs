use mcp_client_rust::transport::impls::sse::SseTransport;
use mcp_client_rust::transport::traits::ClientTransport;
use mcp_core::protocol::message::{JsonRpcMessage, JsonRpcRequest};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SSE Transport Test ===\n");
    
    // Initialize SSE transport
    let transport = SseTransport::new("http://localhost:3000/sse");
    
    // Set up async notification handler
    println!("Setting up async notification handler...");
    if let Some(mut receiver) = transport.take_message_receiver().await {
        tokio::spawn(async move {
            println!("[ASYNC] Handler started, waiting for notifications...");
            while let Some(msg) = receiver.recv().await {
                match msg {
                    JsonRpcMessage::Notification(notification) => {
                        println!("[ASYNC] Notification: method={}, params={}", 
                            notification.method, 
                            serde_json::to_string(&notification.params).unwrap_or_default()
                        );
                    }
                    _ => {
                        println!("[ASYNC] Non-notification message: {:?}", msg);
                    }
                }
            }
        });
    }
    
    // Also set up sync handler
    transport.set_message_handler(|msg| {
        match &msg {
            JsonRpcMessage::Notification(notification) => {
                println!("[SYNC] Notification: method={}, params={}", 
                    notification.method,
                    serde_json::to_string(&notification.params).unwrap_or_default()
                );
            }
            _ => {
                println!("[SYNC] Non-notification message: {:?}", msg);
            }
        }
    }).await;
    
    // Start the transport
    println!("\nStarting transport...");
    match transport.start().await {
        Ok(_) => println!("Transport started successfully!"),
        Err(e) => {
            println!("Failed to start transport: {}", e);
            return Err(e.into());
        }
    }
    
    // Send a test request
    println!("\nSending test request...");
    let request = JsonRpcMessage::Request(JsonRpcRequest::new(
        Some(1),
        "test_method",
        Some(json!({
            "message": "Hello from client!"
        }))
    ));
    
    match transport.send(request).await {
        Ok(response) => {
            println!("Response received: {}", serde_json::to_string_pretty(&response)?);
        }
        Err(e) => {
            println!("Error sending request: {}", e);
        }
    }
    
    // Wait for notifications
    println!("\nWaiting for notifications (10 seconds)...");
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    
    // Clean up
    println!("\nShutting down...");
    transport.close().await?;
    println!("Done!");
    
    Ok(())
}