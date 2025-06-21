use std::sync::Arc;

use async_trait::async_trait;
use eventsource_client::{Client as SseClient, SSE};
use futures::TryStreamExt;
use mcp_core::protocol::message::JsonRpcMessage;
use mcp_error::{Error, Result};
use serde_json::{self, json};
use service_utils_rs::utils::Request;
use tokio::{spawn, sync::RwLock};
use tracing::{self, warn};
use url::Url;

use crate::client::{
    message::PendingRequests,
    traits::{ClientTransport, send_message},
};

pub struct SseTransport {
    sse_url: String,
    post_endpoint: Arc<RwLock<Option<String>>>,
    pending_requests: Arc<PendingRequests>,
    request: Request,
}

impl SseTransport {
    pub fn new(sse_url: String) -> Self {
        let mut request = Request::new();
        request
            .set_default_headers(vec![("Content-Type", "application/json".to_string())])
            .unwrap();
        Self {
            sse_url,
            post_endpoint: Arc::new(RwLock::new(None)),
            pending_requests: Arc::new(PendingRequests::default()),
            request,
        }
    }

    pub async fn get_post_endpoint(&self) -> Result<String> {
        let guard = self.post_endpoint.read().await;
        match &*guard {
            Some(endpoint) => Ok(endpoint.clone()),
            None => Err(Error::System(
                "POST endpoint not discovered yet".to_string(),
            )),
        }
    }
}

#[async_trait]
impl ClientTransport for SseTransport {
    async fn start(&self) -> Result<()> {
        let sse_url = self.sse_url.clone();
        let pending_requests = self.pending_requests.clone();
        let post_endpoint = self.post_endpoint.clone();
        spawn(async move {
            handle_messages(sse_url, pending_requests, post_endpoint).await;
        });

        Ok(())
    }

    async fn send(&self, message: JsonRpcMessage) -> Result<JsonRpcMessage> {
        let post_url = &self.get_post_endpoint().await?;
        let response = self
            .request
            .post(post_url, &serde_json::to_value(&message)?, None)
            .await
            .map_err(|e| {
                warn!("Failed to send message: {}", e);
                Error::System(e.to_string())
            })?;
        let json: JsonRpcMessage = response.json().await?;
        Ok(json)
    }

    async fn close(&self) -> Result<()> {
        // Implementation for closing the SSE transport
        Ok(())
    }
}

async fn handle_messages(
    sse_url: String,
    pending_requests: Arc<PendingRequests>,
    post_endpoint: Arc<RwLock<Option<String>>>,
) {
    let client = match eventsource_client::ClientBuilder::for_url(&sse_url) {
        Ok(builder) => builder.build(),
        Err(e) => {
            pending_requests.clear().await;
            warn!("Failed to connect SSE client: {}", e);
            return;
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
    while let Ok(Some(event)) = stream.try_next().await {
        match event {
            SSE::Event(e) if e.event_type == "message" => {
                // Attempt to parse the SSE data as a JsonRpcMessage
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
                            _ => {} // TODO: Handle other variants (Request, etc.)
                        }
                    }
                    Err(err) => {
                        warn!("Failed to parse SSE message: {err}");
                    }
                }
            }
            _ => { /* ignore other events */ }
        }
    }

    // SSE stream ended or errored; signal any pending requests
    tracing::error!("SSE stream ended or encountered an error; clearing pending requests.");
    pending_requests.clear().await;
}
