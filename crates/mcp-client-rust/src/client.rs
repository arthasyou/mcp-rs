use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

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
    transport::traits::{Connectable, NotifyChannel, RequestSender},
};

pub struct McpClient {
    sender: Option<Arc<dyn RequestSender>>,
    notifier: Option<Arc<dyn NotifyChannel>>,
    connection: Option<Arc<dyn Connectable>>,
    id_counter: AtomicU64,
}

impl McpClient {
    pub fn new() -> Self {
        Self {
            sender: None,
            notifier: None,
            connection: None,
            id_counter: AtomicU64::new(1),
        }
    }

    pub fn with_request_sender(mut self, sender: Arc<dyn RequestSender>) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn with_notify_channel(mut self, notifier: Arc<dyn NotifyChannel>) -> Self {
        self.notifier = Some(notifier);
        self
    }

    pub fn with_connectable(mut self, connection: Arc<dyn Connectable>) -> Self {
        self.connection = Some(connection);
        self
    }
}

impl McpClient {
    fn next_id(&self) -> u64 {
        self.id_counter.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn send(&self, message: JsonRpcMessage) -> Result<JsonRpcMessage> {
        match &self.sender {
            Some(sender) => sender.send(message).await,
            None => Err(crate::error::Error::System(
                "RequestSender not available".into(),
            )),
        }
    }

    pub async fn send_resquest(&self, request: JsonRpcRequest) -> Result<JsonRpcMessage> {
        let message = JsonRpcMessage::Request(request);
        self.send(message).await
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
