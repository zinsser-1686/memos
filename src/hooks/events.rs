//! Hook events and triggers

/// Hook trigger types
#[derive(Debug, Clone)]
pub enum HookTrigger {
    /// Before LLM call
    PreToolUse,
    /// After LLM call
    PostToolUse,
    /// On session start
    SessionStart,
    /// On session stop
    SessionStop,
    /// Before context compaction
    PreCompact,
    /// After compaction
    PostCompact,
    /// Before prompt build
    BeforePromptBuild,
    /// User prompt submitted
    UserPromptSubmit,
}
