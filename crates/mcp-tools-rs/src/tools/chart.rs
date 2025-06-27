use mcp_error_rs::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChartSpec {
    pub chart_type: String,
    pub title: String,
    pub labels: Vec<String>,
    pub values: Vec<f64>,
}

/// Generate a chart specification as a structured JSON value.
pub fn generate_chart(spec: &ChartSpec) -> Result<Value> {
    if spec.labels.len() != spec.values.len() {
        return Err(Error::InvalidParameters(
            "labels and values must be the same length".to_string(),
        ));
    }
    let json = json!({
        "type": spec.chart_type,
        "title": spec.title,
        "labels": spec.labels,
        "values": spec.values,
    });
    Ok(json)
}

/// Public async function to be used by router logic.
pub async fn call_generate_chart(args: Value) -> Result<Value> {
    let spec: ChartSpec = serde_json::from_value(args)
        .map_err(|e| Error::InvalidParameters(format!("Invalid chart arguments: {}", e)))?;
    generate_chart(&spec)
}
