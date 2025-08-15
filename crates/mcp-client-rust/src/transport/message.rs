use std::collections::HashMap;

use mcp_core::protocol::message::JsonRpcMessage;
use tokio::sync::{RwLock, oneshot};

/// MessageHandler 是一个线程安全的通知消息处理函数类型
pub type MessageHandler = Box<dyn Fn(JsonRpcMessage) + Send + Sync + 'static>;
/// A message that can be sent through the transport
#[derive(Debug)]
pub struct TransportMessage {
    /// The JSON-RPC message to send
    pub message: JsonRpcMessage,
    /// Channel to receive the response on (None for notifications)
    pub response_tx: Option<oneshot::Sender<JsonRpcMessage>>,
}
/// A data structure to store pending requests and their response channels
pub struct PendingRequests {
    requests: RwLock<HashMap<String, oneshot::Sender<JsonRpcMessage>>>,
}

impl Default for PendingRequests {
    fn default() -> Self {
        Self::new()
    }
}

impl PendingRequests {
    pub fn new() -> Self {
        Self {
            requests: RwLock::new(HashMap::new()),
        }
    }

    pub async fn insert(&self, id: String, sender: oneshot::Sender<JsonRpcMessage>) {
        self.requests.write().await.insert(id, sender);
    }

    pub async fn respond(&self, id: &str, response: JsonRpcMessage) {
        if let Some(tx) = self.requests.write().await.remove(id) {
            let _ = tx.send(response);
        }
    }

    pub async fn clear(&self) {
        self.requests.write().await.clear();
    }
}
