//! Memory Frame schema — the core data structure for a single memory

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Memory content types with different decay characteristics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryType {
    /// Architectural/important decisions — never decay
    Decision,
    /// Static reference knowledge — never decay
    Hub,
    /// Negative patterns — never decay
    Antipattern,
    /// Research findings — 90 day half-life
    Research,
    /// Project context — 120 day half-life
    Project,
    /// Session handoffs — 30 day half-life
    Handoff,
    /// Work progress — 45 day half-life
    Progress,
    /// General notes — 60 day half-life
    Note,
}

impl Default for MemoryType {
    fn default() -> Self {
        MemoryType::Note
    }
}

impl MemoryType {
    pub fn baseline_confidence(&self) -> f32 {
        match self {
            MemoryType::Decision => 0.85,
            MemoryType::Hub => 0.80,
            MemoryType::Antipattern => 0.75,
            MemoryType::Research => 0.70,
            MemoryType::Project => 0.65,
            MemoryType::Handoff => 0.60,
            MemoryType::Progress => 0.50,
            MemoryType::Note => 0.50,
        }
    }

    pub fn half_life_days(&self) -> Option<i64> {
        match self {
            // Never decay
            MemoryType::Decision => None,
            MemoryType::Hub => None,
            MemoryType::Antipattern => None,
            // Linear decay with half-life
            MemoryType::Research => Some(90),
            MemoryType::Project => Some(120),
            MemoryType::Handoff => Some(30),
            MemoryType::Progress => Some(45),
            MemoryType::Note => Some(60),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryType::Decision => "decision",
            MemoryType::Hub => "hub",
            MemoryType::Antipattern => "antipattern",
            MemoryType::Research => "research",
            MemoryType::Project => "project",
            MemoryType::Handoff => "handoff",
            MemoryType::Progress => "progress",
            MemoryType::Note => "note",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "decision" => MemoryType::Decision,
            "hub" => MemoryType::Hub,
            "antipattern" => MemoryType::Antipattern,
            "research" => MemoryType::Research,
            "project" => MemoryType::Project,
            "handoff" => MemoryType::Handoff,
            "progress" => MemoryType::Progress,
            "note" => MemoryType::Note,
            _ => MemoryType::Note,
        }
    }
}

/// Memory lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LifecycleState {
    Active,
    Archived,
    Snoozed,
    Forgotten,
}

impl Default for LifecycleState {
    fn default() -> Self {
        LifecycleState::Active
    }
}

impl LifecycleState {
    pub fn as_str(&self) -> &'static str {
        match self {
            LifecycleState::Active => "active",
            LifecycleState::Archived => "archived",
            LifecycleState::Snoozed => "snoozed",
            LifecycleState::Forgotten => "forgotten",
        }
    }
}

/// Core memory frame — immutable once committed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFrame {
    /// Unique frame identifier
    pub frame_id: String,

    /// Memory type (determines decay behavior)
    pub memory_type: MemoryType,

    /// Short title/summary
    pub title: String,

    /// Raw text content
    pub content: String,

    /// Semantic chunk for embedding
    pub fragment: String,

    /// Extracted keywords (3–7)
    pub keywords: Vec<String>,

    /// Extracted tags (3–5)
    pub tags: Vec<String>,

    /// Generated context description
    pub context_desc: String,

    /// Stable URI for addressing
    pub uri: String,

    /// Source tracking
    pub source_session: Option<String>,
    pub source_context: Option<String>,
    pub observed_at: Option<DateTime<Utc>>,

    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    /// A-MEM signals
    pub confidence: f32,
    pub access_count: u32,
    pub last_accessed: Option<DateTime<Utc>>,
    pub decay_started: Option<DateTime<Utc>>,

    /// Lifecycle
    pub lifecycle: LifecycleState,
    pub snooze_until: Option<DateTime<Utc>>,
    pub pin_priority: f32,

    /// Conflict resolution
    pub replaces: Vec<String>,
    pub replaced_by: Option<String>,

    /// Vector embedding
    pub vector: Vec<f32>,

    /// Additional metadata
    pub metadata: serde_json::Value,
}

impl MemoryFrame {
    pub fn new(content: impl Into<String>) -> Self {
        let now = Utc::now();
        let content = content.into();
        Self {
            frame_id: Uuid::new_v4().to_string(),
            memory_type: MemoryType::default(),
            title: String::new(),
            content: content.clone(),
            fragment: content.clone(),
            keywords: Vec::new(),
            tags: Vec::new(),
            context_desc: String::new(),
            uri: String::new(),
            source_session: None,
            source_context: None,
            observed_at: None,
            created_at: now,
            updated_at: now,
            confidence: MemoryType::default().baseline_confidence(),
            access_count: 0,
            last_accessed: None,
            decay_started: None,
            lifecycle: LifecycleState::default(),
            snooze_until: None,
            pin_priority: 0.0,
            replaces: Vec::new(),
            replaced_by: None,
            vector: Vec::new(),
            metadata: serde_json::Value::Object(Default::default()),
        }
    }

    /// Compute recency decay based on content type
    pub fn recency_decay(&self) -> f32 {
        let half_life = match self.memory_type.half_life_days() {
            Some(days) => days,
            None => return 1.0, // Never decay
        };

        let age = match self.last_accessed {
            Some(last) => (Utc::now() - last).num_days() as f32,
            None => (Utc::now() - self.created_at).num_days() as f32,
        };

        (0.5_f32).powf(age / half_life as f32)
    }

    /// Quality multiplier based on document structure
    pub fn quality_multiplier(&self) -> f32 {
        let length_norm = {
            let len = self.content.len() as f32;
            let normalized = 1.0 / (1.0 + 0.5 * (len / 500.0).log2().max(0.0));
            normalized.max(0.3) // Floor at 30%
        };

        // Boost for good structure
        let structure_boost = if self.fragment.contains('\n') { 1.05 } else { 1.0 };
        let keyword_boost = if !self.keywords.is_empty() { 1.05 } else { 1.0 };

        length_norm * structure_boost * keyword_boost
    }

    /// Pin boost if pinned
    pub fn pin_boost(&self) -> f32 {
        if self.pin_priority > 0.0 {
            self.pin_priority.min(0.3)
        } else {
            0.0
        }
    }
}

/// Options for putting a memory
#[derive(Debug, Clone, Default)]
pub struct PutOptions {
    pub title: Option<String>,
    pub memory_type: Option<MemoryType>,
    pub tags: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    pub context_desc: Option<String>,
    pub uri: Option<String>,
    pub source_session: Option<String>,
    pub source_context: Option<String>,
    pub observed_at: Option<DateTime<Utc>>,
    pub replaces: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

impl PutOptions {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn memory_type(mut self, mtype: impl Into<String>) -> Self {
        self.memory_type = Some(MemoryType::from_str(&mtype.into()));
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    pub fn source_session(mut self, session: impl Into<String>) -> Self {
        self.source_session = Some(session.into());
        self
    }

    pub fn replaces(mut self, ids: Vec<String>) -> Self {
        self.replaces = Some(ids);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_type_defaults() {
        let frame = MemoryFrame::new("test content");
        assert_eq!(frame.memory_type, MemoryType::Note);
        assert_eq!(frame.confidence, 0.50);
    }

    #[test]
    fn test_recency_decay_never() {
        let frame = MemoryFrame::new("decision content");
        assert_eq!(frame.recency_decay(), 1.0);
    }

    #[test]
    fn test_quality_multiplier() {
        let frame = MemoryFrame::new("test content");
        let qm = frame.quality_multiplier();
        assert!(qm >= 0.3);
    }
}
