//! Sample MotionTimeline tracks across a frame grid.

use reciplexa_motion::{
    apply_range_policy, sample_f64, MotionTimeline, SampleError, SampleMode, TimeMs, TimelineTrack,
};

use crate::{VideoError, VideoOptions, VideoProfile};

/// One sampled track value at a frame time.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackSample {
    pub track_id: u64,
    pub value: f64,
}

/// All track samples at one playhead time.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameSample {
    pub time: TimeMs,
    pub tracks: Vec<TrackSample>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoLossKind {
    /// Strict Final sampling refused an approximation; frame skipped or held.
    StrictApproximationSkipped,
    /// Track inactive under Transparent range policy.
    TrackTransparent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoLoss {
    pub kind: VideoLossKind,
    pub profile: VideoProfile,
    pub track_id: Option<u64>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SampledSequence {
    pub profile: VideoProfile,
    pub fps: f64,
    pub frames: Vec<FrameSample>,
    pub losses: Vec<VideoLoss>,
}

/// Sample every timeline track on a uniform frame grid for `opts.fps`.
pub fn sample_timeline_sequence(
    timeline: &MotionTimeline,
    opts: &VideoOptions,
) -> Result<SampledSequence, VideoError> {
    if !opts.fps.is_finite() {
        return Err(VideoError::NonFiniteFps);
    }
    if opts.fps <= 0.0 {
        return Err(VideoError::NonPositiveFps);
    }
    if timeline.duration.0 == 0 && timeline.tracks.is_empty() {
        return Err(VideoError::EmptyTimeline);
    }
    let mode = opts.profile.sample_mode();
    let frame_count = frame_count_for(timeline.duration.0, opts.fps);
    let mut frames = Vec::with_capacity(frame_count);
    let mut losses = Vec::new();
    for i in 0..frame_count {
        let t = TimeMs(((i as f64) * 1000.0 / opts.fps).round() as u64);
        let t = TimeMs(t.0.min(timeline.duration.0));
        let mut tracks = Vec::new();
        for tr in &timeline.tracks {
            match sample_placed_track(tr, t, mode) {
                Ok(Some(value)) => tracks.push(TrackSample {
                    track_id: tr.id,
                    value,
                }),
                Ok(None) => {
                    losses.push(VideoLoss {
                        kind: VideoLossKind::TrackTransparent,
                        profile: opts.profile,
                        track_id: Some(tr.id),
                        detail: "track inactive under Transparent range policy".into(),
                    });
                }
                Err(SampleError::StrictApproximation) => {
                    losses.push(VideoLoss {
                        kind: VideoLossKind::StrictApproximationSkipped,
                        profile: opts.profile,
                        track_id: Some(tr.id),
                        detail: "strict Final refused keyframe interpolation".into(),
                    });
                }
                Err(_) => {
                    // Other sample errors: surface as empty contribution with explicit loss.
                    losses.push(VideoLoss {
                        kind: VideoLossKind::StrictApproximationSkipped,
                        profile: opts.profile,
                        track_id: Some(tr.id),
                        detail: "track sample failed".into(),
                    });
                }
            }
        }
        frames.push(FrameSample { time: t, tracks });
    }
    Ok(SampledSequence {
        profile: opts.profile,
        fps: opts.fps,
        frames,
        losses,
    })
}

fn frame_count_for(duration_ms: u64, fps: f64) -> usize {
    if duration_ms == 0 {
        return 1;
    }
    let secs = duration_ms as f64 / 1000.0;
    ((secs * fps).ceil() as usize).max(1)
}

fn sample_placed_track(
    tr: &TimelineTrack,
    parent_t: TimeMs,
    mode: SampleMode,
) -> Result<Option<f64>, SampleError> {
    let p = &tr.placement;
    // Map parent time into placement window, then into child source time.
    let local = match apply_range_policy(
        parent_t,
        p.parent_start,
        p.parent_end,
        p.range_policy,
    ) {
        Ok(Some(local)) => local,
        Ok(None) => return Ok(None),
        Err(_) => return Ok(None),
    };
    let child_t = p.transform.apply(TimeMs(
        p.child_source_start
            .0
            .saturating_add(local.0)
            .min(p.child_source_end.0),
    ));
    sample_f64(&tr.track, child_t, mode).map(Some)
}
