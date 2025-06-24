use std::thread::sleep;

use mcp_client::client::McpClient;
use mcp_core::protocol::message::{JsonRpcMessage, JsonRpcRequest};
use mcp_transport::client::{impls::sse::SseTransport, traits::ClientTransport};
use serde_json::json;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let transport = SseTransport::new("http://localhost:18000/sse");
    transport.start().await.unwrap();
    sleep(std::time::Duration::from_secs(1)); // Wait for the transport to start
    let mut client = McpClient::new(transport);
    let message = JsonRpcMessage::Request(JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "increment",
            "arguments": {}
        })),
    });

    let response = client.send_request(message).await;
    println!(" ====== example Response: {:?}", response);

    println!("shutting down transport...");
    client.transport.close().await.unwrap();

    sleep(std::time::Duration::from_secs(2));
}
