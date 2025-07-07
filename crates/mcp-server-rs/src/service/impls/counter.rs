use std::sync::Arc;

use async_trait::async_trait;
use mcp_core_rs::{
    MimeType, Resource, Tool,
    content::Content,
    prompt::{Prompt, PromptArgument},
    protocol::capabilities::ServerCapabilities,
};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::{
    error::{Error, Result},
    service::{capabilities::CapabilitiesBuilder, traits::Router},
};

#[derive(Clone)]
pub struct CounterRouter {
    counter: Arc<Mutex<i32>>,
}

impl CounterRouter {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
        }
    }

    async fn increment(&self) -> Result<i32> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        Ok(*counter)
    }

    async fn decrement(&self) -> Result<i32> {
        let mut counter = self.counter.lock().await;
        *counter -= 1;
        Ok(*counter)
    }

    async fn get_value(&self) -> Result<i32> {
        let counter = self.counter.lock().await;
        Ok(*counter)
    }

    fn _create_resource_text(&self, uri: &str, name: &str) -> Resource {
        Resource::new(uri, MimeType::Text, Some(name.to_string())).unwrap()
    }
}

#[async_trait]
impl Router for CounterRouter {
    fn name(&self) -> String {
        "counter".to_string()
    }

    fn instructions(&self) -> String {
        "This server provides a counter tool that can increment and decrement values. The counter \
         starts at 0 and can be modified using the 'increment' and 'decrement' tools. Use \
         'get_value' to check the current count."
            .to_string()
    }

    fn capabilities(&self) -> ServerCapabilities {
        CapabilitiesBuilder::new()
            .with_tools(false)
            .with_resources(false, false)
            .with_prompts(false)
            .build()
    }

    fn list_tools(&self) -> Vec<Tool> {
        vec![
            Tool::new(
                "increment".to_string(),
                "Increment the counter by 1".to_string(),
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            ),
            Tool::new(
                "decrement".to_string(),
                "Decrement the counter by 1".to_string(),
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            ),
            Tool::new(
                "get_value".to_string(),
                "Get the current counter value".to_string(),
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            ),
        ]
    }

    async fn call_tool(&self, tool_name: &str, _arguments: Value) -> Result<Vec<Content>> {
        let this = self.clone();
        let tool_name = tool_name.to_string();

        match tool_name.as_str() {
            "increment" => {
                let value = this.increment().await?;
                Ok(vec![Content::text(value.to_string())])
            }
            "decrement" => {
                let value = this.decrement().await?;
                Ok(vec![Content::text(value.to_string())])
            }
            "get_value" => {
                let value = this.get_value().await?;
                Ok(vec![Content::text(value.to_string())])
            }
            _ => Err(Error::System(format!("Tool {} not found", tool_name))),
        }
    }

    fn list_resources(&self) -> Vec<Resource> {
        vec![
            self._create_resource_text("str:////Users/to/some/path/", "cwd"),
            self._create_resource_text("memo://insights", "memo-name"),
        ]
    }

    async fn read_resource(&self, uri: &str) -> Result<String> {
        let uri = uri.to_string();
        match uri.as_str() {
            "str:////Users/to/some/path/" => {
                let cwd = "/Users/to/some/path/";
                Ok(cwd.to_string())
            }
            "memo://insights" => {
                let memo = "Business Intelligence Memo\n\nAnalysis has revealed 5 key insights ...";
                Ok(memo.to_string())
            }
            _ => Err(Error::System(format!("Resource {} not found", uri))),
        }
    }

    fn list_prompts(&self) -> Vec<Prompt> {
        vec![Prompt::new(
            "example_prompt",
            Some("This is an example prompt that takes one required agrument, message"),
            Some(vec![PromptArgument {
                name: "message".to_string(),
                description: Some("A message to put in the prompt".to_string()),
                required: Some(true),
            }]),
        )]
    }

    async fn get_prompt(&self, prompt_name: &str) -> Result<String> {
        let prompt_name = prompt_name.to_string();
        match prompt_name.as_str() {
            "example_prompt" => {
                let prompt = "This is an example prompt with your message here: '{message}'";
                Ok(prompt.to_string())
            }
            _ => Err(Error::System(format!("Prompt {} not found", prompt_name))),
        }
    }
}
