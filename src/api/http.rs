//! HTTP REST API server

use crate::Vault;
use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tracing::info;

/// HTTP API server
pub struct HttpServer {
    vault: Vault,
    port: u16,
}

impl HttpServer {
    pub fn new(vault: Vault, port: u16) -> Self {
        Self { vault, port }
    }

    /// Start the HTTP server
    pub async fn run(&self) -> Result<()> {
        info!("Starting HTTP API server on port {}", self.port);

        let app = Router::new()
            .route("/health", get(health))
            .route("/v1/search", post(search))
            .route("/v1/memories", post(put_memory))
            .route("/v1/status", get(status));

        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let listener = tokio::net::TcpListener::bind(addr).await?;

        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn health() -> &'static str {
    "OK"
}

async fn search() -> &'static str {
    "search endpoint"
}

async fn put_memory() -> &'static str {
    "put endpoint"
}

async fn status() -> &'static str {
    "status endpoint"
}
