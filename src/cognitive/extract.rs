//! Decision extraction from session transcripts

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Decision type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DecisionType {
    Decision,
    Observation,
    Preference,
    Fact,
}

impl Default for DecisionType {
    fn default() -> Self {
        DecisionType::Fact
    }
}

/// A key-value fact extracted from transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub key: String,
    pub value: String,
}

/// An extracted decision/observation from a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractDecision {
    pub decision_type: DecisionType,
    pub title: String,
    pub facts: Vec<Fact>,
    pub narrative: String,
    pub contradicts: Vec<String>, // frame_ids of superseded decisions
    pub confidence: f32,
}

/// Extract decisions from a transcript
///
/// This is a stub implementation. In production:
/// 1. Split transcript into user/assistant turns
/// 2. For each assistant turn with conclusions:
///    - Call local LLM with extraction prompt
///    - Parse structured JSON output
/// 3. Detect contradictions against prior decisions
///
/// For now, returns empty vec (not yet implemented).
pub fn extract_decisions(transcript: &str) -> Vec<ExtractDecision> {
    if transcript.trim().is_empty() {
        return Vec::new();
    }

    // STUB: Not yet implemented
    // In production, this would call a local GGUF model
    tracing::warn!("Decision extraction not yet implemented — returning empty");
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_transcript() {
        let result = extract_decisions("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_whitespace_transcript() {
        let result = extract_decisions("   \n\n  ");
        assert!(result.is_empty());
    }
}
