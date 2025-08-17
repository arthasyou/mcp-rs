use std::time::Duration;

use mcp_client_rust::{
    transport::{
        impls::sse::{SseConfig, SseTransport},
        traits::{Connectable, NotifyChannel, RequestSender},
        types::MessageHandler,
    },
};
use mcp_core::protocol::{
    message::{JsonRpcMessage, JsonRpcNotification},
    constants::JSONRPC_VERSION_FIELD,
};
use serde_json::json;
use tokio::time::sleep;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// 定义transport wrapper来实现所需的trait
struct SseTransportWrapper {
    transport: SseTransport,
}

impl SseTransportWrapper {
    fn new(transport: SseTransport) -> Self {
        Self { transport }
    }
}

#[async_trait::async_trait]
impl RequestSender for SseTransportWrapper {
    async fn send(&self, msg: JsonRpcMessage) -> mcp_client_rust::error::Result<JsonRpcMessage> {
        // SSE transport目前只支持单向通信（服务端到客户端）
        // 对于请求-响应模式，需要使用HTTP POST endpoint
        match msg {
            JsonRpcMessage::Request(req) => {
                // 发送请求到服务端
                let notification = JsonRpcNotification {
                    jsonrpc: JSONRPC_VERSION_FIELD.to_string(),
                    method: "request".to_string(),
                    params: Some(serde_json::to_value(&req).unwrap()),
                };
                self.transport.send_message(notification).await?;
                
                // 注意：在实际应用中，这里需要实现一个机制来等待响应
                // 可能需要使用channel或其他同步机制
                Err(mcp_client_rust::error::Error::System(
                    "SSE transport doesn't support request-response pattern directly".to_string()
                ))
            }
            _ => Err(mcp_client_rust::error::Error::System(
                "Invalid message type for request".to_string()
            ))
        }
    }
}

#[async_trait::async_trait]
impl NotifyChannel for SseTransportWrapper {
    async fn notify(&self, msg: JsonRpcMessage) -> mcp_client_rust::error::Result<()> {
        match msg {
            JsonRpcMessage::Notification(notification) => {
                self.transport.send_message(notification).await
            }
            _ => Err(mcp_client_rust::error::Error::System(
                "Invalid message type for notification".to_string()
            ))
        }
    }

    async fn set_message_handler(&self, handler: MessageHandler) -> mcp_client_rust::error::Result<()> {
        // 在实际实现中，这里需要设置处理来自服务端的消息的回调
        // 可以通过启动一个任务来处理 message_receiver
        Ok(())
    }
}

#[async_trait::async_trait]
impl Connectable for SseTransportWrapper {
    async fn start(&self) -> mcp_client_rust::error::Result<()> {
        // Transport已经在new时初始化，这里不需要额外操作
        Ok(())
    }

    async fn close(&self) -> mcp_client_rust::error::Result<()> {
        // 关闭操作需要在transport上实现
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting SSE MCP client example");

    // 创建SSE transport配置
    let config = SseConfig {
        initial_retry_interval: Duration::from_secs(1),
        max_retry_interval: Duration::from_secs(30),
        connection_timeout: Duration::from_secs(10),
        exponential_backoff: true,
        max_retries: Some(5),
        shutdown_timeout: Duration::from_secs(5),
    };

    // 连接到服务端 - 使用counter服务作为示例
    let service_url = "http://localhost:18000/sse?service=counter";
    info!("Connecting to SSE service: {}", service_url);

    // 创建transport
    let mut transport = SseTransport::with_config(service_url, config);
    
    // 连接到服务端
    transport.connect().await?;
    info!("Connected to SSE service");

    // 获取消息接收器
    let mut receiver = transport.take_message_receiver()
        .ok_or("Failed to get message receiver")?;

    // 启动消息处理任务
    let handle = tokio::spawn(async move {
        info!("Starting message receiver task");
        while let Some(msg) = receiver.recv().await {
            match msg {
                JsonRpcMessage::Notification(notification) => {
                    info!("Received notification: method={}, params={:?}", 
                        notification.method, notification.params);
                }
                JsonRpcMessage::Request(request) => {
                    info!("Received request: id={:?}, method={}, params={:?}", 
                        request.id, request.method, request.params);
                }
                JsonRpcMessage::Response(response) => {
                    info!("Received response: id={:?}, result={:?}, error={:?}", 
                        response.id, response.result, response.error);
                }
                JsonRpcMessage::Error(error) => {
                    error!("Received error: id={:?}, error={:?}", 
                        error.id, error.error);
                }
                JsonRpcMessage::Nil => {
                    info!("Received Nil message");
                }
            }
        }
        warn!("Message receiver task ended");
    });

    // 发送一些示例通知
    info!("Sending test notifications...");
    
    // 发送 roots/changed 通知
    let roots_changed = JsonRpcNotification {
        jsonrpc: JSONRPC_VERSION_FIELD.to_string(),
        method: "notifications/roots/changed".to_string(),
        params: Some(json!({})),
    };
    
    if let Err(e) = transport.send_message(roots_changed).await {
        error!("Failed to send roots/changed notification: {}", e);
    } else {
        info!("Sent roots/changed notification");
    }

    // 等待一段时间以接收消息
    info!("Waiting for messages...");
    sleep(Duration::from_secs(10)).await;

    // 发送另一个通知
    let custom_notification = JsonRpcNotification {
        jsonrpc: JSONRPC_VERSION_FIELD.to_string(),
        method: "custom/test".to_string(),
        params: Some(json!({
            "message": "Hello from SSE client",
            "timestamp": chrono::Utc::now().to_rfc3339()
        })),
    };
    
    if let Err(e) = transport.send_message(custom_notification).await {
        error!("Failed to send custom notification: {}", e);
    } else {
        info!("Sent custom notification");
    }

    // 再等待一段时间
    sleep(Duration::from_secs(5)).await;

    // 断开连接
    info!("Disconnecting from SSE service");
    transport.disconnect().await?;

    // 等待消息处理任务结束
    let _ = handle.await;

    info!("SSE MCP client example completed");
    Ok(())
}