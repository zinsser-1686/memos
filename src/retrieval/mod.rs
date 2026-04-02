//! Retrieval layer — hybrid search, intent classification, RRF, reranking

pub mod search;
pub mod intent;
pub mod rerank;
pub mod graph;

pub use search::{SearchRequest, SearchHit};
pub use intent::QueryIntent;
pub use rerank::rerank_candidates;
pub use graph::expand_with_causal_graph;
