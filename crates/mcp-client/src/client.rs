use mcp_core::protocol::message::JsonRpcMessage;
use mcp_error::Result;
use mcp_transport::client::traits::ClientTransport;

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

    pub async fn send_request(&mut self, request: JsonRpcMessage) -> Result<JsonRpcMessage> {
        self.transport.send(request).await
    }
}
