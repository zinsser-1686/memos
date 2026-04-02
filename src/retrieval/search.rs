//! Search pipeline — BM25 + vector hybrid search with RRF

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Query intent classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueryIntent {
    /// "why did we decide X" — causal reasoning
    Why,
    /// "when was Y discussed" — temporal
    When,
    /// "who worked on Z" — entity lookup
    Entity,
    /// General recall
    What,
}

impl Default for QueryIntent {
    fn default() -> Self {
        QueryIntent::What
    }
}

/// Search request
#[derive(Debug, Clone)]
pub struct SearchRequest {
    /// The query string
    pub query: String,
    /// Maximum results to return
    pub top_k: usize,
    /// Optional intent hint
    pub intent: Option<QueryIntent>,
    /// Whether to include causal graph expansion
    pub expand_graph: bool,
    /// Whether to rerank results
    pub rerank: bool,
    /// Snippet character limit
    pub snippet_chars: usize,
}

impl Default for SearchRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            top_k: 10,
            intent: None,
            expand_graph: true,
            rerank: true,
            snippet_chars: 200,
        }
    }
}

/// A search result hit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub frame_id: String,
    pub title: String,
    pub snippet: String,
    pub memory_type: String,
    pub score: f32,
    pub confidence: f32,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// RRF (Reciprocal Rank Fusion) merge
pub fn rrf_merge<T: Ord>(ranked_lists: Vec<Vec<(T, f32)>>, k: usize) -> Vec<(T, f32)> {
    use std::collections::BTreeMap;

    let mut scores: BTreeMap<T, f32> = BTreeMap::new();

    for list in ranked_lists {
        for (id, _score) in list.iter().enumerate() {
            let rrf_score = 1.0 / (k + id + 1) as f32;
            *scores.entry(id.clone()).or_insert(0.0) += rrf_score;
        }
    }

    // Sort by RRF score descending
    let mut sorted: Vec<_> = scores.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    sorted.into_iter().take(k).map(|(id, score)| (id, score)).collect()
}

/// MMR (Maximum Marginal Relevance) diversity filter
pub fn mmr_filter<T: Clone>(candidates: Vec<T>, scores: Vec<f32>, target: usize, lambda: f32, threshold: f32) -> Vec<T> {
    if candidates.len() <= target {
        return candidates;
    }

    let mut selected = Vec::with_capacity(target);
    let mut remaining: Vec<_> = candidates.into_iter().zip(scores.into_iter()).collect();

    // Select first best
    if let Some(pos) = remaining.iter().position(|(_, s)| *s == remaining.iter().map(|(_, s)| *s).fold(f32::MIN, f32::max) {
        let (item, score) = remaining.remove(pos);
        selected.push((item, score));
    }

    while selected.len() < target && !remaining.is_empty() {
        let mut best_idx = 0;
        let mut best_combined = f32::MIN;

        for (i, (item, relevance)) in remaining.iter().enumerate() {
            // Relevance score
            let relevance_score = relevance;

            // Diversity score (max similarity to selected)
            let diversity_score = {
                let mut max_sim = 0.0f32;
                for (selected_item, _) in &selected {
                    let sim = jaccard_similarity(
                        format!("{:?}", item).as_bytes(),
                        format!("{:?}", selected_item).as_bytes(),
                    );
                    max_sim = max_sim.max(sim);
                }
                max_sim
            };

            // Combined score
            let combined = lambda * relevance_score - (1.0 - lambda) * diversity_score;

            if combined > best_combined {
                best_combined = combined;
                best_idx = i;
            }
        }

        let (item, score) = remaining.remove(best_idx);
        selected.push((item, score));
    }

    selected.into_iter().map(|(item, _)| item).collect()
}

/// Simple Jaccard similarity for diversity
fn jaccard_similarity(a: &[u8], b: &[u8]) -> f32 {
    let set_a: std::collections::HashSet<_> = a.iter().collect();
    let set_b: std::collections::HashSet<_> = b.iter().collect();

    let intersection = set_a.intersection(&set_b).count() as f32;
    let union = set_a.union(&set_b).count() as f32;

    if union == 0.0 { 0.0 } else { intersection / union }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_merge() {
        let list1 = vec![("a", 0.9), ("b", 0.8), ("c", 0.7)];
        let list2 = vec![("b", 0.95), ("d", 0.85), ("a", 0.6)];

        let merged = rrf_merge(vec![list1, list2], 4);
        assert!(!merged.is_empty());
    }

    #[test]
    fn test_mmr_filter() {
        let items = vec!["a", "b", "c", "d", "e"];
        let scores = vec![0.9, 0.8, 0.7, 0.6, 0.5];

        let filtered = mmr_filter(items, scores, 3, 0.7, 0.6);
        assert_eq!(filtered.len(), 3);
    }
}
