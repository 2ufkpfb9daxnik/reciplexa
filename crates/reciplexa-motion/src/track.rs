//! Motion tracks: Constant | Keyframes | Samples (spec MotionIR).

use crate::easing::Easing;
use crate::time::TimeMs;

/// One keyframe on a typed track.
#[derive(Debug, Clone, PartialEq)]
pub struct Keyframe<A> {
    pub at: TimeMs,
    pub value: A,
    pub easing_to_next: Easing,
}

/// Spec `Track<A>` for MotionIR.
#[derive(Debug, Clone, PartialEq)]
pub enum MotionTrack<A> {
    Constant(A),
    Keyframes(Vec<Keyframe<A>>),
    /// Uniform samples; `rate_hz` must be finite and > 0.
    Samples {
        rate_hz: f64,
        values: Vec<A>,
    },
}

impl<A: Clone> MotionTrack<A> {
    pub fn constant(value: A) -> Self {
        Self::Constant(value)
    }

    pub fn keyframes(frames: Vec<Keyframe<A>>) -> Self {
        Self::Keyframes(frames)
    }
}
