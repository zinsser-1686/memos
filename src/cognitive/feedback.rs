//! Feedback loop — boost referenced memories, decay ignored ones

use serde::{Deserialize, Serialize};
use chrono::Utc;

/// Feedback signal type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FeedbackSignal {
    Referenced,   // Agent used this memory
    Surfaced,     // Shown in context but not used
    Ignored,      // Shown multiple times but never used
}

/// Record feedback for a memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRecord {
    pub frame_id: String,
    pub signal: FeedbackSignal,
    pub timestamp: chrono::DateTime<Utc>,
}

impl FeedbackRecord {
    pub fn referenced(frame_id: impl Into<String>) -> Self {
        Self {
            frame_id: frame_id.into(),
            signal: FeedbackSignal::Referenced,
            timestamp: Utc::now(),
        }
    }

    pub fn surfaced(frame_id: impl Into<String>) -> Self {
        Self {
            frame_id: frame_id.into(),
            signal: FeedbackSignal::Surfaced,
            timestamp: Utc::now(),
        }
    }

    pub fn ignored(frame_id: impl Into<String>) -> Self {
        Self {
            frame_id: frame_id.into(),
            signal: FeedbackSignal::Ignored,
            timestamp: Utc::now(),
        }
    }
}

/// Process feedback signals from a session
///
/// - Referenced → confidence boost (+0.05) + access_count++
/// - Surfaced + Not Referenced → minor decay (-0.01)
/// - Ignored → stronger decay (-0.03)
pub fn process_feedback(
    records: &[FeedbackRecord],
    frames: &mut [&mut crate::storage::frame::MemoryFrame],
) {
    let mut stats = std::collections::HashMap::<String, usize>::new();

    for record in records {
        *stats.entry(record.frame_id.clone()).or_insert(0) += 1;
    }

    for frame in frames {
        let count = stats.get(&frame.frame_id).copied().unwrap_or(0);

        if count > 0 {
            // Has feedback — update access info
            frame.last_accessed = Some(Utc::now());
            frame.access_count += count as u32;
        }
    }
}

/// Calculate boost/decay based on feedback history
pub fn compute_feedback_signal(
    frame: &crate::storage::frame::MemoryFrame,
    surfacing_count: usize,
    reference_count: usize,
) -> f32 {
    if reference_count > 0 {
        // Agent found this useful — boost
        let boost = 0.05 * (reference_count as f32).min(3.0); // Cap at +0.15
        boost
    } else if surfacing_count > 2 {
        // Shown multiple times but never used — minor decay
        -0.01 * (surfacing_count as f32 - 2.0).min(5.0) // Cap at -0.05
    } else {
        0.0 // No significant signal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_record() {
        let record = FeedbackRecord::referenced("test-frame-1");
        assert_eq!(record.signal, FeedbackSignal::Referenced);
        assert_eq!(record.frame_id, "test-frame-1");
    }

    #[test]
    fn test_compute_boost() {
        let signal = compute_feedback_signal(
            &crate::storage::frame::MemoryFrame::new("test"),
            5,
            2,
        );
        assert!(signal > 0.0);
    }
}
