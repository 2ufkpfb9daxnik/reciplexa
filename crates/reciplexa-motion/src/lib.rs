//! Motion IR — time domain, animation tracks, timeline, and deterministic sampling.
//!
//! Phase 12 foundation: static and dynamic content share document identity upstream;
//! this crate owns the `Time -> value` track model and Preview/Final sampling policy.

#![forbid(unsafe_code)]

pub mod easing;
pub mod sample;
pub mod time;
pub mod timeline;
pub mod track;
pub mod transform;
pub mod validate;

pub use easing::Easing;
pub use sample::{sample_f64, SampleError, SampleMode};
pub use time::{DurationMs, TimeMs};
pub use timeline::{MotionTimeline, TemporalPlacement, TimelineTrack};
pub use track::{Keyframe, MotionTrack};
pub use transform::{
    apply_range_policy, RangePolicy, RangePolicyError, TimeTransform,
};
pub use validate::{
    validate_source_range, validate_track_f64, validate_transform, MotionValidationError,
};
