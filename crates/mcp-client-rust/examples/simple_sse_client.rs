use std::time::Duration;

use mcp_client_rust::transport::impls::sse::SseTransport;
use tokio::time::sleep;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting SSE MCP client example");

    // 连接到 mcp-service 的 counter 服务
    let url = "http://localhost:18000/sse?service=counter";
    let mut transport = SseTransport::new(url);

    info!("Connecting to: {}", url);
    transport.connect().await?;
    info!("Connected to SSE service");

    // 获取消息接收器来处理服务端推送的消息
    if let Some(mut receiver) = transport.take_message_receiver() {
        // 启动一个任务来处理接收到的消息
        let handle = tokio::spawn(async move {
            info!("Starting message receiver task");
            while let Some(msg) = receiver.recv().await {
                info!("Received message: {:?}", msg);
            }
            info!("Message receiver task ended");
        });

        // 等待一段时间来接收消息
        info!("Waiting for messages...");
        sleep(Duration::from_secs(30)).await;

        // 断开连接
        info!("Disconnecting...");
        transport.disconnect().await?;

        // 等待接收任务结束
        handle.await?;
    }

    info!("Example completed");
    Ok(())
}
