use async_trait::async_trait;
use mcp_core_rs::{
    Resource, Tool, content::Content, prompt::Prompt, protocol::capabilities::ServerCapabilities,
};
use mcp_error_rs::{Error, Result};
use serde_json::Value;

#[async_trait]
pub trait Router: Send + Sync {
    fn name(&self) -> String;

    fn instructions(&self) -> String;

    fn capabilities(&self) -> ServerCapabilities;

    fn list_tools(&self) -> Vec<Tool>;

    async fn call_tool(&self, tool_name: &str, arguments: Value) -> Result<Vec<Content>>;

    fn list_resources(&self) -> Vec<Resource> {
        vec![]
    }

    async fn read_resource(&self, _uri: &str) -> Result<String> {
        Err(Error::System(
            "No resources implemented for this server.".into(),
        ))
    }

    fn list_prompts(&self) -> Vec<Prompt> {
        vec![]
    }

    async fn get_prompt(&self, _prompt_name: &str) -> Result<String> {
        Err(Error::System(
            "No prompts implemented for this server.".into(),
        ))
    }
}
