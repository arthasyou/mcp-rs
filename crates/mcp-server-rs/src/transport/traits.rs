use async_trait::async_trait;

use crate::{core::protocol::message::JsonRpcMessage, error::Result};

#[async_trait]
pub trait ServerTransport: Send + Sync {
    /// Reads a JSON-RPC message (could be a Request or Notification)
    async fn read_message(&mut self) -> Option<Result<JsonRpcMessage>>;

    /// Sends a JSON-RPC message (usually a Response)
    async fn write_message(&mut self, msg: JsonRpcMessage) -> Result<()>;

    /// Closes the transport connection
    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}
