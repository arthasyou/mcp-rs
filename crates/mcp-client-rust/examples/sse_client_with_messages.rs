use mcp_client_rust::transport::impls::sse::SseTransport;
use mcp_core::protocol::{
    message::{JsonRpcMessage, JsonRpcNotification},
    constants::JSONRPC_VERSION_FIELD,
};
use serde_json::json;
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mcp_client_rust=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting SSE MCP client example with messages");

    // 连接到 mcp-service 的 counter 服务
    // 请确保服务端在运行：cargo run -p mcp-service
    let url = "http://localhost:3000/sse?service=counter";
    let mut transport = SseTransport::new(url);
    
    info!("Connecting to: {}", url);
    match transport.connect().await {
        Ok(_) => info!("Connected to SSE service successfully"),
        Err(e) => {
            error!("Failed to connect: {}", e);
            error!("请确保服务端在运行。启动命令：");
            error!("cd /Users/ancient/src/rust/mcp-service && cargo run");
            return Err(e.into());
        }
    }
    
    // 获取消息接收器来处理服务端推送的消息
    let receiver = transport.take_message_receiver()
        .ok_or("Failed to get message receiver")?;
    
    // 启动一个任务来处理接收到的消息
    let handle = tokio::spawn(async move {
        handle_incoming_messages(receiver).await;
    });
    
    // 发送一些测试消息
    info!("Sending test messages...");
    
    // 发送一个简单的通知
    let notification1 = JsonRpcNotification {
        jsonrpc: JSONRPC_VERSION_FIELD.to_string(),
        method: "test/hello".to_string(),
        params: Some(json!({
            "message": "Hello from SSE client",
            "timestamp": chrono::Utc::now().to_rfc3339()
        })),
    };
    
    match transport.send_message(notification1).await {
        Ok(_) => info!("Sent hello notification"),
        Err(e) => error!("Failed to send hello notification: {}", e),
    }
    
    // 等待一会儿
    sleep(Duration::from_secs(2)).await;
    
    // 发送 counter 相关的通知（如果 counter 服务支持的话）
    let notification2 = JsonRpcNotification {
        jsonrpc: JSONRPC_VERSION_FIELD.to_string(),
        method: "counter/increment".to_string(),
        params: Some(json!({
            "value": 1
        })),
    };
    
    match transport.send_message(notification2).await {
        Ok(_) => info!("Sent counter increment notification"),
        Err(e) => error!("Failed to send counter notification: {}", e),
    }
    
    // 等待更多消息
    info!("Waiting for messages... Press Ctrl+C to exit");
    
    // 等待用户中断或者30秒后自动退出
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("User interrupted");
        }
        _ = sleep(Duration::from_secs(30)) => {
            info!("Timeout reached");
        }
    }
    
    // 断开连接
    info!("Disconnecting...");
    transport.disconnect().await?;
    
    // 等待接收任务结束
    handle.await?;
    
    info!("Example completed");
    Ok(())
}

async fn handle_incoming_messages(mut receiver: mcp_client_rust::transport::types::MessageReceiver) {
    info!("Message handler started");
    
    while let Some(msg) = receiver.recv().await {
        match msg {
            JsonRpcMessage::Notification(notification) => {
                info!("📨 Received notification:");
                info!("   Method: {}", notification.method);
                info!("   Params: {}", 
                    notification.params
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "None".to_string())
                );
            }
            JsonRpcMessage::Request(request) => {
                info!("📥 Received request:");
                info!("   ID: {:?}", request.id);
                info!("   Method: {}", request.method);
                info!("   Params: {}", 
                    request.params
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "None".to_string())
                );
            }
            JsonRpcMessage::Response(response) => {
                info!("📤 Received response:");
                info!("   ID: {:?}", response.id);
                if let Some(result) = response.result {
                    info!("   Result: {}", result);
                }
                if let Some(error) = response.error {
                    warn!("   Error: {:?}", error);
                }
            }
            JsonRpcMessage::Error(error) => {
                error!("❌ Received error:");
                error!("   ID: {:?}", error.id);
                error!("   Error: {:?}", error.error);
            }
            JsonRpcMessage::Nil => {
                info!("Received Nil message");
            }
        }
    }
    
    info!("Message handler ended");
}