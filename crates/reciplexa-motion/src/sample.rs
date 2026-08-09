//! Deterministic track sampling (TEST-IR-007).

use crate::easing::Easing;
use crate::time::TimeMs;
use crate::track::{Keyframe, MotionTrack};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleMode {
    /// Allow keyframe interpolation and sample playback.
    Preview,
    /// Reject approximate fitting; only exact Constant / exact keyframe hits / sample bins.
    Strict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SampleError {
    EmptyKeyframes,
    NonFiniteRate,
    EmptySamples,
    StrictApproximation,
    NonFiniteValue,
}

/// Sample an `f64` track at time `t`.
pub fn sample_f64(track: &MotionTrack<f64>, t: TimeMs, mode: SampleMode) -> Result<f64, SampleError> {
    match track {
        MotionTrack::Constant(v) => {
            ensure_finite(*v)?;
            Ok(*v)
        }
        MotionTrack::Keyframes(frames) => sample_keyframes(frames, t, mode),
        MotionTrack::Samples { rate_hz, values } => sample_uniform(rate_hz, values, t, mode),
    }
}

fn ensure_finite(v: f64) -> Result<(), SampleError> {
    if v.is_finite() {
        Ok(())
    } else {
        Err(SampleError::NonFiniteValue)
    }
}

fn sample_keyframes(
    frames: &[Keyframe<f64>],
    t: TimeMs,
    mode: SampleMode,
) -> Result<f64, SampleError> {
    if frames.is_empty() {
        return Err(SampleError::EmptyKeyframes);
    }
    for f in frames {
        ensure_finite(f.value)?;
    }
    if t.0 <= frames[0].at.0 {
        return Ok(frames[0].value);
    }
    if let Some(last) = frames.last() {
        if t.0 >= last.at.0 {
            return Ok(last.value);
        }
    }
    for w in frames.windows(2) {
        let a = &w[0];
        let b = &w[1];
        if t.0 >= a.at.0 && t.0 <= b.at.0 {
            if t.0 == a.at.0 {
                return Ok(a.value);
            }
            if t.0 == b.at.0 {
                return Ok(b.value);
            }
            if mode == SampleMode::Strict {
                return Err(SampleError::StrictApproximation);
            }
            let span = b.at.0.saturating_sub(a.at.0).max(1) as f64;
            let u = (t.0 - a.at.0) as f64 / span;
            let e = a.easing_to_next.apply(u);
            return Ok(a.value + (b.value - a.value) * e);
        }
    }
    Ok(frames.last().unwrap().value)
}

fn sample_uniform(
    rate_hz: &f64,
    values: &[f64],
    t: TimeMs,
    mode: SampleMode,
) -> Result<f64, SampleError> {
    if !rate_hz.is_finite() || *rate_hz <= 0.0 {
        return Err(SampleError::NonFiniteRate);
    }
    if values.is_empty() {
        return Err(SampleError::EmptySamples);
    }
    for v in values {
        ensure_finite(*v)?;
    }
    let idx_f = t.as_secs_f64() * rate_hz;
    let idx = idx_f.floor() as usize;
    if idx >= values.len() {
        return Ok(*values.last().unwrap());
    }
    if mode == SampleMode::Strict {
        // Exact bin only — no sub-sample lerp.
        return Ok(values[idx]);
    }
    if idx + 1 >= values.len() {
        return Ok(values[idx]);
    }
    let frac = idx_f - idx as f64;
    Ok(values[idx] + (values[idx + 1] - values[idx]) * frac)
}

/// Re-export easing apply for callers that only interpolate manually.
pub fn ease(easing: Easing, t: f64) -> f64 {
    easing.apply(t)
}
