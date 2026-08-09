//! Discrete millisecond time domain shared by Preview and Final sampling.

/// Instant on the motion timeline (milliseconds from timeline origin).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TimeMs(pub u64);

impl TimeMs {
    pub const ZERO: Self = Self(0);

    pub fn saturating_add(self, d: DurationMs) -> Self {
        Self(self.0.saturating_add(d.0))
    }

    pub fn saturating_sub(self, d: DurationMs) -> Self {
        Self(self.0.saturating_sub(d.0))
    }

    pub fn as_secs_f64(self) -> f64 {
        self.0 as f64 / 1000.0
    }
}

/// Non-negative duration in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DurationMs(pub u64);

impl DurationMs {
    pub const ZERO: Self = Self(0);

    pub fn from_secs_f64(secs: f64) -> Option<Self> {
        if !secs.is_finite() || secs < 0.0 {
            return None;
        }
        Some(Self((secs * 1000.0).round() as u64))
    }
}
