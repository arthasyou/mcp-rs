use async_trait::async_trait;
use mcp_core_rs::{
    ResourceContents,
    content::Content,
    prompt::{PromptMessage, PromptMessageRole},
    protocol::{
        message::{JsonRpcRequest, JsonRpcResponse},
        result::{
            CallToolResult, GetPromptResult, Implementation, InitializeResult, ListPromptsResult,
            ListResourcesResult, ListToolsResult, ReadResourceResult,
        },
    },
};
use mcp_error_rs::{Error, Result};
use serde_json::Value;

use crate::service::traits::Service;

#[async_trait]
pub trait ServiceExt: Service {
    // Helper method to create base response
    fn create_response(&self, id: Option<u64>) -> JsonRpcResponse {
        JsonRpcResponse::empty(id)
    }

    async fn handle_initialize(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: self.capabilities(),
            server_info: Implementation {
                name: self.name(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some(self.instructions()),
        };

        let mut response = self.create_response(req.id);
        response.result = Some(
            serde_json::to_value(result)
                .map_err(|e| Error::System(format!("JSON serialization error: {}", e)))?,
        );

        Ok(response)
    }

    async fn handle_tools_list(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let tools = self.list_tools();

        let result = ListToolsResult {
            tools,
            next_cursor: None,
        };
        let mut response = self.create_response(req.id);
        response.result = Some(
            serde_json::to_value(result)
                .map_err(|e| Error::System(format!("JSON serialization error: {}", e)))?,
        );

        Ok(response)
    }

    async fn handle_tools_call(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let params = req
            .params
            .ok_or_else(|| Error::InvalidParameters("Missing parameters".into()))?;

        let name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::InvalidParameters("Missing tool name".into()))?;

        let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

        let result = match self.call_tool(name, arguments).await {
            Ok(result) => CallToolResult {
                content: result,
                is_error: None,
            },
            Err(err) => CallToolResult {
                content: vec![Content::text(err.to_string())],
                is_error: Some(true),
            },
        };

        let mut response = self.create_response(req.id);
        response.result = Some(
            serde_json::to_value(result)
                .map_err(|e| Error::System(format!("JSON serialization error: {}", e)))?,
        );

        Ok(response)
    }

    async fn handle_resources_list(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let resources = self.list_resources();

        let result = ListResourcesResult {
            resources,
            next_cursor: None,
        };
        let mut response = self.create_response(req.id);
        response.result = Some(
            serde_json::to_value(result)
                .map_err(|e| Error::System(format!("JSON serialization error: {}", e)))?,
        );

        Ok(response)
    }

    async fn handle_resources_read(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let params = req
            .params
            .ok_or_else(|| Error::InvalidParameters("Missing parameters".into()))?;

        let uri = params
            .get("uri")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::InvalidParameters("Missing resource URI".into()))?;

        let contents = self.read_resource(uri).await.map_err(Error::from)?;

        let result = ReadResourceResult {
            contents: vec![ResourceContents::TextResourceContents {
                uri: uri.to_string(),
                mime_type: Some("text/plain".to_string()),
                text: contents,
            }],
        };

        let mut response = self.create_response(req.id);
        response.result = Some(
            serde_json::to_value(result)
                .map_err(|e| Error::System(format!("JSON serialization error: {}", e)))?,
        );

        Ok(response)
    }

    async fn handle_prompts_list(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let prompts = self.list_prompts();

        let result = ListPromptsResult { prompts };

        let mut response = self.create_response(req.id);
        response.result = Some(
            serde_json::to_value(result)
                .map_err(|e| Error::System(format!("JSON serialization error: {}", e)))?,
        );

        Ok(response)
    }

    async fn handle_prompts_get(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let params = req
            .params
            .ok_or_else(|| Error::InvalidParameters("Missing parameters".into()))?;

        // Extract "name" field
        let prompt_name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::InvalidParameters("Missing prompt name".into()))?;

        // Extract "arguments" field
        let arguments = params
            .get("arguments")
            .and_then(Value::as_object)
            .ok_or_else(|| Error::InvalidParameters("Missing arguments object".into()))?;

        // Fetch the prompt definition first
        let prompt = self
            .list_prompts()
            .into_iter()
            .find(|p| p.name == prompt_name)
            .ok_or_else(|| Error::System(format!("Prompt '{}' not found", prompt_name)))?;

        // Validate required arguments
        if let Some(args) = &prompt.arguments {
            for arg in args {
                if arg.required.is_some()
                    && arg.required.unwrap()
                    && (!arguments.contains_key(&arg.name)
                        || arguments
                            .get(&arg.name)
                            .and_then(Value::as_str)
                            .is_none_or(str::is_empty))
                {
                    return Err(Error::InvalidParameters(format!(
                        "Missing required argument: '{}'",
                        arg.name
                    )));
                }
            }
        }

        // Now get the prompt content
        let description = self
            .get_prompt(prompt_name)
            .await
            .map_err(|e| Error::System(e.to_string()))?;

        // Validate prompt arguments for potential security issues from user text input
        // Checks:
        // - Prompt must be less than 10000 total characters
        // - Argument keys must be less than 1000 characters
        // - Argument values must be less than 1000 characters
        // - Dangerous patterns, eg "../", "//", "\\\\", "<script>", "{{", "}}"
        for (key, value) in arguments.iter() {
            // Check for empty or overly long keys/values
            if key.is_empty() || key.len() > 1000 {
                return Err(Error::InvalidParameters(
                    "Argument keys must be between 1-1000 characters".into(),
                ));
            }

            let value_str = value.as_str().unwrap_or_default();
            if value_str.len() > 1000 {
                return Err(Error::InvalidParameters(
                    "Argument values must not exceed 1000 characters".into(),
                ));
            }

            // Check for potentially dangerous patterns
            let dangerous_patterns = ["../", "//", "\\\\", "<script>", "{{", "}}"];
            for pattern in dangerous_patterns {
                if key.contains(pattern) || value_str.contains(pattern) {
                    return Err(Error::InvalidParameters(format!(
                        "Arguments contain potentially unsafe pattern: {}",
                        pattern
                    )));
                }
            }
        }

        // Validate the prompt description length
        if description.len() > 10000 {
            return Err(Error::System(
                "Prompt description exceeds maximum allowed length".into(),
            ));
        }

        // Create a mutable copy of the description to fill in arguments
        let mut description_filled = description.clone();

        // Replace each argument placeholder with its value from the arguments object
        for (key, value) in arguments {
            let placeholder = format!("{{{}}}", key);
            description_filled =
                description_filled.replace(&placeholder, value.as_str().unwrap_or_default());
        }

        let messages = vec![PromptMessage::new_text(
            PromptMessageRole::User,
            description_filled.to_string(),
        )];

        // Build the final response
        let mut response = self.create_response(req.id);
        response.result = Some(
            serde_json::to_value(GetPromptResult {
                description: Some(description_filled),
                messages,
            })
            .map_err(|e| Error::System(format!("JSON serialization error: {}", e)))?,
        );
        Ok(response)
    }

    // 可继续添加 handle_tools_call, handle_resources_read, handle_prompts_get 等
}

impl<T: Service + ?Sized> ServiceExt for T {}
