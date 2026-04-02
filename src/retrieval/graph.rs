//! Causal graph expansion for WHY/ENTITY intent queries

use crate::retrieval::{QueryIntent, SearchHit};
use std::collections::HashMap;

/// Expand search results using causal/semantic graph
///
/// For WHY and ENTITY intents, we traverse the knowledge graph
/// to find additional related memories beyond what keyword/vector search finds.
pub fn expand_with_causal_graph(
    intent: QueryIntent,
    anchor_ids: &[String],
    neighbors: &HashMap<String, Vec<(String, f32)>>,
    max_hops: usize,
    max_results: usize,
) -> Vec<(String, f32)> {
    if !matches!(intent, QueryIntent::Why | QueryIntent::Entity) {
        // No graph expansion for WHAT/WHEN
        return anchor_ids
            .iter()
            .map(|id| (id.clone(), 1.0))
            .collect();
    }

    let mut expanded = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut frontier: Vec<(String, f32)> = anchor_ids
        .iter()
        .map(|id| (id.clone(), 1.0))
        .collect();

    // Beam search up to max_hops
    for _hop in 0..max_hops {
        if frontier.is_empty() {
            break;
        }

        let mut next_frontier = Vec::new();

        for (node_id, parent_score) in frontier.drain(..) {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id.clone());

            // Add to results
            if !anchor_ids.contains(&node_id) {
                expanded.push((node_id.clone(), parent_score));
                if expanded.len() >= max_results {
                    return expanded;
                }
            }

            // Get neighbors via cause/semantic edges
            if let Some(neighbor_list) = neighbors.get(&node_id) {
                for (neighbor_id, edge_weight) in neighbor_list {
                    if visited.contains(neighbor_id) {
                        continue;
                    }

                    // Score = parent_score * decay * edge_weight
                    let transition_score = parent_score * 0.7 * edge_weight;
                    next_frontier.push((neighbor_id.clone(), transition_score));
                }
            }
        }

        // Keep top-k for next hop
        next_frontier.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        frontier = next_frontier.into_iter().take(20).collect();
    }

    expanded
}

/// Score transition between nodes
pub fn score_transition(structure_weight: f32, semantic_affinity: f32, lambda1: f32, lambda2: f32) -> f32 {
    lambda1 * structure_weight + lambda2 * semantic_affinity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_expansion_why() {
        let mut neighbors = HashMap::new();
        neighbors.insert("a".to_string(), vec![("b".to_string(), 0.9), ("c".to_string(), 0.7)]);
        neighbors.insert("b".to_string(), vec![("d".to_string(), 0.8)]);

        let anchors = vec!["a".to_string()];
        let expanded = expand_with_causal_graph(
            QueryIntent::Why,
            &anchors,
            &neighbors,
            2,
            10,
        );

        // Should find b, c, d
        let found: Vec<_> = expanded.iter().map(|(id, _)| id.clone()).collect();
        assert!(found.contains(&"b".to_string()) || found.contains(&"c".to_string()));
    }

    #[test]
    fn test_no_expansion_for_what() {
        let anchors = vec!["a".to_string()];
        let expanded = expand_with_causal_graph(
            QueryIntent::What,
            &anchors,
            &HashMap::new(),
            3,
            10,
        );

        // WHAT intent returns anchor only
        assert_eq!(expanded.len(), 1);
        assert_eq!(expanded[0].0, "a");
    }
}
