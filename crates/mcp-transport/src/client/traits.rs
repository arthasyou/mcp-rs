use async_trait::async_trait;
use mcp_core::protocol::message::JsonRpcMessage;
use mcp_error::{Error, Result};
use tokio::sync::{mpsc, oneshot};

use crate::client::message::TransportMessage;

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

// Helper function that contains the common send implementation
// pub async fn send_message(
//     sender: &mpsc::Sender<TransportMessage>,
//     message: JsonRpcMessage,
// ) -> Result<()> {
//     match message {
//         JsonRpcMessage::Request(request) => {
//             let (respond_to, response) = oneshot::channel();
//             let msg = TransportMessage {
//                 message: JsonRpcMessage::Request(request),
//                 response_tx: Some(respond_to),
//             };
//             sender.send(msg).await.map_err(|_| Error::ChannelClosed)?;
//             Ok(response.await.map_err(|_| Error::ChannelClosed)?)
//         }
//         JsonRpcMessage::Notification(notification) => {
//             let msg = TransportMessage {
//                 message: JsonRpcMessage::Notification(notification),
//                 response_tx: None,
//             };
//             sender.send(msg).await.map_err(|_| Error::ChannelClosed)?;
//             Ok(JsonRpcMessage::Nil)
//         }
//         _ => Err(Error::UnsupportedMessage),
//     }
// }
