use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::protocol::{constants::JSONRPC_VERSION_FIELD, error::ErrorData};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorData>,
}

impl JsonRpcResponse {
    pub fn new_empty(id: Option<u64>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION_FIELD.to_string(),
            id,
            result: None,
            error: None,
        }
    }

    pub fn with_error(id: Option<u64>, error: ErrorData) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION_FIELD.to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct JsonRpcError {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    pub error: ErrorData,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged, try_from = "JsonRpcRaw")]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
    Error(JsonRpcError),
    Nil, // used to respond to notifications
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRaw {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorData>,
}

impl TryFrom<JsonRpcRaw> for JsonRpcMessage {
    type Error = String;

    fn try_from(raw: JsonRpcRaw) -> Result<Self, <Self as TryFrom<JsonRpcRaw>>::Error> {
        // If it has an error field, it's an error response
        if raw.error.is_some() {
            return Ok(JsonRpcMessage::Error(JsonRpcError {
                jsonrpc: raw.jsonrpc,
                id: raw.id,
                error: raw.error.unwrap(),
            }));
        }

        // If it has a result field, it's a response
        if raw.result.is_some() {
            return Ok(JsonRpcMessage::Response(JsonRpcResponse {
                jsonrpc: raw.jsonrpc,
                id: raw.id,
                result: raw.result,
                error: None,
            }));
        }

        // If we have a method, it's either a notification or request
        if let Some(method) = raw.method {
            if raw.id.is_none() {
                return Ok(JsonRpcMessage::Notification(JsonRpcNotification {
                    jsonrpc: raw.jsonrpc,
                    method,
                    params: raw.params,
                }));
            }

            return Ok(JsonRpcMessage::Request(JsonRpcRequest {
                jsonrpc: raw.jsonrpc,
                id: raw.id,
                method,
                params: raw.params,
            }));
        }

        // If we have no method and no result/error, it's a nil response
        if raw.id.is_none() && raw.result.is_none() && raw.error.is_none() {
            return Ok(JsonRpcMessage::Nil);
        }

        // If we get here, something is wrong with the message
        Err(format!(
            "Invalid JSON-RPC message format: id={:?}, method={:?}, result={:?}, error={:?}",
            raw.id, raw.method, raw.result, raw.error
        ))
    }
}
