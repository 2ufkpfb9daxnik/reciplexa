//! Scribble `(doc …)` layout package (meaning, not reading).
//!
//! Reading lives in `reciplexa-syntax::doc`. This module turns [`DocPart`]s into
//! sized / spaced items ready to emit as `(text …)` / `(line …)` shapes.

use reciplexa_syntax::{flatten_lines, flatten_readable, DocPart};

/// One drawable text line after package layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LaidLine {
    pub size_mm: f64,
    /// Distance to subtract from this baseline to place the next item.
    pub y_gap_after: f64,
    /// Extra inset from [`DocFrame::left_mm`] (e.g. quotes).
    pub indent_mm: f64,
    pub content: String,
}

/// Flow item after Scribble package layout (text, gap, or rule).
#[derive(Debug, Clone, PartialEq)]
pub enum LaidItem {
    Text(LaidLine),
    /// Advance the cursor by `mm` without drawing (from `@vspace{…}`).
    VSpace {
        mm: f64,
    },
    /// Horizontal rule at the current baseline (`@hr`).
    Hr {
        y_gap_after: f64,
    },
}

impl LaidItem {
    fn y_gap_after(&self) -> f64 {
        match self {
            LaidItem::Text(t) => t.y_gap_after,
            LaidItem::VSpace { mm } => *mm,
            LaidItem::Hr { y_gap_after } => *y_gap_after,
        }
    }
}

pub const TITLE_SIZE_MM: f64 = 14.0;
pub const TITLE_GAP_MM: f64 = 18.0;
pub const H2_SIZE_MM: f64 = 11.0;
pub const H2_GAP_MM: f64 = 14.0;
pub const BODY_SIZE_MM: f64 = 8.0;
pub const BODY_GAP_MM: f64 = 12.0;
pub const QUOTE_SIZE_MM: f64 = 7.0;
pub const QUOTE_GAP_MM: f64 = 12.0;
pub const QUOTE_INDENT_MM: f64 = 10.0;
pub const QUOTE_WRAP_CHARS: usize = 36;
pub const HR_GAP_MM: f64 = 12.0;
pub const HR_WIDTH_MM: f64 = 0.4;

/// Soft wrap budget for body lines (~A4 content width at body size; not JLReq).
pub const BODY_WRAP_CHARS: usize = 40;
/// Soft wrap budget for title lines.
pub const TITLE_WRAP_CHARS: usize = 24;
/// Soft wrap budget for h2 lines.
pub const H2_WRAP_CHARS: usize = 32;

/// Page frame used when emitting `(page a4 …)` from laid items.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DocFrame {
    pub left_mm: f64,
    pub top_mm: f64,
    pub bottom_mm: f64,
    /// Paper width in mm (A4 = 210); right margin = width - left.
    pub width_mm: f64,
}

impl DocFrame {
    pub const A4: Self = Self {
        left_mm: 25.0,
        top_mm: 270.0,
        bottom_mm: 25.0,
        width_mm: 210.0,
    };

    pub fn right_mm(self) -> f64 {
        self.width_mm - self.left_mm
    }
}

/// One text shape placement after pagination (page-local coordinates).
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedText {
    pub page_index: usize,
    pub x_mm: f64,
    pub y_mm: f64,
    pub size_mm: f64,
    pub content: String,
}

/// Drawable placement after pagination.
#[derive(Debug, Clone, PartialEq)]
pub enum PlacedItem {
    Text(PlacedText),
    Line {
        page_index: usize,
        x1_mm: f64,
        y1_mm: f64,
        x2_mm: f64,
        y2_mm: f64,
        width_mm: f64,
    },
}

impl PlacedItem {
    pub fn page_index(&self) -> usize {
        match self {
            PlacedItem::Text(t) => t.page_index,
            PlacedItem::Line { page_index, .. } => *page_index,
        }
    }
}

