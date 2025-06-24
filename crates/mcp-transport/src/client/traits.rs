use async_trait::async_trait;
use mcp_core_rs::protocol::message::JsonRpcMessage;
use mcp_error_rs::Result;

/// A generic asynchronous transport trait with channel-based communication
#[async_trait]
pub trait ClientTransport: Send + Sync + 'static {
    /// Start the transport and establish the underlying connection.
    /// Returns an instance implementing the same trait.
    async fn start(&self) -> Result<()>;

    /// Close the transport and free any resources.
    async fn close(&self) -> Result<()>;

    /// Send a JSON-RPC message and await a response.
    async fn send(&self, message: JsonRpcMessage) -> Result<JsonRpcMessage>;
}
