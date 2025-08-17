use mcp_core::protocol::message::JsonRpcNotification;

use crate::{
    error::Result,
    transport::{
        impls::sse::{SseConfig, SseTransport},
        types::{ConnectionState, MessageReceiver, TransportType},
    },
};

/// 服务端推送的通知处理器
pub type NotificationHandler = Box<dyn Fn(JsonRpcNotification) + Send + Sync>;

/// 统一的 Transport 枚举
pub enum Transport {
    Sse(SseTransport),
}

impl Transport {
    pub fn new_sse(url: impl Into<String>, config: Option<SseConfig>) -> Self {
        let sse_transport = match config {
            Some(config) => SseTransport::with_config(url, config),
            None => SseTransport::new(url),
        };
        Transport::Sse(sse_transport)
    }
}

impl Transport {
    /// 获取 Transport 类型
    pub fn transport_type(&self) -> TransportType {
        TransportType::Sse
    }

    /// 获取当前状态
    pub async fn state(&self) -> ConnectionState {
        match self {
            Transport::Sse(sse) => sse.get_state().await,
        }
    }

    /// 检查是否支持通知
    pub fn supports_notifications(&self) -> bool {
        true
    }

    /// 检查是否需要建立连接
    pub fn requires_connection(&self) -> bool {
        true
    }

    /// 建立连接（如果需要）
    pub async fn connect(&mut self) -> Result<()> {
        match self {
            Transport::Sse(sse) => sse.connect().await,
        }
    }

    /// 断开连接
    pub async fn disconnect(&mut self) -> Result<()> {
        match self {
            Transport::Sse(sse) => sse.disconnect().await,
        }
    }

    /// 发送通知（不等待响应）
    pub async fn send_message(&mut self, notification: JsonRpcNotification) -> Result<()> {
        match self {
            Transport::Sse(sse) => sse.send_message(notification).await,
        }
    }

    /// 获取消息接收器（如果支持）
    pub fn take_message_receiver(&mut self) -> Option<MessageReceiver> {
        match self {
            Transport::Sse(sse) => sse.take_message_receiver(),
        }
    }
}
