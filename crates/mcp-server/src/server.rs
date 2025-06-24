use mcp_core_rs::protocol::{
    constants::{INTERNAL_ERROR, INVALID_REQUEST, PARSE_ERROR},
    error::ErrorData,
    message::{JsonRpcError, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse},
};
use mcp_error_rs::{BoxError, Error, Result};
use mcp_transport_rs::server::traits::ServerTransport;
use tower_service::Service;

pub struct Server<S> {
    service: S,
}

impl<S> Server<S>
where
    S: Service<JsonRpcRequest, Response = JsonRpcResponse> + Send,
    S::Error: Into<BoxError>,
    S::Future: Send,
{
    pub fn new(service: S) -> Self {
        Self { service }
    }

    pub async fn run(self, mut transport: impl ServerTransport) -> Result<()> {
        let mut service = self.service;

        tracing::info!("Server started");
        while let Some(msg_result) = transport.read_message().await {
            let _span = tracing::span!(tracing::Level::INFO, "message_processing");
            let _enter = _span.enter();

            match msg_result {
                Ok(msg) => {
                    Self::handle_message(&mut service, &mut transport, msg).await?;
                }
                Err(e) => {
                    Self::handle_error(&mut transport, e).await?;
                }
            }
        }
        tracing::info!("Server transport closed, exiting run loop");

        Ok(())
    }

    async fn handle_message(
        service: &mut S,
        transport: &mut impl ServerTransport,
        msg: JsonRpcMessage,
    ) -> Result<()> {
        match msg {
            JsonRpcMessage::Request(request) => {
                let response = Self::process_request(service, request).await;
                Self::send_response(transport, response).await?;
            }
            JsonRpcMessage::Response(_)
            | JsonRpcMessage::Notification(_)
            | JsonRpcMessage::Nil
            | JsonRpcMessage::Error(_) => {
                // TODO: Handle other message types
            }
        }
        Ok(())
    }

    async fn process_request(service: &mut S, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id;
        let request_json = serde_json::to_string(&request)
            .unwrap_or_else(|_| "Failed to serialize request".to_string());

        tracing::debug!(
            request_id = ?id,
            method = ?request.method,
            json = %request_json,
            "Received request"
        );

        match service.call(request).await {
            Ok(resp) => resp,
            Err(e) => {
                let error_msg = e.into().to_string();
                tracing::error!(error = %error_msg, "Request processing failed");
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(ErrorData {
                        code: INTERNAL_ERROR,
                        message: error_msg,
                        data: None,
                    }),
                }
            }
        }
    }

    async fn send_response(
        transport: &mut impl ServerTransport,
        response: JsonRpcResponse,
    ) -> Result<()> {
        let response_json = serde_json::to_string(&response)
            .unwrap_or_else(|_| "Failed to serialize response".to_string());

        tracing::debug!(
            response_id = ?response.id,
            json = %response_json,
            "Sending response"
        );

        transport
            .write_message(JsonRpcMessage::Response(response))
            .await
    }

    async fn handle_error(transport: &mut impl ServerTransport, e: Error) -> Result<()> {
        let error = match e {
            Error::Json(_) | Error::InvalidMessage(_) => ErrorData {
                code: PARSE_ERROR,
                message: e.to_string(),
                data: None,
            },
            Error::Protocol(_) => ErrorData {
                code: INVALID_REQUEST,
                message: e.to_string(),
                data: None,
            },
            _ => ErrorData {
                code: INTERNAL_ERROR,
                message: e.to_string(),
                data: None,
            },
        };

        let error_response = JsonRpcMessage::Error(JsonRpcError {
            jsonrpc: "2.0".to_string(),
            id: None,
            error,
        });

        transport.write_message(error_response).await
    }
}
