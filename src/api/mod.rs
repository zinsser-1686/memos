//! API layer — MCP stdio and HTTP REST interfaces

pub mod mcp;
pub mod http;

pub use mcp::McpServer;
pub use http::HttpServer;
