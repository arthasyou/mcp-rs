use async_trait::async_trait;
use mcp_core::protocol::message::JsonRpcMessage;

use crate::{error::Result, transport::types::MessageHandler};

#[async_trait]
pub trait RequestSender: Send + Sync + 'static {
    async fn send(&self, msg: JsonRpcMessage) -> Result<JsonRpcMessage>;
}

#[async_trait]
pub trait NotifyChannel: Send + Sync + 'static {
    /// 客户端向服务端发送 Notification（不要求返回）
    async fn notify(&self, msg: JsonRpcMessage) -> Result<()>;

    /// 设置处理服务端推送 Notification 的回调函数
    async fn set_message_handler(&self, handler: MessageHandler) -> Result<()>;
}

#[async_trait]
pub trait Connectable: Send + Sync + 'static {
    async fn start(&self) -> Result<()>;
    async fn close(&self) -> Result<()>;
}
