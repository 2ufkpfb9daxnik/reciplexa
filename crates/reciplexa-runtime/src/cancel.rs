//! Cancellation tokens — distinct from failure.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancellationToken(u64);

impl CancellationToken {
    pub const NONE: Self = Self(0);

    pub const fn id(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct CancellationTokenSource {
    token: CancellationToken,
    cancelled: bool,
}

impl CancellationTokenSource {
    pub fn new(id: u64) -> Self {
        Self {
            token: CancellationToken(id),
            cancelled: false,
        }
    }

    pub fn token(&self) -> CancellationToken {
        self.token
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancel_flips_flag() {
        let mut src = CancellationTokenSource::new(1);
        assert!(!src.is_cancelled());
        src.cancel();
        assert!(src.is_cancelled());
    }
}
