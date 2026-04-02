//! Cognitive layer — decision extraction, causal graph, A-MEM evolution, feedback

pub mod extract;
pub mod graph;
pub mod amem;
pub mod feedback;
pub mod conflict;

pub use extract::{ExtractDecision, DecisionType, extract_decisions};
pub use graph::{RelationType, build_causal_edges};
pub use amem::{enrich_memory, extract_keywords, extract_tags};
pub use feedback::{process_feedback, FeedbackSignal};
