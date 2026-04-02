//! Cross-encoder reranking

use crate::retrieval::SearchHit;

/// Re-rank candidates using cross-encoder model
///
/// In a full implementation, this would call a local GGUF reranker
/// or an external API. For now, it's a stub.
pub fn rerank_candidates(
    _query: &str,
    candidates: Vec<SearchHit>,
    _top_k: usize,
    _context_chars: usize,
) -> Vec<SearchHit> {
    // In production:
    // 1. Construct query-document pairs with context
    // 2. Score with cross-encoder model
    // 3. Sort by cross-encoder scores
    // 4. Return top-k

    // For now, just return candidates as-is
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_rerank_returns_candidates() {
        let hits = vec![
            SearchHit {
                frame_id: "1".to_string(),
                title: "Test 1".to_string(),
                snippet: "Test snippet".to_string(),
                memory_type: "decision".to_string(),
                score: 0.9,
                confidence: 0.85,
                tags: vec![],
                created_at: Utc::now(),
            },
            SearchHit {
                frame_id: "2".to_string(),
                title: "Test 2".to_string(),
                snippet: "Another snippet".to_string(),
                memory_type: "note".to_string(),
                score: 0.8,
                confidence: 0.70,
                tags: vec![],
                created_at: Utc::now(),
            },
        ];

        let reranked = rerank_candidates("test query", hits, 2, 4000);
        assert_eq!(reranked.len(), 2);
    }
}
