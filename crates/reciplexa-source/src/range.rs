//! Half-open UTF-8 byte ranges.

use core::fmt;

use crate::offset::ByteOffset;

/// A half-open UTF-8 byte range `[start, end)` into a source buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TextRange {
    start: ByteOffset,
    end: ByteOffset,
}

/// Error returned when constructing an invalid [`TextRange`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRangeError {
    EndBeforeStart { start: ByteOffset, end: ByteOffset },
}

impl TextRange {
    pub const EMPTY: Self = Self {
        start: ByteOffset::ZERO,
        end: ByteOffset::ZERO,
    };

    pub const fn new(start: ByteOffset, end: ByteOffset) -> Self {
        Self { start, end }
    }

    pub fn try_new(start: ByteOffset, end: ByteOffset) -> Result<Self, TextRangeError> {
        if end < start {
            Err(TextRangeError::EndBeforeStart { start, end })
        } else {
            Ok(Self { start, end })
        }
    }

    pub const fn at(offset: ByteOffset) -> Self {
        Self {
            start: offset,
            end: offset,
        }
    }

    pub const fn start(self) -> ByteOffset {
        self.start
    }

    pub const fn end(self) -> ByteOffset {
        self.end
    }

    pub fn len(self) -> u32 {
        self.end.0.saturating_sub(self.start.0)
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    pub fn contains_offset(self, offset: ByteOffset) -> bool {
        offset >= self.start && offset < self.end
    }

    pub fn contains_range(self, other: Self) -> bool {
        other.start >= self.start && other.end <= self.end
    }

    pub fn union(self, other: Self) -> Result<Self, TextRangeError> {
        let start = self.start.min(other.start);
        let end = self.end.max(other.end);
        Self::try_new(start, end)
    }

    pub fn intersect(self, other: Self) -> Option<Self> {
        let start = self.start.max(other.start);
        let end = self.end.min(other.end);
        if end < start {
            None
        } else {
            Some(Self { start, end })
        }
    }

    pub fn cover(self, offset: ByteOffset) -> Result<Self, TextRangeError> {
        let start = self.start.min(offset);
        let end = self.end.max(offset);
        Self::try_new(start, end)
    }
}

impl fmt::Display for TextRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {})", self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn off(n: u32) -> ByteOffset {
        ByteOffset::new(n)
    }

    #[test]
    fn empty_range_has_zero_len() {
        assert!(TextRange::EMPTY.is_empty());
        assert_eq!(TextRange::EMPTY.len(), 0);
    }

    #[test]
    fn const_new_builds_unchecked_range() {
        let r = TextRange::new(off(2), off(5));
        assert_eq!(r.start(), off(2));
        assert_eq!(r.end(), off(5));
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn rejects_end_before_start() {
        assert!(matches!(
            TextRange::try_new(off(5), off(3)),
            Err(TextRangeError::EndBeforeStart { .. })
        ));
    }

    #[test]
    fn contains_offset_is_half_open() {
        let r = TextRange::try_new(off(2), off(5)).unwrap();
        assert!(!r.contains_offset(off(1)));
        assert!(r.contains_offset(off(2)));
        assert!(r.contains_offset(off(4)));
        assert!(!r.contains_offset(off(5)));
    }

    #[test]
    fn union_and_intersect() {
        let a = TextRange::try_new(off(0), off(3)).unwrap();
        let b = TextRange::try_new(off(2), off(6)).unwrap();
        assert_eq!(
            a.union(b).unwrap(),
            TextRange::try_new(off(0), off(6)).unwrap()
        );
        assert_eq!(
            a.intersect(b).unwrap(),
            TextRange::try_new(off(2), off(3)).unwrap()
        );
    }

    #[test]
    fn disjoint_intersect_is_none() {
        let a = TextRange::try_new(off(0), off(2)).unwrap();
        let b = TextRange::try_new(off(3), off(5)).unwrap();
        assert!(a.intersect(b).is_none());
    }

    #[test]
    fn contains_range_and_cover() {
        let outer = TextRange::try_new(off(0), off(10)).unwrap();
        let inner = TextRange::try_new(off(2), off(5)).unwrap();
        assert!(outer.contains_range(inner));
        assert!(!inner.contains_range(outer));
        let covered = outer.cover(off(12)).unwrap();
        assert_eq!(covered, TextRange::try_new(off(0), off(12)).unwrap());
    }

    #[test]
    fn at_offset_is_zero_width() {
        let r = TextRange::at(off(7));
        assert!(r.is_empty());
        assert!(!r.contains_offset(off(7)));
        assert!(!r.contains_offset(off(8)));
    }

    #[test]
    fn display_formats_half_open_range() {
        let r = TextRange::try_new(off(1), off(4)).unwrap();
        assert_eq!(r.to_string(), "[1, 4)");
    }
}
