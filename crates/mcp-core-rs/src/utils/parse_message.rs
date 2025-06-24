use mcp_error_rs::{Error, Result};

use crate::protocol::{
    constants::{JSONRPC_EXPECTED_VERSION, JSONRPC_VERSION_FIELD},
    message::JsonRpcMessage,
};

/// Parses a JSON-RPC message from a string, validating structure and version.
pub fn parse_json_rpc_message(line: &str) -> Result<JsonRpcMessage> {
    let value: serde_json::Value = serde_json::from_str(line)?;
    if !value.is_object() {
        return Err(Error::InvalidMessage(
            "Message must be a JSON object".into(),
        ));
    }
    let obj = value.as_object().unwrap();

    match obj.get(JSONRPC_VERSION_FIELD) {
        Some(serde_json::Value::String(v)) if v == JSONRPC_EXPECTED_VERSION => {}
        _ => {
            return Err(Error::InvalidMessage(
                "Missing or invalid jsonrpc version".into(),
            ));
        }
    }

    let msg = serde_json::from_value(value)?;
    Ok(msg)
}
