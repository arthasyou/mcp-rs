use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    core::{
        MimeType, Resource, Tool, content::Content, prompt::Prompt,
        protocol::capabilities::ServerCapabilities,
    },
    error::{Error, Result},
    service::{capabilities::CapabilitiesBuilder, traits::Router},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChartSpec {
    pub chart_type: String,
    pub title: String,
    pub labels: Vec<String>,
    pub values: Vec<f64>,
}

#[derive(Clone)]
pub struct ChartRouter;

impl ChartRouter {
    pub fn new() -> Self {
        Self
    }

    fn _create_resource_text(&self, uri: &str, name: &str) -> Resource {
        Resource::new(uri, MimeType::Text, Some(name.to_string())).unwrap()
    }
}

#[async_trait]
impl Router for ChartRouter {
    fn name(&self) -> String {
        "chart".to_string()
    }

    fn instructions(&self) -> String {
        "This server provides a chart generation tool. Input should include chart_type, title, \
         labels, and values."
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
        vec![Tool::new(
            "generate_chart".to_string(),
            "Generate a chart spec from given input".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "chart_type": { "type": "string" },
                    "title": { "type": "string" },
                    "labels": { "type": "array", "items": { "type": "string" } },
                    "values": { "type": "array", "items": { "type": "number" } }
                },
                "required": ["chart_type", "title", "labels", "values"]
            }),
        )]
    }

    async fn call_tool(&self, tool_name: &str, arguments: Value) -> Result<Vec<Content>> {
        let tool_name = tool_name.to_string();
        match tool_name.as_str() {
            "generate_chart" => {
                let spec: ChartSpec = serde_json::from_value(arguments)
                    .map_err(|e| Error::System(format!("Invalid input: {}", e)))?;
                let content = serde_json::to_string(&spec)
                    .map_err(|e| Error::System(format!("Serialization error: {}", e)))?;
                Ok(vec![Content::text(content)])
            }
            _ => Err(Error::System(format!("Tool {} not found", tool_name))),
        }
    }

    fn list_resources(&self) -> Vec<Resource> {
        vec![]
    }

    // fn read_resource(
    //     &self,
    //     _uri: &str,
    // ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'static>> {
    //     Box::pin(async move {
    //         Err(Error::System(
    //             "No resources implemented for chart router.".into(),
    //         ))
    //     })
    // }

    fn list_prompts(&self) -> Vec<Prompt> {
        vec![]
    }

    // fn get_prompt(
    //     &self,
    //     _prompt_name: &str,
    // ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'static>> {
    //     Box::pin(async move {
    //         Err(Error::System(
    //             "No prompts implemented for chart router.".into(),
    //         ))
    //     })
    // }
}
