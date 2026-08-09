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
        f.write_str(&format!("[{}, {})", self.start, self.end))
    }
}
