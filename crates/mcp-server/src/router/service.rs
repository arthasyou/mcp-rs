use std::{
    pin::Pin,
    task::{Context, Poll},
};

use mcp_core::protocol::{ErrorData, INVALID_REQUEST, JsonRpcRequest, JsonRpcResponse};
use mcp_error::BoxError;
use tower_service::Service;

use crate::router::traits::Router;

pub struct RouterService<T>(pub T);

impl<T> Service<JsonRpcRequest> for RouterService<T>
where
    T: Router + Clone + Send + Sync + 'static,
{
    type Response = JsonRpcResponse;
    type Error = BoxError;
    type Future =
        Pin<Box<dyn Future<Output = core::result::Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<core::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: JsonRpcRequest) -> Self::Future {
        let this = self.0.clone();

        Box::pin(async move {
            let result = match req.method.as_str() {
                "initialize" => this.handle_initialize(req).await,
                "tools/list" => this.handle_tools_list(req).await,
                "tools/call" => this.handle_tools_call(req).await,
                "resources/list" => this.handle_resources_list(req).await,
                "resources/read" => this.handle_resources_read(req).await,
                "prompts/list" => this.handle_prompts_list(req).await,
                "prompts/get" => this.handle_prompts_get(req).await,
                _ => {
                    let mut response = this.create_response(req.id);
                    let error_msg = format!("Method '{}' not found", req.method);
                    let error_data = ErrorData {
                        code: INVALID_REQUEST,
                        message: error_msg,
                        data: None,
                    };
                    response.error = Some(error_data);
                    Ok(response)
                }
            };

            result.map_err(BoxError::from)
        })
    }
}
