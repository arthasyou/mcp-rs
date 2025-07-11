use mcp_client_rs::{
    client::McpClient,
    transport::{impls::sse::SseTransport, traits::ClientTransport},
};
use mcp_core_rs::protocol::message::JsonRpcRequest;
use serde_json::json;
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(true)
                .with_line_number(true),
        )
        .init();

    let transport = SseTransport::new("http://localhost:18000/sse?service=chart");
    transport.start().await.unwrap();
    debug!("Starting MCP Client Example");
    // sleep(std::time::Duration::from_secs(1)); // Wait for the transport to start
    let client = McpClient::new(transport);
    let message = JsonRpcRequest::new(
        Some(1),
        "tools/call",
        Some(json!({
            "name": "generate_chart",
            "arguments": {
                "chart_type": "bar",
                "title": "Sales Data",
                "labels": ["Q1", "Q2", "Q3", "Q4"],
                "values": [100.0, 150.0, 200.0, 250.0]
            }
        })),
    );

    let response = client.send_resquest(message).await;
    debug!(" ====== example Response: {:?}", response);

    // println!("shutting down transport...");
    // client.transport.close().await.unwrap();

    tokio::signal::ctrl_c().await.unwrap();
}
