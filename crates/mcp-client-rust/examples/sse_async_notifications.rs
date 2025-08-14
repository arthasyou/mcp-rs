use mcp_client_rust::transport::impls::sse::SseTransport;
use mcp_client_rust::transport::traits::ClientTransport;
use mcp_core::protocol::message::JsonRpcMessage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize SSE transport
    let transport = SseTransport::new("http://localhost:3000/sse");
    
    // Method 1: Set a synchronous callback handler
    transport.set_message_handler(|msg| {
        println!("Received notification (sync handler): {:?}", msg);
    }).await;
    
    // Method 2: Get the async receiver for handling messages in a separate task
    if let Some(mut receiver) = transport.take_message_receiver().await {
        tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                match msg {
                    JsonRpcMessage::Notification(notification) => {
                        println!("Received notification (async): method={}, params={:?}", 
                            notification.method, 
                            notification.params
                        );
                        // Handle different notification types
                        match notification.method.as_str() {
                            "progress" => {
                                // Handle progress updates
                            }
                            "log" => {
                                // Handle log messages
                            }
                            _ => {
                                // Handle other notifications
                            }
                        }
                    }
                    _ => {
                        // This shouldn't happen as only notifications are sent through this channel
                    }
                }
            }
        });
    }
    
    // Start the transport
    transport.start().await?;
    
    // Now you can use the transport for synchronous request/response
    // while also receiving async notifications
    
    // Example: Send a request and wait for response synchronously
    // let request = create_json_rpc_request("someMethod", params);
    // let response = transport.send(request).await?;
    
    // Keep the program running to receive notifications
    tokio::signal::ctrl_c().await?;
    
    // Clean up
    transport.close().await?;
    
    Ok(())
}