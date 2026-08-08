//! Line and column positions derived from UTF-8 byte offsets.

use crate::offset::ByteOffset;

/// One-based line number for human-facing diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LineNumber(pub u32);

impl LineNumber {
  pub const ONE: Self = Self(1);

  pub const fn new(value: u32) -> Self {
    Self(value)
  }

  pub const fn get(self) -> u32 {
    self.0
  }
}

/// One-based UTF-8 byte column within a line (not display width).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ColumnNumber(pub u32);

impl ColumnNumber {
  pub const ONE: Self = Self(1);

  pub const fn new(value: u32) -> Self {
    Self(value)
  }

  pub const fn get(self) -> u32 {
    self.0
  }
}

/// A line/column position in source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourcePosition {
  pub line: LineNumber,
  pub column: ColumnNumber,
}

/// Index mapping byte offsets to line starts.
#[derive(Debug, Clone, Default)]
pub struct LineIndex {
  line_starts: Vec<ByteOffset>,
}

impl LineIndex {
  pub fn new(source: &str) -> Self {
    let mut line_starts = vec![ByteOffset::ZERO];
    for (idx, ch) in source.char_indices() {
      if ch == '\n' {
        line_starts.push(ByteOffset::new((idx + ch.len_utf8()) as u32));
      }
    }
    Self { line_starts }
  }

  pub fn line_count(&self) -> usize {
    self.line_starts.len()
  }

  pub fn line_start(&self, line: LineNumber) -> Option<ByteOffset> {
    let idx = line.get().saturating_sub(1) as usize;
    self.line_starts.get(idx).copied()
  }

  pub fn position(&self, offset: ByteOffset) -> SourcePosition {
    let byte = offset.get() as usize;
    let line_idx = match self.line_starts.binary_search(&offset) {
      Ok(i) => i,
      Err(i) => i.saturating_sub(1),
    };
    let line_start = self.line_starts[line_idx].get() as usize;
    let column = byte.saturating_sub(line_start) + 1;
    SourcePosition {
      line: LineNumber::new((line_idx + 1) as u32),
      column: ColumnNumber::new(column as u32),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn single_line_position() {
    let idx = LineIndex::new("(page a4)");
    let pos = idx.position(ByteOffset::new(6));
    assert_eq!(pos.line.get(), 1);
    assert_eq!(pos.column.get(), 7);
  }

  #[test]
  fn crlf_counts_as_one_line_break() {
    let idx = LineIndex::new("a\r\nb");
    assert_eq!(idx.line_count(), 2);
    let pos = idx.position(ByteOffset::new(3));
    assert_eq!(pos.line.get(), 2);
    assert_eq!(pos.column.get(), 1);
  }

  #[test]
  fn multiline_position() {
    let idx = LineIndex::new("(page a4\n  (circle 1 2 3))");
    let pos = idx.position(ByteOffset::new(12));
    assert_eq!(pos.line.get(), 2);
  }
}
