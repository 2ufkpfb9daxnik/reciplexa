//! Minimal Video backend — Motion IR track sampling over time (Phase 12).
//!
//! Preview vs Final use distinct sampling modes / rates; losses are never silent.

#![forbid(unsafe_code)]

pub mod sample;
pub mod writer;

use reciplexa_motion::SampleMode;

pub use sample::{
    sample_timeline_sequence, FrameSample, SampledSequence, TrackSample, VideoLoss, VideoLossKind,
};
pub use writer::{
    export_png_sequence, rasterize_sampled_sequence, write_png_sequence, VideoPipelineError,
    VideoRasterFrame, VideoRasterSequence, WriteError,
};

/// Video output intent (mirrors backend ProfileKind for Motion sampling).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VideoProfile {
    /// Low-latency Preview: interpolated samples allowed.
    Preview,
    /// Final: strict / higher rate; approximations reported or rejected.
    Final,
}

impl VideoProfile {
    pub fn sample_mode(self) -> SampleMode {
        match self {
            Self::Preview => SampleMode::Preview,
            Self::Final => SampleMode::Strict,
        }
    }

    pub fn default_fps(self) -> f64 {
        match self {
            Self::Preview => 12.0,
            Self::Final => 30.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoOptions {
    pub profile: VideoProfile,
    pub fps: f64,
}

impl VideoOptions {
    pub fn preview() -> Self {
        Self {
            profile: VideoProfile::Preview,
            fps: VideoProfile::Preview.default_fps(),
        }
    }

    pub fn final_out() -> Self {
        Self {
            profile: VideoProfile::Final,
            fps: VideoProfile::Final.default_fps(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoError {
    NonFiniteFps,
    NonPositiveFps,
    EmptyTimeline,
}
