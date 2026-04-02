//! MCP (Model Context Protocol) stdio server

use crate::Vault;
use anyhow::Result;
use tracing::info;

/// MCP server for stdio-based communication
pub struct McpServer {
    vault: Vault,
}

impl McpServer {
    pub fn new(vault: Vault) -> Self {
        Self { vault }
    }

    /// Run the MCP stdio server
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting MCP stdio server");

        // TODO: Implement JSON-RPC over stdio
        // Based on @modelcontextprotocol/sdk

        Ok(())
    }
}

// MCP tool definitions
#[derive(Debug, Clone)]
pub enum McpTool {
    #[allow(dead_code)]
    Put,
    #[allow(dead_code)]
    Search,
    #[allow(dead_code)]
    Query,
    #[allow(dead_code)]
    IntentSearch,
    #[allow(dead_code)]
    Surface,
    #[allow(dead_code)]
    Extract,
    #[allow(dead_code)]
    Reflect,
    #[allow(dead_code)]
    Feedback,
    #[allow(dead_code)]
    Forget,
    #[allow(dead_code)]
    Status,
}
