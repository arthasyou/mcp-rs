pub mod capabilities;
pub mod error;
pub mod message;
pub mod result;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::protocol::message::{JsonRpcMessage, JsonRpcRaw};

    #[test]
    fn test_notification_conversion() {
        let raw = JsonRpcRaw {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: Some("notify".to_string()),
            params: Some(json!({"key": "value"})),
            result: None,
            error: None,
        };

        let message = JsonRpcMessage::try_from(raw).unwrap();
        match message {
            JsonRpcMessage::Notification(n) => {
                assert_eq!(n.jsonrpc, "2.0");
                assert_eq!(n.method, "notify");
                assert_eq!(n.params.unwrap(), json!({"key": "value"}));
            }
            _ => panic!("Expected Notification"),
        }
    }

    #[test]
    fn test_request_conversion() {
        let raw = JsonRpcRaw {
            jsonrpc: "2.0".to_string(),
            id: Some(1),
            method: Some("request".to_string()),
            params: Some(json!({"key": "value"})),
            result: None,
            error: None,
        };

        let message = JsonRpcMessage::try_from(raw).unwrap();
        match message {
            JsonRpcMessage::Request(r) => {
                assert_eq!(r.jsonrpc, "2.0");
                assert_eq!(r.id, Some(1));
                assert_eq!(r.method, "request");
                assert_eq!(r.params.unwrap(), json!({"key": "value"}));
            }
            _ => panic!("Expected Request"),
        }
    }
}
