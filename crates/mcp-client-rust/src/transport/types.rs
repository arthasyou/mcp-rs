use mcp_core::protocol::message::JsonRpcMessage;
use tokio::sync::mpsc;

/// MessageHandler 是一个线程安全的通知消息处理函数类型
pub type MessageHandler = Box<dyn Fn(JsonRpcMessage) + Send + Sync + 'static>;

/// 消息发送通道类型
pub type MessageSender = mpsc::UnboundedSender<JsonRpcMessage>;

/// 消息接收通道类型
pub type MessageReceiver = mpsc::UnboundedReceiver<JsonRpcMessage>;

/// Transport 状态
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Closing,
    Closed,
}

/// Transport 类型标识
#[derive(Debug, Clone, PartialEq)]
pub enum TransportType {
    Http,
    Sse,
    WebSocket,
    Stdio,
}
