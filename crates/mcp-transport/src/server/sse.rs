use std::io;

use async_trait::async_trait;
use mcp_core::protocol::message::JsonRpcMessage;
use mcp_error::{Error, Result};
use tokio::sync::mpsc::UnboundedSender;

use crate::server::traits::ServerTransport;

/// A simple serializable message for SSE-compatible transport
#[derive(Debug, Clone)]
pub struct SseMessage {
    pub data: String,
}

impl From<String> for SseMessage {
    fn from(data: String) -> Self {
        SseMessage { data }
    }
}

/// A transport that wraps an SSE-style message sender
pub struct SseTransport {
    sender: UnboundedSender<SseMessage>,
}

impl SseTransport {
    pub fn new(sender: UnboundedSender<SseMessage>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl ServerTransport for SseTransport {
    async fn write_message(&mut self, msg: JsonRpcMessage) -> Result<()> {
        let json = serde_json::to_string(&msg).map_err(|e| Error::Json(e))?;
        let message = SseMessage::from(json);
        self.sender
            .send(message)
            .map_err(|_| Error::Io(io::Error::new(io::ErrorKind::BrokenPipe, "SSE send failed")))
    }

    async fn read_message(&mut self) -> Option<Result<JsonRpcMessage>> {
        // SSE is write-only on the server side
        None
    }
}
