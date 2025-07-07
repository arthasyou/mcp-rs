use mcp_core_rs::Tool;

use crate::{
    core::protocol::message::{JsonRpcMessage, JsonRpcRequest},
    error::{Error, Result},
    transport::traits::ClientTransport,
};

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

    pub async fn get_tools(&self, id: Option<u64>) -> Result<Vec<Tool>> {
        let request = JsonRpcRequest::new(id, "tools/list", None);
        let response = self.send_resquest(request).await?;

        match response {
            JsonRpcMessage::Response(result) => {
                if let Some(tools_value) = result.result {
                    if let Some(tools_array) = tools_value.get("tools") {
                        if let Ok(tools_vec) =
                            serde_json::from_value::<Vec<Tool>>(tools_array.clone())
                        {
                            return Ok(tools_vec);
                        }
                    }
                }
                Err(Error::System("Failed to parse tools".into()))
            }
            _ => Err(Error::System("Unexpected response type".into())),
        }
    }
}
