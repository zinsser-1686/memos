//! Lifecycle hooks for session events

pub mod events;

pub use events::{HookEvent, HookTrigger};

/// Hook events that trigger memory operations
#[derive(Debug, Clone)]
pub enum HookEvent {
    /// Session starting
    SessionStart,
    /// User submitted a prompt
    UserPrompt,
    /// Session ending (stop)
    SessionStop,
    /// Pre-compaction checkpoint
    PreCompact,
    /// Post-compaction
    PostCompact,
}
