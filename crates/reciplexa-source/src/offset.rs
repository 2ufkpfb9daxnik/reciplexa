//! UTF-8 byte offsets into a source buffer.

use core::fmt;

/// A zero-based UTF-8 byte offset into a source buffer.
///
/// Offsets are not character indices and must not be confused with Unicode
/// code point positions (`specification.md` §1.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ByteOffset(pub u32);

impl ByteOffset {
    pub const ZERO: Self = Self(0);

    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }

    pub fn checked_add(self, delta: u32) -> Option<Self> {
        self.0.checked_add(delta).map(Self)
    }

    pub fn saturating_add(self, delta: u32) -> Self {
        Self(self.0.saturating_add(delta))
    }
}

impl From<u32> for ByteOffset {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<ByteOffset> for usize {
    fn from(offset: ByteOffset) -> Self {
        offset.0 as usize
    }
}

impl From<ByteOffset> for u32 {
    fn from(offset: ByteOffset) -> Self {
        offset.0
    }
}

impl fmt::Display for ByteOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_default() {
        assert_eq!(ByteOffset::default(), ByteOffset::ZERO);
    }

    #[test]
    fn checked_add_overflows() {
        assert!(ByteOffset::new(u32::MAX).checked_add(1).is_none());
    }

    #[test]
    fn converts_to_usize() {
        assert_eq!(usize::from(ByteOffset::new(42)), 42);
    }
}
