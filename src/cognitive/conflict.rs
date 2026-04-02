//! Conflict detection — identify contradicting decisions

use regex::Regex;
use once_cell::sync::Lazy;
use crate::cognitive::extract::ExtractDecision;
use crate::storage::frame::MemoryFrame;

/// Patterns that indicate negation or value change
static NEGATION_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"\bnot\b|\bno\b|\bnever\b|\bdon't\b|\bdoesn't\b|\bdidn't\b|\bwon't\b|\bwouldn't\b").unwrap(),
        Regex::new(r"\bchange[sd]?\b|\balter[ed]?\b|\breplac[ed]?\b|\bswitch[ed]?\b").unwrap(),
        Regex::new(r"\bhowever\b|\bbut\b|\byet\b|\balthough\b").unwrap(),
    ]
});

/// Status keywords that indicate transitions
static STATUS_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"\bTODO\b|\bDONE\b|\bIN PROGRESS\b|\bBLOCKED\b|\bCOMPLETED\b").unwrap(),
        Regex::new(r"\bwas\b.*\bnow\b|\bchanged\b|\bupdated\b").unwrap(),
    ]
});

/// A detected conflict between two decisions
#[derive(Debug, Clone)]
pub struct Conflict {
    /// The newer decision that supersedes
    pub new_id: String,
    /// The older decision that was contradicted
    pub old_id: String,
    /// Severity of contradiction (0.0–1.0)
    pub severity: f32,
    /// Human-readable reason
    pub reason: String,
}

/// Detect conflicts between a new decision and prior decisions
pub fn detect_conflicts(
    new: &ExtractDecision,
    existing: &[MemoryFrame],
) -> Vec<Conflict> {
    let mut conflicts = Vec::new();

    // Only check decision-type memories
    let candidates: Vec<_> = existing
        .iter()
        .filter(|m| m.memory_type == crate::storage::frame::MemoryType::Decision)
        .collect();

    for prior in candidates {
        // Skip if same topic (identified by title similarity)
        if prior.title == new.title {
            continue;
        }

        // Check for negation patterns in both
        let prior_has_negation = contains_negation(&prior.narrative());
        let new_has_negation = contains_negation(&new.narrative);

        // Check for status changes
        let prior_has_status = contains_status(&prior.narrative());
        let new_has_status = contains_status(&new.narrative);

        if prior_has_negation && new_has_negation && prior_has_status != new_has_status {
            // Likely a contradiction — value changed from A to B
            conflicts.push(Conflict {
                new_id: new.title.clone(),
                old_id: prior.frame_id.clone(),
                severity: 0.25,
                reason: format!(
                    "Decision '{}' appears to contradict prior decision '{}'",
                    new.title, prior.title
                ),
            });
        }
    }

    conflicts
}

fn contains_negation(text: &str) -> bool {
    NEGATION_PATTERNS.iter().any(|p| p.is_match(text))
}

fn contains_status(text: &str) -> bool {
    STATUS_PATTERNS.iter().any(|p| p.is_match(text))
}

impl MemoryFrame {
    /// Get narrative text for conflict detection
    fn narrative(&self) -> String {
        if !self.context_desc.is_empty() {
            self.context_desc.clone()
        } else {
            self.content.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negation_detection() {
        assert!(contains_negation("We should NOT use this approach"));
        assert!(contains_negation("This is no longer valid"));
        assert!(!contains_negation("We should use this approach"));
    }

    #[test]
    fn test_status_detection() {
        assert!(contains_status("Status: TODO"));
        assert!(contains_status("Was: A, Now: B"));
        assert!(!contains_status("No status change"));
    }
}
