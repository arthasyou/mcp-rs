use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;

use crate::{
    core::{
        Tool,
        protocol::{
            message::{JsonRpcMessage, JsonRpcRequest},
            result::InitializeResult,
        },
    },
    error::Result,
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

    pub async fn initialize(&self) -> Result<InitializeResult> {
        let request = JsonRpcRequest::new(Some(self.next_id()), "initialize", None);
        let response = self.send_resquest(request).await?.try_into()?;
        Ok(response)
    }

    pub async fn get_tools(&self) -> Result<Vec<Tool>> {
        let request = JsonRpcRequest::new(Some(self.next_id()), "tools/list", None);
        let response = self.send_resquest(request).await?.try_into()?;
        Ok(response)
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