/// Assign laid items to pages, starting a new page when the baseline would
/// fall below [`DocFrame::bottom_mm`].
pub fn place_items(items: &[LaidItem], frame: DocFrame) -> Vec<PlacedItem> {
    let mut out = Vec::new();
    if items.is_empty() {
        return out;
    }
    let mut page = 0usize;
    let mut y = frame.top_mm;
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            let next_y = y - items[i - 1].y_gap_after();
            if next_y < frame.bottom_mm {
                page += 1;
                y = frame.top_mm;
            } else {
                y = next_y;
            }
        }
        match item {
            LaidItem::Text(line) => {
                out.push(PlacedItem::Text(PlacedText {
                    page_index: page,
                    x_mm: frame.left_mm + line.indent_mm,
                    y_mm: y,
                    size_mm: line.size_mm,
                    content: line.content.clone(),
                }));
            }
            LaidItem::VSpace { .. } => {
                // Cursor advances via y_gap_after only; nothing drawn.
            }
            LaidItem::Hr { .. } => {
                out.push(PlacedItem::Line {
                    page_index: page,
                    x1_mm: frame.left_mm,
                    y1_mm: y,
                    x2_mm: frame.right_mm(),
                    y2_mm: y,
                    width_mm: HR_WIDTH_MM,
                });
            }
        }
    }
    out
}

/// Turn Scribble parts into laid items (`@title` / `@p` / `@vspace` / `@hr` …).
pub fn layout_doc_parts(parts: &[DocPart]) -> Vec<LaidItem> {
    let mut out = Vec::new();
    let mut buf = String::new();

    for part in parts {
        match part {
            DocPart::Text(t) => buf.push_str(t),
            DocPart::Newline => flush_body(&mut buf, &mut out),
            DocPart::At {
                name,
                bracket_args,
                brace_body,
            } => match name.as_str() {
                "title" => {
                    flush_body(&mut buf, &mut out);
                    push_styled_block(
                        brace_body,
                        TITLE_SIZE_MM,
                        TITLE_GAP_MM,
                        TITLE_WRAP_CHARS,
                        &mut out,
                    );
                }
                "h2" => {
                    flush_body(&mut buf, &mut out);
                    push_styled_block(brace_body, H2_SIZE_MM, H2_GAP_MM, H2_WRAP_CHARS, &mut out);
                }
                "li" => {
                    flush_body(&mut buf, &mut out);
                    push_list_items(brace_body, &mut out);
                }
                "quote" => {
                    flush_body(&mut buf, &mut out);
                    push_styled_block_indent(
                        brace_body,
                        QUOTE_SIZE_MM,
                        QUOTE_GAP_MM,
                        QUOTE_WRAP_CHARS,
                        QUOTE_INDENT_MM,
                        &mut out,
                    );
                }
                "ol" => {
                    flush_body(&mut buf, &mut out);
                    push_ordered_list(brace_body, &mut out);
                }
                "p" => {
                    flush_body(&mut buf, &mut out);
                    push_styled_block(
                        brace_body,
                        BODY_SIZE_MM,
                        BODY_GAP_MM,
                        BODY_WRAP_CHARS,
                        &mut out,
                    );
                }
                "vspace" => {
                    flush_body(&mut buf, &mut out);
                    push_vspace(brace_body, &mut out);
                }
                "hr" => {
                    flush_body(&mut buf, &mut out);
                    out.push(LaidItem::Hr {
                        y_gap_after: HR_GAP_MM,
                    });
                }
                _ => {
                    if !brace_body.is_empty() {
                        buf.push_str(&flatten_readable(brace_body));
                    } else if let Some(args) = bracket_args {
                        buf.push_str(args.trim());
                    }
                }
            },
        }
    }
    flush_body(&mut buf, &mut out);
    out
}

fn flush_body(buf: &mut String, out: &mut Vec<LaidItem>) {
    let text = collapse_ws(buf);
    buf.clear();
    if text.is_empty() {
        return;
    }
    push_wrapped(&text, BODY_SIZE_MM, BODY_GAP_MM, BODY_WRAP_CHARS, 0.0, out);
}

