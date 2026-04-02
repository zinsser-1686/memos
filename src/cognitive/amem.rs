//! A-MEM (Adaptive Memory Evolution) — self-evolving memory metadata

use crate::storage::frame::MemoryFrame;

/// Extract keywords from content (3–7 specific terms)
pub fn extract_keywords(content: &str, top_k: usize) -> Vec<String> {
    // STUB: In production, use NLP keyword extraction
    // For now, extract capitalized terms and common technical words

    let words: Vec<&str> = content
        .split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '-' && c != '_')
        .filter(|w| w.len() > 3)
        .collect();

    // Simple frequency-based extraction (placeholder)
    let mut freq = std::collections::HashMap::new();
    for word in &words {
        let lower = word.to_lowercase();
        *freq.entry(lower).or_insert(0) += 1;
    }

    let mut sorted: Vec<_> = freq.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    sorted
        .into_iter()
        .take(top_k.min(7))
        .map(|(word, _)| word)
        .collect()
}

/// Extract tags from content (3–5 broad categories)
pub fn extract_tags(content: &str, top_k: usize) -> Vec<String> {
    // STUB: In production, use topic modeling or LLM-based classification
    // For now, look for common domain keywords

    let domain_keywords = [
        "architecture", "design", "api", "database", "security",
        "performance", "testing", "deployment", "monitoring", "documentation",
        "bug", "feature", "refactor", "optimization", "integration",
    ];

    let content_lower = content.to_lowercase();
    let found: Vec<_> = domain_keywords
        .iter()
        .filter(|kw| content_lower.contains(*kw))
        .take(top_k.min(5))
        .map(|s| s.to_string())
        .collect();

    if found.is_empty() {
        vec!["general".to_string()]
    } else {
        found
    }
}

/// Enrich a memory frame with A-MEM metadata
pub fn enrich_memory(frame: &mut MemoryFrame) {
    // Extract keywords
    frame.keywords = extract_keywords(&frame.content, 7);

    // Extract tags
    frame.tags = extract_tags(&frame.content, 5);

    // Generate context description (placeholder — would use LLM)
    frame.context_desc = if frame.title.is_empty() {
        frame.content.chars().take(100).collect::<String>() + "..."
    } else {
        frame.title.clone()
    };

    // Set initial confidence based on type
    frame.confidence = frame.memory_type.baseline_confidence();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_keywords() {
        let content = "Rust is a systems programming language focused on safety and performance. Memory safety without garbage collection.";
        let keywords = extract_keywords(content, 5);
        assert!(!keywords.is_empty());
    }

    #[test]
    fn test_extract_tags() {
        let content = "We need to improve the API performance and add database caching";
        let tags = extract_tags(content, 5);
        assert!(tags.contains(&"api".to_string()) || tags.contains(&"database".to_string()));
    }
}
