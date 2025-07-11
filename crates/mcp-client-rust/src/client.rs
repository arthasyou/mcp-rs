use std::sync::atomic::{AtomicU64, Ordering};

use mcp_core_rs::Tool;
use serde_json::json;

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
    id_counter: AtomicU64,
}

impl<T> McpClient<T>
where
    T: ClientTransport + Send + Sync + 'static,
{
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            id_counter: AtomicU64::new(1),
        }
    }

    fn next_id(&self) -> u64 {
        self.id_counter.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn send(&self, message: JsonRpcMessage) -> Result<JsonRpcMessage> {
        self.transport.send(message).await
    }

    pub async fn send_resquest(&self, request: JsonRpcRequest) -> Result<JsonRpcMessage> {
        let message = JsonRpcMessage::Request(request);
        self.transport.send(message).await
    }

    pub async fn initialize(&self) -> Result<JsonRpcMessage> {
        let request = JsonRpcRequest::new(Some(self.next_id()), "initialize", None);
        self.send_resquest(request).await
    }

    pub async fn get_tools(&self) -> Result<Vec<Tool>> {
        let request = JsonRpcRequest::new(Some(self.next_id()), "tools/list", None);
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

    pub async fn call_tool(&self, params: serde_json::Value) -> Result<JsonRpcMessage> {
        let request = JsonRpcRequest::new(Some(self.next_id()), "tools/call", Some(params));
        self.send_resquest(request).await
    }

    pub async fn list_resources(&self) -> Result<JsonRpcMessage> {
        let request = JsonRpcRequest::new(Some(self.next_id()), "resources/list", None);
        self.send_resquest(request).await
    }

    pub async fn read_resource(&self, resource_id: &str) -> Result<JsonRpcMessage> {
        let params = json!({ "id": resource_id });
        let request = JsonRpcRequest::new(Some(self.next_id()), "resources/read", Some(params));
        self.send_resquest(request).await
    }

    pub async fn list_prompts(&self) -> Result<JsonRpcMessage> {
        let request = JsonRpcRequest::new(Some(self.next_id()), "prompts/list", None);
        self.send_resquest(request).await
    }

    pub async fn get_prompt(&self, name: &str) -> Result<JsonRpcMessage> {
        let params = json!({ "name": name });
        let request = JsonRpcRequest::new(Some(self.next_id()), "prompts/get", Some(params));
        self.send_resquest(request).await
    }
}
