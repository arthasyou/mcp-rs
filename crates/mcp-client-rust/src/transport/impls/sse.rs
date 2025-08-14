use std::sync::Arc;

use async_trait::async_trait;
use eventsource_client::{Client as SseClient, SSE};
use futures::TryStreamExt;
use mcp_core::protocol::message::JsonRpcMessage;
use serde_json::{self};
use service_utils_rs::utils::Request;
use tokio::{
    spawn,
    sync::{RwLock, oneshot, mpsc},
};
use tokio_util::sync::CancellationToken;
use tracing::{self, warn};
use url::Url;

use crate::{
    error::{Error, Result},
    transport::{message::PendingRequests, traits::ClientTransport},
};

/// Callback type for handling server-pushed messages (notifications)
pub type MessageHandler = Box<dyn Fn(JsonRpcMessage) + Send + Sync>;

/// SSE Transport implementation that supports both synchronous request/response
/// and asynchronous server-pushed notifications.
/// 
/// # Examples
/// 
/// ## Synchronous callback handler
/// ```no_run
/// # use mcp_client_rust::transport::impls::sse::SseTransport;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let transport = SseTransport::new("http://localhost:3000/sse");
/// transport.set_message_handler(|msg| {
///     println!("Received notification: {:?}", msg);
/// }).await;
/// # Ok(())
/// # }
/// ```
/// 
/// ## Asynchronous message receiver
/// ```no_run
/// # use mcp_client_rust::transport::impls::sse::SseTransport;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let transport = SseTransport::new("http://localhost:3000/sse");
/// if let Some(mut receiver) = transport.take_message_receiver().await {
///     tokio::spawn(async move {
///         while let Some(msg) = receiver.recv().await {
///             // Handle server-pushed notifications
///         }
///     });
/// }
/// # Ok(())
/// # }
/// ```
pub struct SseTransport {
    sse_url: String,
    post_endpoint: Arc<RwLock<Option<String>>>,
    pending_requests: Arc<PendingRequests>,
    request: Request,
    shutdown: CancellationToken,
    /// Optional handler for server-pushed messages (notifications)
    message_handler: Arc<RwLock<Option<MessageHandler>>>,
    /// Channel for receiving server-pushed messages asynchronously
    message_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<JsonRpcMessage>>>>,
    message_sender: mpsc::UnboundedSender<JsonRpcMessage>,
}

impl SseTransport {
    pub fn new(sse_url: &str) -> Self {
        let mut request = Request::new();
        request
            .set_default_headers(vec![("Content-Type", "application/json".to_string())])
            .unwrap();
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            sse_url: sse_url.to_owned(),
            post_endpoint: Arc::new(RwLock::new(None)),
            pending_requests: Arc::new(PendingRequests::default()),
            request,
            shutdown: CancellationToken::new(),
            message_handler: Arc::new(RwLock::new(None)),
            message_receiver: Arc::new(RwLock::new(Some(rx))),
            message_sender: tx,
        }
    }

    /// Set a handler for server-pushed messages (notifications)
    pub async fn set_message_handler<F>(&self, handler: F)
    where
        F: Fn(JsonRpcMessage) + Send + Sync + 'static,
    {
        *self.message_handler.write().await = Some(Box::new(handler));
    }

    /// Remove the message handler
    pub async fn clear_message_handler(&self) {
        *self.message_handler.write().await = None;
    }

    /// Get a receiver for server-pushed messages (notifications)
    /// This allows async handling of messages
    pub async fn take_message_receiver(&self) -> Option<mpsc::UnboundedReceiver<JsonRpcMessage>> {
        self.message_receiver.write().await.take()
    }

    /// Get a clone of the message sender (for sending notifications to receivers)
    pub fn get_message_sender(&self) -> mpsc::UnboundedSender<JsonRpcMessage> {
        self.message_sender.clone()
    }

    async fn get_post_endpoint(&self) -> Result<String> {
        let guard = self.post_endpoint.read().await;
        match &*guard {
            Some(endpoint) => Ok(endpoint.clone()),
            None => Err(Error::System(
                "POST endpoint not discovered yet".to_string(),
            )),
        }
    }

    async fn wait_until_post_endpoint_ready(&self) -> Result<String> {
        loop {
            if let Ok(endpoint) = self.get_post_endpoint().await {
                return Ok(endpoint);
            }
            // sleep 可以防止 busy loop
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }
}

#[async_trait]
impl ClientTransport for SseTransport {
    async fn start(&self) -> Result<()> {
        let sse_url = self.sse_url.clone();
        let pending_requests = self.pending_requests.clone();
        let post_endpoint = self.post_endpoint.clone();
        let shutdown = self.shutdown.clone();
        let message_handler = self.message_handler.clone();
        let message_sender = self.message_sender.clone();
        spawn(async move {
            tokio::select! {
                _ = handle_messages_loop(sse_url, pending_requests, post_endpoint, message_handler, message_sender, shutdown.clone()) => {},
                _ = shutdown.cancelled() => {},
            }
        });

        self.wait_until_post_endpoint_ready().await?;

        Ok(())
    }

