//! Motion ↔ GUI Preview sync: playhead → sampled values under Preview vs Final policy.

use crate::sample::{sample_f64, SampleError, SampleMode};
use crate::timeline::{MotionTimeline, TimelineTrack};
use crate::transform::apply_range_policy;
use crate::time::TimeMs;

/// Sampling profile for interactive Preview vs Final export bake.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyncProfile {
    Preview,
    Final,
}

impl SyncProfile {
    pub fn sample_mode(self) -> SampleMode {
        match self {
            Self::Preview => SampleMode::Preview,
            Self::Final => SampleMode::Strict,
        }
    }
}

/// One track's contribution at the current playhead.
#[derive(Debug, Clone, PartialEq)]
pub struct SyncedTrackValue {
    pub track_id: u64,
    pub name: String,
    pub value: Option<f64>,
    pub error: Option<SampleError>,
}

/// Snapshot of all timeline tracks at a playhead under a sync profile.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayheadSync {
    pub playhead: TimeMs,
    pub profile: SyncProfile,
    pub values: Vec<SyncedTrackValue>,
}

/// Map `timeline.playhead` to sampled values for `profile` (Preview vs Final).
pub fn sync_playhead(timeline: &MotionTimeline, profile: SyncProfile) -> PlayheadSync {
    sync_at(timeline, timeline.playhead, profile)
}

/// Map an explicit playhead time to sampled values for `profile`.
pub fn sync_at(timeline: &MotionTimeline, playhead: TimeMs, profile: SyncProfile) -> PlayheadSync {
    let mode = profile.sample_mode();
    let mut values = Vec::with_capacity(timeline.tracks.len());
    for tr in &timeline.tracks {
        values.push(sample_synced_track(tr, playhead, mode));
    }
    PlayheadSync {
        playhead,
        profile,
        values,
    }
}

/// Compare Preview vs Final at the same playhead — differences are never silent.
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewFinalDiff {
    pub playhead: TimeMs,
    pub preview: PlayheadSync,
    pub final_sync: PlayheadSync,
    /// Track ids where Preview produced a value but Final errored (or values differ).
    pub divergent_track_ids: Vec<u64>,
}

pub fn compare_preview_final(timeline: &MotionTimeline) -> PreviewFinalDiff {
    let preview = sync_playhead(timeline, SyncProfile::Preview);
    let final_sync = sync_playhead(timeline, SyncProfile::Final);
    let mut divergent_track_ids = Vec::new();
    for (p, f) in preview.values.iter().zip(final_sync.values.iter()) {
        let diverge = match (&p.value, &f.value, &f.error) {
            (Some(a), Some(b), _) => (a - b).abs() > 1e-9,
            (Some(_), None, Some(_)) => true,
            (Some(_), None, None) => true,
            (None, Some(_), _) => true,
            _ => p.error != f.error,
        };
        if diverge {
            divergent_track_ids.push(p.track_id);
        }
    }
    PreviewFinalDiff {
        playhead: timeline.playhead,
        preview,
        final_sync,
        divergent_track_ids,
    }
}

fn sample_synced_track(
    tr: &TimelineTrack,
    parent_t: TimeMs,
    mode: SampleMode,
) -> SyncedTrackValue {
    let p = &tr.placement;
    let local = match apply_range_policy(parent_t, p.parent_start, p.parent_end, p.range_policy) {
        Ok(Some(local)) => local,
        Ok(None) => {
            return SyncedTrackValue {
                track_id: tr.id,
                name: tr.name.clone(),
                value: None,
                error: None,
            };
        }
        Err(_) => {
            return SyncedTrackValue {
                track_id: tr.id,
                name: tr.name.clone(),
                value: None,
                error: Some(SampleError::EmptyKeyframes),
            };
        }
    };
    let child_t = p.transform.apply(TimeMs(
        p.child_source_start
            .0
            .saturating_add(local.0)
            .min(p.child_source_end.0),
    ));
    match sample_f64(&tr.track, child_t, mode) {
        Ok(value) => SyncedTrackValue {
            track_id: tr.id,
            name: tr.name.clone(),
            value: Some(value),
            error: None,
        },
        Err(error) => SyncedTrackValue {
            track_id: tr.id,
            name: tr.name.clone(),
            value: None,
            error: Some(error),
        },
    }
}
