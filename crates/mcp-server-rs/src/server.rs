use crate::{
    core::protocol::{
        constants::{INTERNAL_ERROR, INVALID_REQUEST, PARSE_ERROR},
        error::ErrorData,
        message::{JsonRpcError, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse},
    },
    error::{Error, Result},
    router::{ext::RouterExt, traits::Router},
    transport::traits::ServerTransport,
};

pub struct Server {
    router: Box<dyn Router>,
}

impl Server {
    pub fn new(router: Box<dyn Router>) -> Self {
        Self { router }
    }

    pub async fn run(mut self, mut transport: impl ServerTransport) -> Result<()> {
        let router = &mut *self.router;

        tracing::info!("Server started");
        while let Some(msg_result) = transport.read_message().await {
            let _span = tracing::span!(tracing::Level::INFO, "message_processing");
            let _enter = _span.enter();

            match msg_result {
                Ok(msg) => {
                    Self::handle_message(router, &mut transport, msg).await?;
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
        router: &mut dyn Router,
        transport: &mut impl ServerTransport,
        msg: JsonRpcMessage,
    ) -> Result<()> {
        match msg {
            JsonRpcMessage::Request(request) => {
                let response = Self::process_request(router, request).await;
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

    async fn process_request(router: &dyn Router, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id;
        let request_json = serde_json::to_string(&request)
            .unwrap_or_else(|_| "Failed to serialize request".to_string());

        tracing::debug!(
            request_id = ?id,
            method = ?request.method,
            json = %request_json,
            "Received request"
        );

        let result = match request.method.as_str() {
            "initialize" => router.handle_initialize(request).await,
            "tools/list" => router.handle_tools_list(request).await,
            "tools/call" => router.handle_tools_call(request).await,
            "resources/list" => router.handle_resources_list(request).await,
            "resources/read" => router.handle_resources_read(request).await,
            "prompts/list" => router.handle_prompts_list(request).await,
            "prompts/get" => router.handle_prompts_get(request).await,
            _ => {
                let mut response = router.create_response(id);
                response.error = Some(ErrorData {
                    code: INVALID_REQUEST,
                    message: format!("Method '{}' not found", request.method),
                    data: None,
                });
                return response;
            }
        };

        match result {
            Ok(resp) => resp,
            Err(e) => {
                let error_msg = e.to_string();
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
