//! Integration tests moved from src/line_index.rs for region coverage.

use reciplexa_source::line_index::*;
use reciplexa_source::offset::ByteOffset;

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
fn position_formats_as_line_column() {
    let idx = LineIndex::new("(page a4)");
    let pos = idx.position(ByteOffset::new(6));
    assert_eq!(pos.to_string(), "1:7");
}

#[test]
fn multiline_position() {
    let idx = LineIndex::new("(page a4\n  (circle 1 2 3))");
    let pos = idx.position(ByteOffset::new(12));
    assert_eq!(pos.line.get(), 2);
}

#[test]
fn line_start_returns_offsets_and_none_past_end() {
    let idx = LineIndex::new("a\nb\nc");
    assert_eq!(idx.line_start(LineNumber::ONE), Some(ByteOffset::ZERO));
    assert_eq!(idx.line_start(LineNumber::new(2)), Some(ByteOffset::new(2)));
    assert_eq!(idx.line_start(LineNumber::new(3)), Some(ByteOffset::new(4)));
    assert!(idx.line_start(LineNumber::new(99)).is_none());
    assert_eq!(idx.line_start(LineNumber::new(0)), Some(ByteOffset::ZERO));
}
