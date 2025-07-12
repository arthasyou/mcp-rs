use std::io;

use async_trait::async_trait;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    core::protocol::message::JsonRpcMessage,
    error::{Error, Result},
    transport::traits::ServerTransport,
};

/// A transport that wraps an SSE-style message sender
pub struct SseTransport {
    sender: UnboundedSender<JsonRpcMessage>,
    receiver: UnboundedReceiver<JsonRpcMessage>,
}

impl SseTransport {
    pub fn new(
        sender: UnboundedSender<JsonRpcMessage>,
        receiver: UnboundedReceiver<JsonRpcMessage>,
    ) -> Self {
        Self { sender, receiver }
    }
}

#[async_trait]
impl ServerTransport for SseTransport {
    async fn write_message(&mut self, msg: JsonRpcMessage) -> Result<()> {
        self.sender
            .send(msg)
            .map_err(|_| Error::Io(io::Error::new(io::ErrorKind::BrokenPipe, "SSE send failed")))
    }

    async fn read_message(&mut self) -> Option<Result<JsonRpcMessage>> {
        self.receiver.recv().await.map(Ok)
    }
}
