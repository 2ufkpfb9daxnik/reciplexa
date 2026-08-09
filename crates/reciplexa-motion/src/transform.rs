//! Analytic time transforms (not arbitrary functions).

use crate::time::{DurationMs, TimeMs};

/// How to behave outside the child source range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RangePolicy {
    /// No contribution outside range.
    #[default]
    Transparent,
    /// Clamp to nearest endpoint value.
    Hold,
    /// Euclidean modulo loop of source range.
    Loop,
    /// Alternate forward/backward loops.
    PingPong,
    /// Sampling outside range is a validation/runtime failure.
    Failure,
}

/// Spec Time Transform IR.
#[derive(Debug, Clone, PartialEq)]
pub enum TimeTransform {
    Identity,
    Offset(DurationMs),
    /// Multiply child time by `factor` (finite, > 0).
    Scale {
        factor: f64,
    },
    Reverse {
        source_len: DurationMs,
    },
    Freeze(TimeMs),
    Compose(Box<TimeTransform>, Box<TimeTransform>),
}

impl TimeTransform {
    /// Map parent-local time into child source time before range policy.
    pub fn apply(&self, t: TimeMs) -> TimeMs {
        match self {
            Self::Identity => t,
            Self::Offset(d) => t.saturating_add(*d),
            Self::Scale { factor } => {
                let ms = (t.0 as f64 * factor).round();
                TimeMs(if ms <= 0.0 { 0 } else { ms as u64 })
            }
            Self::Reverse { source_len } => {
                if t.0 >= source_len.0 {
                    TimeMs(0)
                } else {
                    TimeMs(source_len.0 - t.0)
                }
            }
            Self::Freeze(at) => *at,
            Self::Compose(outer, inner) => outer.apply(inner.apply(t)),
        }
    }
}

/// Map `t` into `[start, end)` under `policy`. Returns `None` for Transparent/Failure miss.
pub fn apply_range_policy(
    t: TimeMs,
    start: TimeMs,
    end: TimeMs,
    policy: RangePolicy,
) -> Result<Option<TimeMs>, RangePolicyError> {
    if start.0 >= end.0 {
        return Err(RangePolicyError::EmptySourceRange);
    }
    let len = end.0 - start.0;
    if t.0 >= start.0 && t.0 < end.0 {
        return Ok(Some(TimeMs(t.0 - start.0)));
    }
    match policy {
        RangePolicy::Transparent => Ok(None),
        RangePolicy::Hold => {
            if t.0 < start.0 {
                Ok(Some(TimeMs(0)))
            } else {
                Ok(Some(TimeMs(len - 1)))
            }
        }
        RangePolicy::Loop => {
            let offset = if t.0 < start.0 {
                // before: wrap from end
                let back = (start.0 - t.0) % len;
                if back == 0 {
                    0
                } else {
                    len - back
                }
            } else {
                (t.0 - start.0) % len
            };
            Ok(Some(TimeMs(offset)))
        }
        RangePolicy::PingPong => {
            let cycle = len.saturating_mul(2).saturating_sub(2).max(1);
            let raw = if t.0 < start.0 {
                0
            } else {
                (t.0 - start.0) % cycle
            };
            let local = if raw < len { raw } else { cycle - raw };
            Ok(Some(TimeMs(local.min(len - 1))))
        }
        RangePolicy::Failure => Err(RangePolicyError::OutOfRange),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RangePolicyError {
    EmptySourceRange,
    OutOfRange,
}
