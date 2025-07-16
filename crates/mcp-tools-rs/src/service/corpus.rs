use async_trait::async_trait;
use model_gateway_rs::{
    clients::llm::LlmClient,
    model::llm::{ChatMessage, LlmInput, LlmOutput},
    sdk::ModelSDK,
    traits::ModelClient,
};
use serde_json::Value;
use service_utils_rs::utils::request::Request;

use crate::{
    core::{Resource, Tool, content::Content, protocol::capabilities::ServerCapabilities},
    error::{Error, Result},
    server::service::{capabilities::CapabilitiesBuilder, traits::Service},
};

/// Service for expanding corpus text via an LLM (e.g., OpenAI Chat API).

pub struct CorpusService<T>
where
    T: ModelSDK + Sync + Send,
{
    llm_client: LlmClient<T>,
    request: Request,
}

impl<T> CorpusService<T>
where
    T: ModelSDK<Input = LlmInput, Output = LlmOutput> + Sync + Send,
{
    /// Create a new CorpusService with given API key and model name.
    pub fn new(llm_client: LlmClient<T>) -> Self {
        let request = Request::new();

        Self {
            llm_client,
            request,
        }
    }

    /// Expand the given text once using the LLM.
    async fn expand_once(&self, content_url: &str, prompt_url: &str) -> Result<String> {
        // system prompt for expansion
        let system_prompt = self.read_text_file(prompt_url).await?;

        let content = self.read_text_file(content_url).await?;

        let user_message = ChatMessage::user(content.as_str());
        let assistant_message = ChatMessage::system(system_prompt.as_str());
        let messages = vec![user_message, assistant_message];
        let input = LlmInput {
            messages,
            max_tokens: Some(32768),
        };

        println!("📄 发送内容: {:?}", input);

        let r: LlmOutput = self.llm_client.infer(input).await?;

        println!("📄 接收内容: {:?}", r);

        let a = r.get_content().to_owned();

        Ok(a)
    }

    async fn read_text_file(&self, file_path: &str) -> Result<String> {
        let response = self.request.get(file_path, None, None).await?;
        let text = response.text().await?;
        Ok(text)
    }
}

#[async_trait]
impl<T> Service for CorpusService<T>
where
    T: ModelSDK<Input = LlmInput, Output = LlmOutput> + Sync + Send,
{
    fn name(&self) -> String {
        "corpus_expansion".into()
    }

    fn instructions(&self) -> String {
        "Use an LLM with Chinese language capabilities to expand user-provided corpus text \
         enriching it with more detail and examples."
            .into()
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
            "expand_corpus".to_string(),
            "Expand corpus text using the language model".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content_path": { "type": "string" },
                    "prompt_path": { "type": "string" },
                    "iterations": { "type": "integer", "minimum": 1 }
                },
                "required": ["file_name", "content"]
            }),
        )]
    }

    async fn call_tool(&self, tool_name: &str, args: Value) -> Result<Vec<Content>> {
        match tool_name {
            "expand_corpus" => {
                let content_path = args
                    .get("content_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::System("Missing content_path".into()))?;

                let prompt_path = args
                    .get("prompt_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::System("Missing content".into()))?;

                let a = self.expand_once(content_path, prompt_path).await?;
                Ok(vec![Content::text(a)])
            }
            _ => Err(Error::System(format!("Tool {} not found", tool_name))),
        }
    }

    fn list_resources(&self) -> Vec<Resource> {
        Vec::new()
    }

    async fn read_resource(&self, _uri: &str) -> Result<String> {
        Err(Error::System("No resources available".into()))
    }
}