fn push_styled_block(
    body: &[DocPart],
    size_mm: f64,
    y_gap_after: f64,
    wrap_chars: usize,
    out: &mut Vec<LaidItem>,
) {
    push_styled_block_indent(body, size_mm, y_gap_after, wrap_chars, 0.0, out);
}

fn push_styled_block_indent(
    body: &[DocPart],
    size_mm: f64,
    y_gap_after: f64,
    wrap_chars: usize,
    indent_mm: f64,
    out: &mut Vec<LaidItem>,
) {
    for line in flatten_lines(body) {
        if line.is_empty() {
            continue;
        }
        push_wrapped(&line, size_mm, y_gap_after, wrap_chars, indent_mm, out);
    }
}

/// `@li{…}` → body-sized lines prefixed with a bullet (package meaning, not font glyphs).
fn push_list_items(body: &[DocPart], out: &mut Vec<LaidItem>) {
    for line in flatten_lines(body) {
        if line.is_empty() {
            continue;
        }
        let bulleted = format!("• {line}");
        push_wrapped(
            &bulleted,
            BODY_SIZE_MM,
            BODY_GAP_MM,
            BODY_WRAP_CHARS,
            0.0,
            out,
        );
    }
}

/// `@ol{a; b; c}` → numbered body lines. Semicolons separate items (brace text stays simple).
fn push_ordered_list(body: &[DocPart], out: &mut Vec<LaidItem>) {
    let flat = flatten_readable(body);
    let mut n = 0usize;
    for segment in flat.split(';') {
        let item = segment.trim();
        if item.is_empty() {
            continue;
        }
        n += 1;
        let numbered = format!("{n}. {item}");
        push_wrapped(
            &numbered,
            BODY_SIZE_MM,
            BODY_GAP_MM,
            BODY_WRAP_CHARS,
            0.0,
            out,
        );
    }
}

/// `@vspace{N}` — N must parse as a finite positive number (mm); otherwise skip.
fn push_vspace(body: &[DocPart], out: &mut Vec<LaidItem>) {
    let raw = flatten_readable(body);
    let Ok(mm) = raw.parse::<f64>() else {
        return;
    };
    if !(mm.is_finite() && mm > 0.0) {
        return;
    }
    out.push(LaidItem::VSpace { mm });
}

fn push_wrapped(
    text: &str,
    size_mm: f64,
    y_gap_after: f64,
    wrap_chars: usize,
    indent_mm: f64,
    out: &mut Vec<LaidItem>,
) {
    let chunks = wrap_line(text, wrap_chars);
    let n = chunks.len();
    for (i, chunk) in chunks.into_iter().enumerate() {
        let gap = if i + 1 == n {
            y_gap_after
        } else {
            // Continuations use body leading until the block gap on the last fragment.
            BODY_GAP_MM.min(y_gap_after)
        };
        out.push(LaidItem::Text(LaidLine {
            size_mm,
            y_gap_after: gap,
            indent_mm,
            content: chunk,
        }));
    }
}

/// Soft-wrap `text` to at most `max_chars` Unicode scalars per line.
///
/// Breaks at the previous ASCII space only when the limit would split a word;
/// otherwise hard-breaks. `max_chars == 0` means no wrapping.
pub fn wrap_line(text: &str, max_chars: usize) -> Vec<String> {
    if max_chars == 0 {
        return if text.is_empty() {
            Vec::new()
        } else {
            vec![text.to_string()]
        };
    }
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        if chars.len() - start <= max_chars {
            let slice: String = chars[start..].iter().collect();
            let trimmed = slice.trim();
            if !trimmed.is_empty() {
                out.push(trimmed.to_string());
            }
            break;
        }
        let mut end = start + max_chars;
        let mid_word = chars[end - 1] != ' ' && chars[end] != ' ';
        if mid_word {
            if let Some(rel) = chars[start..end].iter().rposition(|c| *c == ' ') {
                if rel > 0 {
                    end = start + rel;
                }
            }
        }
        let slice: String = chars[start..end].iter().collect();
        let trimmed = slice.trim();
        if !trimmed.is_empty() {
            out.push(trimmed.to_string());
        }
        start = end;
        while start < chars.len() && chars[start] == ' ' {
            start += 1;
        }
    }
    out
}

