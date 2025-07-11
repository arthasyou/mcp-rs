use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use once_cell::sync::Lazy;

use crate::{
    client::McpClient,
    error::{Error, Result},
    transport::{impls::sse::SseTransport, traits::ClientTransport},
};

/// MCP Client Registry
#[derive(Default)]
pub struct McpClientRegistry {
    clients: Mutex<HashMap<String, Arc<McpClient<SseTransport>>>>,
}

impl McpClientRegistry {
    pub fn register(&self, server_id: &str, client: Arc<McpClient<SseTransport>>) {
        let mut map = self.clients.lock().unwrap();
        map.insert(server_id.to_string(), client);
    }

    pub fn get(&self, server_id: &str) -> Result<Arc<McpClient<SseTransport>>> {
        let map = self.clients.lock().unwrap();
        map.get(server_id).cloned().ok_or_else(|| {
            Error::System(format!("MCP client not found for server_id: {}", server_id))
        })
    }

    pub fn list_keys(&self) -> Vec<String> {
        let map = self.clients.lock().unwrap();
        map.keys().cloned().collect()
    }
}

/// Global MCP client registry
static MCP_CLIENT_REGISTRY: Lazy<McpClientRegistry> = Lazy::new(McpClientRegistry::default);

/// Get the global MCP client registry
pub fn get_mcp_registry() -> &'static McpClientRegistry {
    &MCP_CLIENT_REGISTRY
}

/// Initialize and register a default MCP client
pub async fn register_mcp_clients(configs: Vec<(&str, &str)>) -> Result<()> {
    for (server_id, url) in configs {
        let transport = SseTransport::new(url);
        transport.start().await?;
        let client = Arc::new(McpClient::new(transport));
        get_mcp_registry().register(server_id, client);
    }
    Ok(())
}
