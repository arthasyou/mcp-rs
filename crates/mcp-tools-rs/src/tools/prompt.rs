use mcp_error_rs::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Serialize, Deserialize)]
pub struct SummaryInput {
    pub text: String,
}

pub fn summarize_text(input: &SummaryInput) -> Result<Value> {
    if input.text.trim().is_empty() {
        return Err(Error::InvalidParameters("Input text is empty".to_string()));
    }

    // 构造摘要任务描述，供 LLM 使用
    let prompt = format!(
        "请对以下内容进行简要总结，控制在 3~5 句话内：\n\n{}",
        input.text.trim()
    );

    Ok(json!({
        "prompt": prompt
    }))
}

pub async fn call_summarize_text(args: Value) -> Result<Value> {
    let input: SummaryInput = serde_json::from_value(args)
        .map_err(|e| Error::InvalidParameters(format!("Invalid summary input: {}", e)))?;
    summarize_text(&input)
}
