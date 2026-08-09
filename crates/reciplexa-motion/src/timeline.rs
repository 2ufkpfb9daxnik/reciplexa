//! Timeline container and temporal placement of child tracks.

use reciplexa_identity::document::StableNodeId;

use crate::time::{DurationMs, TimeMs};
use crate::track::MotionTrack;
use crate::transform::{RangePolicy, TimeTransform};

/// One track bound to a document node (shared identity with static content).
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineTrack {
    pub id: u64,
    pub node: Option<StableNodeId>,
    pub name: String,
    pub placement: TemporalPlacement,
    pub track: MotionTrack<f64>,
}

/// Parent active range + child source mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct TemporalPlacement {
    pub parent_start: TimeMs,
    pub parent_end: TimeMs,
    pub child_source_start: TimeMs,
    pub child_source_end: TimeMs,
    pub transform: TimeTransform,
    pub range_policy: RangePolicy,
}

impl TemporalPlacement {
    pub fn span(parent_start: TimeMs, parent_end: TimeMs) -> Self {
        Self {
            parent_start,
            parent_end,
            child_source_start: TimeMs::ZERO,
            child_source_end: TimeMs(parent_end.0.saturating_sub(parent_start.0)),
            transform: TimeTransform::Identity,
            range_policy: RangePolicy::Hold,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MotionTimeline {
    pub playhead: TimeMs,
    pub duration: DurationMs,
    pub tracks: Vec<TimelineTrack>,
    pub playing: bool,
}

impl MotionTimeline {
    pub fn seek(&mut self, t: TimeMs) {
        self.playhead = TimeMs(t.0.min(self.duration.0));
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    pub fn tick(&mut self, delta_ms: u64) {
        if !self.playing {
            return;
        }
        let next = self.playhead.0.saturating_add(delta_ms);
        if next >= self.duration.0 {
            self.playhead = TimeMs(self.duration.0);
            self.playing = false;
        } else {
            self.playhead = TimeMs(next);
        }
    }
}
