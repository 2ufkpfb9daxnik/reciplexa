//! `rpx.std.motion` — thin façade over `reciplexa-motion`.

use std::fmt;

use reciplexa_identity::document::StableNodeId;
pub use reciplexa_motion::{
    DurationMs, Easing, Keyframe, MotionTimeline, MotionTrack, TemporalPlacement, TimeMs,
    TimelineTrack,
};

/// Std package timeline wrapper retaining a StableNodeId for document binding.
#[derive(Debug, Clone, PartialEq)]
pub struct Timeline {
    pub id: StableNodeId,
    pub inner: MotionTimeline,
}

impl Timeline {
    pub fn new(id: StableNodeId, duration: DurationMs) -> Self {
        Self {
            id,
            inner: MotionTimeline {
                duration,
                playhead: TimeMs::ZERO,
                playing: false,
                tracks: Vec::new(),
            },
        }
    }

    pub fn push_track(&mut self, track: TimelineTrack) {
        self.inner.tracks.push(track);
    }

    pub fn seek(&mut self, t: TimeMs) {
        self.inner.seek(t);
    }

    pub fn play(&mut self) {
        self.inner.play();
    }

    pub fn pause(&mut self) {
        self.inner.pause();
    }

    pub fn tick(&mut self, delta_ms: u64) {
        self.inner.tick(delta_ms);
    }

    pub fn track_count(&self) -> usize {
        self.inner.tracks.len()
    }

    pub fn is_playing(&self) -> bool {
        self.inner.playing
    }
}

impl fmt::Display for Timeline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Timeline(id={}, duration_ms={}, tracks={})",
            self.id,
            self.inner.duration.0,
            self.track_count()
        )
    }
}

/// Convenience: build a f64 keyframe track with shared easing-to-next.
pub fn keyframe_track(frames: Vec<(TimeMs, f64)>, easing: Easing) -> MotionTrack<f64> {
    let frames = frames
        .into_iter()
        .map(|(at, value)| Keyframe {
            at,
            value,
            easing_to_next: easing,
        })
        .collect();
    MotionTrack::keyframes(frames)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> StableNodeId {
        StableNodeId::new(n)
    }

    #[test]
    fn timeline_facade_controls() {
        let mut tl = Timeline::new(id(1), DurationMs(1000));
        assert!(!tl.is_playing());
        assert_eq!(tl.track_count(), 0);
        let track = keyframe_track(vec![(TimeMs(0), 0.0), (TimeMs(1000), 1.0)], Easing::Linear);
        tl.push_track(TimelineTrack {
            id: 1,
            node: Some(id(2)),
            name: "opacity".into(),
            placement: TemporalPlacement::span(TimeMs(0), TimeMs(1000)),
            track,
        });
        assert_eq!(tl.track_count(), 1);
        tl.play();
        assert!(tl.is_playing());
        tl.tick(100);
        assert_eq!(tl.inner.playhead, TimeMs(100));
        tl.pause();
        assert!(!tl.is_playing());
        tl.seek(TimeMs(500));
        assert_eq!(tl.inner.playhead, TimeMs(500));
        assert!(tl.to_string().contains("Timeline"));
    }
}
