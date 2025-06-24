use mcp_core_rs::protocol::message::{JsonRpcMessage, JsonRpcRequest};
use mcp_error_rs::Result;
use mcp_transport_rs::client::traits::ClientTransport;

pub struct McpClient<T>
where
    T: ClientTransport + Send + Sync + 'static,
{
    pub transport: T,
}

impl<T> McpClient<T>
where
    T: ClientTransport + Send + Sync + 'static,
{
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub async fn send(&self, message: JsonRpcMessage) -> Result<JsonRpcMessage> {
        self.transport.send(message).await
    }

    pub async fn send_resquest(&self, request: JsonRpcRequest) -> Result<JsonRpcMessage> {
        let message = JsonRpcMessage::Request(request);
        self.transport.send(message).await
    }
}