    async fn send(&self, message: JsonRpcMessage) -> Result<JsonRpcMessage> {
        let post_url = &self.get_post_endpoint().await?;
        let id = match &message {
            JsonRpcMessage::Request(req) => req.id.as_ref(),
            JsonRpcMessage::Error(err) => err.id.as_ref(),
            _ => None,
        }
        .ok_or_else(|| Error::System("Message missing id".to_string()))?;

        let (tx, rx) = oneshot::channel();
        self.pending_requests.insert(id.to_string(), tx).await;

        let _response = self
            .request
            .post(post_url, &serde_json::to_value(&message)?, None)
            .await
            .map_err(|e| {
                warn!("Failed to send message: {}", e);
                Error::System(e.to_string())
            })?;

        // 等待响应通过 SSE 返回
        let response_msg = rx
            .await
            .map_err(|_| Error::System("No response received".to_string()))?;
        Ok(response_msg)
    }

    async fn close(&self) -> Result<()> {
        self.shutdown.cancel();
        Ok(())
    }
}

impl Drop for SseTransport {
    fn drop(&mut self) {
        self.shutdown.cancel();
    }
}

async fn handle_messages_loop(
    sse_url: String,
    pending_requests: Arc<PendingRequests>,
    post_endpoint: Arc<RwLock<Option<String>>>,
    message_handler: Arc<RwLock<Option<MessageHandler>>>,
    message_sender: mpsc::UnboundedSender<JsonRpcMessage>,
    shutdown: CancellationToken,
) {
    loop {
        let result = handle_messages_once(
            sse_url.clone(),
            pending_requests.clone(),
            post_endpoint.clone(),
            message_handler.clone(),
            message_sender.clone(),
            shutdown.clone(),
        )
        .await;

        if shutdown.is_cancelled() {
            break;
        }

        if let Err(e) = result {
            warn!("SSE connection failed: {e}, retrying in 3s...");
        } else {
            warn!("SSE handler exited unexpectedly, retrying in 3s...");
        }

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

async fn handle_messages_once(
    sse_url: String,
    pending_requests: Arc<PendingRequests>,
    post_endpoint: Arc<RwLock<Option<String>>>,
    message_handler: Arc<RwLock<Option<MessageHandler>>>,
    message_sender: mpsc::UnboundedSender<JsonRpcMessage>,
    shutdown: CancellationToken,
) -> Result<()> {
    let client = match eventsource_client::ClientBuilder::for_url(&sse_url) {
        Ok(builder) => builder.build(),
        Err(e) => {
            pending_requests.clear().await;
            warn!("Failed to connect SSE client: {}", e);
            return Ok(());
        }
    };
    let mut stream = client.stream();

    // First, wait for the "endpoint" event
    while let Ok(Some(event)) = stream.try_next().await {
        match event {
            SSE::Event(e) if e.event_type == "endpoint" => {
                // SSE server uses the "endpoint" event to tell us the POST URL
                let base_url = Url::parse(&sse_url).expect("Invalid base URL");
                let post_url = base_url
                    .join(&e.data)
                    .expect("Failed to resolve endpoint URL");

                tracing::debug!("Discovered SSE POST endpoint: {}", post_url);
                *post_endpoint.write().await = Some(post_url.to_string());
                break;
            }
            _ => continue,
        }
    }

    // Now handle subsequent events
    loop {
        tokio::select! {
            maybe_event = stream.try_next() => {
                match maybe_event {
                    Ok(Some(SSE::Event(e))) if e.event_type == "message" => {
                        match serde_json::from_str::<JsonRpcMessage>(&e.data) {
                            Ok(message) => {
                                match &message {
                                    JsonRpcMessage::Response(response) => {
                                        if let Some(id) = &response.id {
                                            pending_requests.respond(&id.to_string(), message).await;
                                        }
                                    }
                                    JsonRpcMessage::Error(error) => {
                                        if let Some(id) = &error.id {
                                            pending_requests.respond(&id.to_string(), message).await;
                                        }
                                    }
                                    JsonRpcMessage::Notification(_) => {
                                        // Handle server-pushed notifications asynchronously
                                        // First, call the sync handler if set
                                        let handler_guard = message_handler.read().await;
                                        if let Some(handler) = handler_guard.as_ref() {
                                            handler(message.clone());
                                        }
                                        
                                        // Then, send through the channel for async handling
                                        if let Err(e) = message_sender.send(message) {
                                            tracing::debug!("Failed to send notification through channel: {}", e);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Err(err) => {
                                warn!("Failed to parse SSE message: {err}");
                            }
                        }
                    }
                    Ok(Some(_)) => continue,
                    Ok(None) | Err(_) => {
                        tracing::error!("SSE stream ended or errored");
                        break;
                    }
                }
            },
            _ = shutdown.cancelled() => {
                tracing::info!("SSE handler received shutdown");
                break;
            }
        }
    }

    pending_requests.clear().await;

    Ok(())
}