fn collapse_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_space && !out.is_empty() {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_syntax::{doc_parts, parse_source, SyntaxKind};

    fn parts(src: &str) -> Vec<DocPart> {
        let root = parse_source(src).into_result().unwrap();
        let list = root
            .children()
            .find(|n| n.kind() == SyntaxKind::List)
            .unwrap();
        doc_parts(&list).unwrap()
    }

    fn text_items(items: &[LaidItem]) -> Vec<&LaidLine> {
        items
            .iter()
            .filter_map(|i| match i {
                LaidItem::Text(t) => Some(t),
                _ => None,
            })
            .collect()
    }

    // --- validity: wrap ---

    #[test]
    fn wrap_respects_space_break() {
        let lines = wrap_line("hello world again", 11);
        assert_eq!(lines, vec!["hello world", "again"]);
    }

    #[test]
    fn wrap_hard_breaks_cjk_when_no_space() {
        let s: String = "あ".repeat(5);
        let lines = wrap_line(&s, 2);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].chars().count(), 2);
        assert_eq!(lines[2].chars().count(), 1);
    }

    #[test]
    fn layout_wraps_long_paragraph() {
        let long = "a".repeat(45);
        let src = format!("(doc @p{{{long}}})");
        let laid = layout_doc_parts(&parts(&src));
        let texts = text_items(&laid);
        assert!(texts.len() >= 2, "expected wrap into ≥2 lines: {laid:?}");
        assert!(texts.iter().all(|l| l.size_mm == BODY_SIZE_MM));
        assert!(texts
            .iter()
            .all(|l| l.content.chars().count() <= BODY_WRAP_CHARS));
    }

    // --- defect: wrap ---

    #[test]
    fn wrap_max_zero_is_noop() {
        assert_eq!(wrap_line("abc def", 0), vec!["abc def".to_string()]);
    }

    #[test]
    fn wrap_empty_is_empty() {
        assert!(wrap_line("", 10).is_empty());
    }

    #[test]
    fn layout_skips_empty_title() {
        assert!(layout_doc_parts(&parts("(doc @title{})")).is_empty());
    }

    #[test]
    fn place_items_starts_new_page_at_bottom_margin() {
        let items: Vec<LaidItem> = (0..30)
            .map(|i| {
                LaidItem::Text(LaidLine {
                    size_mm: BODY_SIZE_MM,
                    y_gap_after: BODY_GAP_MM,
                    indent_mm: 0.0,
                    content: format!("L{i}"),
                })
            })
            .collect();
        let placed = place_items(&items, DocFrame::A4);
        let max_page = placed.iter().map(PlacedItem::page_index).max().unwrap();
        assert!(max_page >= 1, "expected a second page, got {placed:?}");
        assert!(placed.iter().all(|p| match p {
            PlacedItem::Text(t) => t.y_mm >= DocFrame::A4.bottom_mm,
            PlacedItem::Line { y1_mm, .. } => *y1_mm >= DocFrame::A4.bottom_mm,
        }));
    }

    #[test]
    fn place_items_emits_hr_line() {
        let items = vec![
            LaidItem::Text(LaidLine {
                size_mm: BODY_SIZE_MM,
                y_gap_after: BODY_GAP_MM,
                indent_mm: 0.0,
                content: "Head".into(),
            }),
            LaidItem::Hr {
                y_gap_after: HR_GAP_MM,
            },
        ];
        let placed = place_items(&items, DocFrame::A4);
        assert_eq!(placed.len(), 2);
        match &placed[1] {
            PlacedItem::Line {
                x1_mm,
                x2_mm,
                y1_mm,
                width_mm,
                ..
            } => {
                assert_eq!(*x1_mm, 25.0);
                assert_eq!(*x2_mm, 185.0);
                assert_eq!(*y1_mm, 258.0);
                assert_eq!(*width_mm, HR_WIDTH_MM);
            }
            other => panic!("expected line, got {other:?}"),
        }
    }
}
