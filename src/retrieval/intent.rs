//! Intent classification for query routing

use crate::retrieval::QueryIntent;
use regex::Regex;
use once_cell::sync::Lazy;

/// Heuristic patterns for fast intent classification
static WHY_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)^why|did we decide|reason for|why did|what caused|ecause").unwrap(),
        Regex::new(r"(?i)what led to|how did we|originated from|root cause").unwrap(),
    ]
});

static WHEN_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)^when|did we discuss|at what point|time when|on what date").unwrap(),
        Regex::new(r"(?i)last week|recently|yesterday|this week|last month|in march|in 202").unwrap(),
    ]
});

static ENTITY_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)^who|did we talk to|worked on|responsible for|assigned to").unwrap(),
        Regex::new(r"(?i)which team|which person|who proposed|who suggested").unwrap(),
    ]
});

/// Classify query intent using heuristics first, LLM fallback second
pub fn classify_intent(query: &str) -> (QueryIntent, f32) {
    // Check WHY patterns
    for pattern in WHY_PATTERNS.iter() {
        if pattern.is_match(query) {
            return (QueryIntent::Why, 0.95);
        }
    }

    // Check WHEN patterns
    for pattern in WHEN_PATTERNS.iter() {
        if pattern.is_match(query) {
            return (QueryIntent::When, 0.95);
        }
    }

    // Check ENTITY patterns
    for pattern in ENTITY_PATTERNS.iter() {
        if pattern.is_match(query) {
            return (QueryIntent::Entity, 0.95);
        }
    }

    // Heuristic threshold not met — default to WHAT
    (QueryIntent::What, 0.5)
}

/// Get intent description for logging/debugging
pub fn intent_description(intent: QueryIntent) -> &'static str {
    match intent {
        QueryIntent::Why => "WHY (causal reasoning)",
        QueryIntent::When => "WHEN (temporal recall)",
        QueryIntent::Entity => "ENTITY (named entity lookup)",
        QueryIntent::What => "WHAT (general recall)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_why_intent() {
        let (intent, conf) = classify_intent("why did we decide on this approach");
        assert_eq!(intent, QueryIntent::Why);
        assert!(conf >= 0.9);
    }

    #[test]
    fn test_when_intent() {
        let (intent, conf) = classify_intent("when was this discussed");
        assert_eq!(intent, QueryIntent::When);
        assert!(conf >= 0.9);
    }

    #[test]
    fn test_entity_intent() {
        let (intent, conf) = classify_intent("who worked on the API integration");
        assert_eq!(intent, QueryIntent::Entity);
        assert!(conf >= 0.9);
    }

    #[test]
    fn test_what_intent() {
        let (intent, _) = classify_intent("tell me about the project");
        assert_eq!(intent, QueryIntent::What);
    }
}
