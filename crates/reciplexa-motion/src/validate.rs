//! Motion IR validation (NaN/Inf forbidden; empty loop ranges rejected).

use crate::time::TimeMs;
use crate::track::MotionTrack;
use crate::transform::{RangePolicy, TimeTransform};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MotionValidationError {
    NonFiniteNumber(&'static str),
    EmptyKeyframes,
    UnsortedKeyframes,
    EmptySamples,
    BadSampleRate,
    EmptySourceRange,
    ScaleFactorInvalid,
}

pub fn validate_track_f64(track: &MotionTrack<f64>) -> Result<(), MotionValidationError> {
    match track {
        MotionTrack::Constant(v) => {
            if !v.is_finite() {
                return Err(MotionValidationError::NonFiniteNumber("constant"));
            }
        }
        MotionTrack::Keyframes(frames) => {
            if frames.is_empty() {
                return Err(MotionValidationError::EmptyKeyframes);
            }
            let mut prev = TimeMs::ZERO;
            for (i, f) in frames.iter().enumerate() {
                if !f.value.is_finite() {
                    return Err(MotionValidationError::NonFiniteNumber("keyframe"));
                }
                if i > 0 && f.at.0 < prev.0 {
                    return Err(MotionValidationError::UnsortedKeyframes);
                }
                prev = f.at;
            }
        }
        MotionTrack::Samples { rate_hz, values } => {
            if !rate_hz.is_finite() || *rate_hz <= 0.0 {
                return Err(MotionValidationError::BadSampleRate);
            }
            if values.is_empty() {
                return Err(MotionValidationError::EmptySamples);
            }
            for v in values {
                if !v.is_finite() {
                    return Err(MotionValidationError::NonFiniteNumber("sample"));
                }
            }
        }
    }
    Ok(())
}

pub fn validate_transform(tf: &TimeTransform) -> Result<(), MotionValidationError> {
    match tf {
        TimeTransform::Identity
        | TimeTransform::Offset(_)
        | TimeTransform::Reverse { .. }
        | TimeTransform::Freeze(_) => Ok(()),
        TimeTransform::Scale { factor } => {
            if !factor.is_finite() || *factor <= 0.0 {
                Err(MotionValidationError::ScaleFactorInvalid)
            } else {
                Ok(())
            }
        }
        TimeTransform::Compose(a, b) => {
            validate_transform(a)?;
            validate_transform(b)
        }
    }
}

pub fn validate_source_range(
    start: TimeMs,
    end: TimeMs,
    policy: RangePolicy,
) -> Result<(), MotionValidationError> {
    if start.0 >= end.0 {
        // Loop over empty range is explicitly a validation error.
        let _ = policy;
        return Err(MotionValidationError::EmptySourceRange);
    }
    Ok(())
}
