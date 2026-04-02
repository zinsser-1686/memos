//! Memos — AI Agent Memory System
//!
//! Combines Memvid's ultra-fast single-file vector storage layer with
//! ClawMem's cognitive engine: autonomous decision extraction, causal graph
//! traversal, A-MEM self-evolution, and feedback-driven decay.

pub mod storage;
pub mod retrieval;
pub mod cognitive;
pub mod lifecycle;
pub mod hooks;
pub mod api;
pub mod config;

pub use config::Config;
pub use storage::vault::Vault;
pub use retrieval::{SearchRequest, SearchHit, QueryIntent};
pub use cognitive::{ExtractDecision, RelationType, MemoryType};
pub use lifecycle::LifecycleState;

// Re-export error types
pub use thiserror::Error;
pub use anyhow::Result;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
