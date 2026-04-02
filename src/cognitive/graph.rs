//! Causal graph construction and management

use serde::{Deserialize, Serialize};
use crate::cognitive::extract::ExtractDecision;

/// Relation types between memory nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelationType {
    Cause,
    Effect,
    Semantic,
    Temporal,
    Contradicts,
    Supports,
}

/// A relation edge in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub source_id: String,
    pub target_id: String,
    pub rel_type: RelationType,
    pub confidence: f32,
}

/// Build causal edges from extracted decisions
///
/// For each decision pair, the LLM determines if there's a cause→effect relationship.
/// Confidence must be >= 0.6 to create an edge.
pub fn build_causal_edges(
    decisions: &[ExtractDecision],
    existing_decisions: &[(String, ExtractDecision)],
) -> Vec<Relation> {
    let mut relations = Vec::new();

    for decision in decisions {
        // Check against existing decisions for causal relationships
        for (existing_id, existing) in existing_decisions {
            if decision.decision_type == crate::cognitive::extract::DecisionType::Fact {
                continue;
            }

            // Check for contradiction
            if !decision.contradicts.is_empty() {
                for contradicted in &decision.contradicts {
                    relations.push(Relation {
                        source_id: contradicted.clone(),
                        target_id: decision.title.clone(), // Use title as proxy for ID
                        rel_type: RelationType::Contradicts,
                        confidence: 0.75,
                    });
                }
            }

            // TODO: LLM-based causal inference
            // In production, prompt LLM: "Does A cause B? Confidence 0-1?"
        }
    }

    relations
}

impl RelationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationType::Cause => "cause",
            RelationType::Effect => "effect",
            RelationType::Semantic => "semantic",
            RelationType::Temporal => "temporal",
            RelationType::Contradicts => "contradicts",
            RelationType::Supports => "supports",
        }
    }
}
