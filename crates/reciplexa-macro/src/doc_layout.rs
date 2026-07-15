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

/// Flow item after Scribble package layout (text, gap, rule, image, or forced break).
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
    /// Embedded image (`@image["path"]`); `y` in placement is bottom-left.
    Image {
        path: String,
        width_mm: f64,
        height_mm: f64,
        /// Advance after the image top reference (typically height + pad).
        y_gap_after: f64,
    },
    /// Force the next drawable item onto a new page (`@pagebreak`).
    PageBreak,
}

impl LaidItem {
    fn is_pagebreak(&self) -> bool {
        matches!(self, LaidItem::PageBreak)
    }
}

pub const TITLE_SIZE_MM: f64 = 14.0;
pub const TITLE_GAP_MM: f64 = 18.0;
pub const H2_SIZE_MM: f64 = 11.0;
pub const H2_GAP_MM: f64 = 14.0;
pub const H3_SIZE_MM: f64 = 9.0;
pub const H3_GAP_MM: f64 = 12.0;
pub const BODY_SIZE_MM: f64 = 8.0;
pub const BODY_GAP_MM: f64 = 12.0;
pub const QUOTE_SIZE_MM: f64 = 7.0;
pub const QUOTE_GAP_MM: f64 = 12.0;
pub const QUOTE_INDENT_MM: f64 = 10.0;
pub const QUOTE_WRAP_CHARS: usize = 36;
pub const CODE_SIZE_MM: f64 = 6.5;
pub const CODE_GAP_MM: f64 = 10.0;
pub const CODE_INDENT_MM: f64 = 8.0;
pub const CODE_WRAP_CHARS: usize = 48;
pub const CAPTION_SIZE_MM: f64 = 6.0;
pub const CAPTION_GAP_MM: f64 = 10.0;
pub const CAPTION_INDENT_MM: f64 = 8.0;
pub const CAPTION_WRAP_CHARS: usize = 42;
/// Fixed blank gap from `@br{}` (same leading as body).
pub const BR_GAP_MM: f64 = 12.0;
pub const HR_GAP_MM: f64 = 12.0;
pub const HR_WIDTH_MM: f64 = 0.4;
/// Default `@image` width (mm).
pub const FIGURE_WIDTH_MM: f64 = 80.0;
/// Default `@image` height (mm).
pub const FIGURE_HEIGHT_MM: f64 = 50.0;
/// Extra gap under a figure before the next flow item.
pub const FIGURE_PAD_MM: f64 = 8.0;

/// Soft wrap budget for body lines (~A4 content width at body size; not JLReq).
pub const BODY_WRAP_CHARS: usize = 40;
/// Soft wrap budget for title lines.
pub const TITLE_WRAP_CHARS: usize = 24;
/// Soft wrap budget for h2 lines.
pub const H2_WRAP_CHARS: usize = 32;
/// Soft wrap budget for h3 lines.
pub const H3_WRAP_CHARS: usize = 36;

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
    Image {
        page_index: usize,
        path: String,
        x_mm: f64,
        /// Bottom-left y (same convention as `(image …)`).
        y_mm: f64,
        width_mm: f64,
        height_mm: f64,
    },
}

impl PlacedItem {
    pub fn page_index(&self) -> usize {
        match self {
            PlacedItem::Text(t) => t.page_index,
            PlacedItem::Line { page_index, .. } => *page_index,
            PlacedItem::Image { page_index, .. } => *page_index,
        }
    }
}

/// Assign laid items to pages, starting a new page when the baseline would
/// fall below [`DocFrame::bottom_mm`], or when a [`LaidItem::PageBreak`] appears.
pub fn place_items(items: &[LaidItem], frame: DocFrame) -> Vec<PlacedItem> {
    let mut out = Vec::new();
    if items.is_empty() {
        return out;
    }
    let mut page = 0usize;
    let mut y = frame.top_mm;
    let mut force_new_page = false;
    let mut prev_gap = 0.0_f64;
    let mut have_prev = false;
    for item in items {
        if item.is_pagebreak() {
            force_new_page = true;
            have_prev = false;
            continue;
        }
        let item_height = match item {
            LaidItem::Image { height_mm, .. } => *height_mm,
            _ => 0.0,
        };
        if force_new_page {
            page += 1;
            y = frame.top_mm;
            force_new_page = false;
        } else if have_prev {
            let next_y = y - prev_gap;
            if next_y - item_height < frame.bottom_mm {
                page += 1;
                y = frame.top_mm;
            } else {
                y = next_y;
            }
        } else if y - item_height < frame.bottom_mm && item_height > 0.0 {
            // First item on a page that still cannot fit: place at top anyway.
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
                prev_gap = line.y_gap_after;
                have_prev = true;
            }
            LaidItem::VSpace { mm } => {
                prev_gap = *mm;
                have_prev = true;
            }
            LaidItem::Hr { y_gap_after } => {
                out.push(PlacedItem::Line {
                    page_index: page,
                    x1_mm: frame.left_mm,
                    y1_mm: y,
                    x2_mm: frame.right_mm(),
                    y2_mm: y,
                    width_mm: HR_WIDTH_MM,
                });
                prev_gap = *y_gap_after;
                have_prev = true;
            }
            LaidItem::Image {
                path,
                width_mm,
                height_mm,
                y_gap_after,
            } => {
                out.push(PlacedItem::Image {
                    page_index: page,
                    path: path.clone(),
                    x_mm: frame.left_mm,
                    y_mm: y - height_mm,
                    width_mm: *width_mm,
                    height_mm: *height_mm,
                });
                prev_gap = *y_gap_after;
                have_prev = true;
            }
            LaidItem::PageBreak => unreachable!("handled above"),
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
                "title" | "h1" => {
                    flush_body(&mut buf, &mut out);
                    push_styled_block(
                        brace_body,
                        TITLE_SIZE_MM,
                        TITLE_GAP_MM,
                        TITLE_WRAP_CHARS,
                        &mut out,
                    );
                }
                "h2" | "section" => {
                    flush_body(&mut buf, &mut out);
                    push_styled_block(brace_body, H2_SIZE_MM, H2_GAP_MM, H2_WRAP_CHARS, &mut out);
                }
                "h3" | "subsubsection" => {
                    flush_body(&mut buf, &mut out);
                    push_styled_block(brace_body, H3_SIZE_MM, H3_GAP_MM, H3_WRAP_CHARS, &mut out);
                }
                "li" | "item" => {
                    flush_body(&mut buf, &mut out);
                    push_list_items(brace_body, &mut out);
                }
                "quote" | "blockquote" => {
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
                "note" => {
                    flush_body(&mut buf, &mut out);
                    push_note_block(brace_body, &mut out);
                }
                "warn" => {
                    flush_body(&mut buf, &mut out);
                    push_warn_block(brace_body, &mut out);
                }
                "todo" => {
                    flush_body(&mut buf, &mut out);
                    push_todo_block(brace_body, &mut out);
                }
                "em" | "italic" => {
                    push_marked_inline(brace_body, "*", &mut buf);
                }
                "strong" | "bold" => {
                    push_marked_inline(brace_body, "**", &mut buf);
                }
                "tt" | "code_inline" => {
                    push_marked_inline(brace_body, "`", &mut buf);
                }
                "center" => {
                    flush_body(&mut buf, &mut out);
                    push_centered_block(brace_body, &mut out);
                }
                "code" | "pre" => {
                    flush_body(&mut buf, &mut out);
                    push_code_block(brace_body, &mut out);
                }
                "ol" => {
                    flush_body(&mut buf, &mut out);
                    push_ordered_list(brace_body, &mut out);
                }
                "ul" => {
                    flush_body(&mut buf, &mut out);
                    push_unordered_list(brace_body, &mut out);
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
                "pagebreak" => {
                    flush_body(&mut buf, &mut out);
                    out.push(LaidItem::PageBreak);
                }
                "caption" => {
                    flush_body(&mut buf, &mut out);
                    push_styled_block_indent(
                        brace_body,
                        CAPTION_SIZE_MM,
                        CAPTION_GAP_MM,
                        CAPTION_WRAP_CHARS,
                        CAPTION_INDENT_MM,
                        &mut out,
                    );
                }
                "br" => {
                    flush_body(&mut buf, &mut out);
                    out.push(LaidItem::VSpace { mm: BR_GAP_MM });
                }
                "link" => {
                    push_link_inline(brace_body, bracket_args.as_deref(), &mut buf);
                }
                "cite" => {
                    push_cite_inline(bracket_args.as_deref(), &mut buf);
                }
                "image" | "figure" => {
                    flush_body(&mut buf, &mut out);
                    push_image(bracket_args.as_deref(), brace_body, &mut out);
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

/// `@link[url]{label}` → `label (url)` (no PDF hyperlink yet; package text only).
fn push_link_inline(body: &[DocPart], bracket_args: Option<&str>, buf: &mut String) {
    let label = flatten_marked(body);
    let url = bracket_args
        .map(strip_bracket_string)
        .filter(|s| !s.is_empty());
    match (label.is_empty(), url) {
        (false, Some(u)) => {
            buf.push_str(&label);
            buf.push_str(" (");
            buf.push_str(&u);
            buf.push(')');
        }
        (false, None) => buf.push_str(&label),
        (true, Some(u)) => buf.push_str(&u),
        (true, None) => {}
    }
}

/// `@cite[key]` → `[key]`.
fn push_cite_inline(bracket_args: Option<&str>, buf: &mut String) {
    let key = bracket_args.map(strip_bracket_string).unwrap_or_default();
    if key.is_empty() {
        return;
    }
    buf.push('[');
    buf.push_str(&key);
    buf.push(']');
}

fn strip_bracket_string(s: &str) -> String {
    let t = s.trim();
    if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
        t[1..t.len() - 1].to_string()
    } else {
        t.to_string()
    }
}

/// `@image["path"]{optional caption}` — default figure size, caption as `@caption`.
/// Size override: `@image["path" width-mm height-mm]`.
fn push_image(bracket_args: Option<&str>, body: &[DocPart], out: &mut Vec<LaidItem>) {
    let Some((path, width_mm, height_mm)) = bracket_args.and_then(parse_image_bracket) else {
        return;
    };
    out.push(LaidItem::Image {
        path,
        width_mm,
        height_mm,
        y_gap_after: height_mm + FIGURE_PAD_MM,
    });
    push_styled_block_indent(
        body,
        CAPTION_SIZE_MM,
        CAPTION_GAP_MM,
        CAPTION_WRAP_CHARS,
        CAPTION_INDENT_MM,
        out,
    );
}

fn parse_image_bracket(s: &str) -> Option<(String, f64, f64)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (path, rest) = if let Some(after_quote) = s.strip_prefix('"') {
        let end = after_quote.find('"')?;
        let path = after_quote[..end].to_string();
        let rest = after_quote[end + 1..].trim().to_string();
        (path, rest)
    } else {
        let mut parts = s.split_whitespace();
        let path = parts.next()?.to_string();
        let rest = parts.collect::<Vec<_>>().join(" ");
        (path, rest)
    };
    if path.is_empty() {
        return None;
    }
    let nums: Vec<f64> = rest
        .split_whitespace()
        .filter_map(|t| t.parse().ok())
        .collect();
    let (width_mm, height_mm) = match nums.as_slice() {
        [w, h, ..] if *w > 0.0 && *h > 0.0 && w.is_finite() && h.is_finite() => (*w, *h),
        _ => (FIGURE_WIDTH_MM, FIGURE_HEIGHT_MM),
    };
    Some((path, width_mm, height_mm))
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

/// `@note{…}` — quote-sized indented callout with a fixed `Note: ` prefix.
fn push_note_block(body: &[DocPart], out: &mut Vec<LaidItem>) {
    let text = flatten_readable(body);
    if text.is_empty() {
        return;
    }
    let prefixed = format!("Note: {text}");
    push_wrapped(
        &prefixed,
        QUOTE_SIZE_MM,
        QUOTE_GAP_MM,
        QUOTE_WRAP_CHARS,
        QUOTE_INDENT_MM,
        out,
    );
}

/// `@warn{…}` — same layout as note, with a `Warning: ` prefix.
fn push_warn_block(body: &[DocPart], out: &mut Vec<LaidItem>) {
    let text = flatten_readable(body);
    if text.is_empty() {
        return;
    }
    let prefixed = format!("Warning: {text}");
    push_wrapped(
        &prefixed,
        QUOTE_SIZE_MM,
        QUOTE_GAP_MM,
        QUOTE_WRAP_CHARS,
        QUOTE_INDENT_MM,
        out,
    );
}

/// `@todo{…}` — callout with a `TODO: ` prefix.
fn push_todo_block(body: &[DocPart], out: &mut Vec<LaidItem>) {
    let text = flatten_readable(body);
    if text.is_empty() {
        return;
    }
    let prefixed = format!("TODO: {text}");
    push_wrapped(
        &prefixed,
        QUOTE_SIZE_MM,
        QUOTE_GAP_MM,
        QUOTE_WRAP_CHARS,
        QUOTE_INDENT_MM,
        out,
    );
}

/// `@em` / `@strong` / `@tt` — surround flat text with markers (no font variants yet).
fn push_marked_inline(body: &[DocPart], marker: &str, buf: &mut String) {
    let text = flatten_marked(body);
    if text.is_empty() {
        return;
    }
    buf.push_str(marker);
    buf.push_str(&text);
    buf.push_str(marker);
}

/// Like [`flatten_readable`], but runs package inline marks (`@em`, `@link`, …).
fn flatten_marked(parts: &[DocPart]) -> String {
    let mut buf = String::new();
    append_marked(parts, &mut buf);
    collapse_ws(&buf)
}

fn append_marked(parts: &[DocPart], buf: &mut String) {
    for part in parts {
        match part {
            DocPart::Text(t) => buf.push_str(t),
            DocPart::Newline => buf.push(' '),
            DocPart::At {
                name,
                bracket_args,
                brace_body,
            } => match name.as_str() {
                "em" | "italic" => push_marked_inline(brace_body, "*", buf),
                "strong" | "bold" => push_marked_inline(brace_body, "**", buf),
                "tt" | "code_inline" => push_marked_inline(brace_body, "`", buf),
                "link" => push_link_inline(brace_body, bracket_args.as_deref(), buf),
                "cite" => push_cite_inline(bracket_args.as_deref(), buf),
                _ => {
                    if !brace_body.is_empty() {
                        append_marked(brace_body, buf);
                    } else if let Some(args) = bracket_args {
                        buf.push_str(args.trim());
                    }
                }
            },
        }
    }
}

/// Line-splitting variant of [`flatten_marked`] for `@li`.
fn flatten_lines_marked(parts: &[DocPart]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    append_marked_lines(parts, &mut lines, &mut cur);
    if !cur.is_empty() || lines.is_empty() {
        lines.push(std::mem::take(&mut cur));
    }
    lines
        .into_iter()
        .map(|l| collapse_ws(&l))
        .filter(|l| !l.is_empty())
        .collect()
}

fn append_marked_lines(parts: &[DocPart], lines: &mut Vec<String>, cur: &mut String) {
    for part in parts {
        match part {
            DocPart::Text(t) => cur.push_str(t),
            DocPart::Newline => {
                lines.push(std::mem::take(cur));
            }
            DocPart::At {
                name,
                bracket_args,
                brace_body,
            } => match name.as_str() {
                "em" | "italic" => push_marked_inline(brace_body, "*", cur),
                "strong" | "bold" => push_marked_inline(brace_body, "**", cur),
                "tt" | "code_inline" => push_marked_inline(brace_body, "`", cur),
                "link" => push_link_inline(brace_body, bracket_args.as_deref(), cur),
                "cite" => push_cite_inline(bracket_args.as_deref(), cur),
                _ => {
                    if !brace_body.is_empty() {
                        append_marked_lines(brace_body, lines, cur);
                    } else if let Some(args) = bracket_args {
                        cur.push_str(args.trim());
                    }
                }
            },
        }
    }
}

/// `@center{…}` — approximate horizontal centering via indent (char-width heuristic).
fn push_centered_block(body: &[DocPart], out: &mut Vec<LaidItem>) {
    for line in flatten_lines_marked(body) {
        if line.is_empty() {
            continue;
        }
        let n = line.chars().count().min(BODY_WRAP_CHARS);
        let content_w = DocFrame::A4.right_mm() - DocFrame::A4.left_mm;
        let char_w = content_w / BODY_WRAP_CHARS as f64;
        let text_w = n as f64 * char_w;
        let indent = ((content_w - text_w) / 2.0).max(0.0);
        push_wrapped(
            &line,
            BODY_SIZE_MM,
            BODY_GAP_MM,
            BODY_WRAP_CHARS,
            indent,
            out,
        );
    }
}

/// `@li{…}` → body-sized lines prefixed with a bullet (package meaning, not font glyphs).
fn push_list_items(body: &[DocPart], out: &mut Vec<LaidItem>) {
    for line in flatten_lines_marked(body) {
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
    let flat = flatten_marked(body);
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

/// `@ul{a; b; c}` → bulleted body lines (same semicolon split as `@ol`).
fn push_unordered_list(body: &[DocPart], out: &mut Vec<LaidItem>) {
    let flat = flatten_marked(body);
    for segment in flat.split(';') {
        let item = segment.trim();
        if item.is_empty() {
            continue;
        }
        let bulleted = format!("• {item}");
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

/// `@code` / `@pre` — preserve internal spaces; soft-wrap still applies.
fn push_code_block(body: &[DocPart], out: &mut Vec<LaidItem>) {
    for line in flatten_code_lines(body) {
        if line.is_empty() {
            continue;
        }
        push_wrapped(
            &line,
            CODE_SIZE_MM,
            CODE_GAP_MM,
            CODE_WRAP_CHARS,
            CODE_INDENT_MM,
            out,
        );
    }
}

/// Like [`flatten_lines`], but keeps runs of spaces (only trim ends of each line).
fn flatten_code_lines(parts: &[DocPart]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    flush_code_parts(parts, &mut lines, &mut cur);
    if !cur.is_empty() || lines.is_empty() {
        lines.push(std::mem::take(&mut cur));
    }
    lines
        .into_iter()
        .map(|l| l.trim_end().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

fn flush_code_parts(parts: &[DocPart], lines: &mut Vec<String>, cur: &mut String) {
    for part in parts {
        match part {
            DocPart::Text(t) => cur.push_str(t),
            DocPart::Newline => {
                lines.push(std::mem::take(cur));
            }
            DocPart::At {
                bracket_args,
                brace_body,
                ..
            } => {
                if !brace_body.is_empty() {
                    flush_code_parts(brace_body, lines, cur);
                } else if let Some(args) = bracket_args {
                    cur.push_str(args);
                }
            }
        }
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
/// otherwise hard-breaks. Applies a tiny JLReq-inspired kinsoku:
/// - **prefer:** break after clause / list punctuation (`。` `、` `・` `：` …)
///   when they fall in the wrap window
/// - **line-end:** opening brackets (e.g. `「` `（` `(`) move to the next line
/// - **line-start:** closing punctuation (e.g. `。` `、` `)`) stays with the previous line
///
/// The wrap budget may shrink by a few chars. `max_chars == 0` means no wrapping.
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
        // Prefer breaking just after Japanese clause / list punctuation inside the window.
        if let Some(rel) = chars[start..end]
            .iter()
            .rposition(|c| is_prefer_break_after(*c))
        {
            let after = start + rel + 1;
            if after > start && after < chars.len() {
                end = after;
            }
        }
        // Line-end kinsoku: don't finish a line on an opening bracket.
        while end > start + 1 && is_not_line_end(chars[end - 1]) {
            end -= 1;
        }
        // Line-start kinsoku: don't leave forbidden chars at the head of the remainder.
        while end > start + 1 && end < chars.len() && is_not_line_start(chars[end]) {
            end -= 1;
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

/// Characters that are good soft-wrap points when they appear mid-window (JLReq-ish).
fn is_prefer_break_after(c: char) -> bool {
    matches!(
        c,
        '。' | '、' | '！' | '？' | '．' | '，' | '・' | '：' | '；' | '‥' | '…'
    )
}

/// Characters that must not end a line (subset of JLReq 禁則処理).
fn is_not_line_end(c: char) -> bool {
    matches!(
        c,
        '「' | '『' | '（' | '［' | '｛' | '〈' | '《' | '〔' | '【' | '(' | '[' | '{'
    )
}

/// Characters that must not begin a line (subset of JLReq 禁則処理).
fn is_not_line_start(c: char) -> bool {
    matches!(
        c,
        '。' | '、'
            | '．'
            | '，'
            | '）'
            | '］'
            | '｝'
            | '」'
            | '』'
            | '〉'
            | '》'
            | '〕'
            | '】'
            | 'ー'
            | '゛'
            | '゜'
            | 'ゝ'
            | 'ゞ'
            | '々'
            | 'ぁ'
            | 'ぃ'
            | 'ぅ'
            | 'ぇ'
            | 'ぉ'
            | 'っ'
            | 'ゃ'
            | 'ゅ'
            | 'ょ'
            | 'ゎ'
            | 'ァ'
            | 'ィ'
            | 'ゥ'
            | 'ェ'
            | 'ォ'
            | 'ッ'
            | 'ャ'
            | 'ュ'
            | 'ョ'
            | 'ヮ'
            | ')'
            | ']'
            | '}'
            | ','
            | '.'
            | ';'
            | ':'
            | '!'
            | '?'
            | '！'
            | '？'
            | '％'
            | '°'
            | '′'
            | '″'
            | '・'
            | '：'
            | '；'
            | '‥'
            | '…'
    )
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
    fn wrap_avoids_line_start_with_cjk_period() {
        // Without kinsoku, max=3 on "あああ。" yields ["あああ", "。"].
        let lines = wrap_line("あああ。", 3);
        assert!(
            lines.iter().all(|l| !l.starts_with('。')),
            "period must not start a line: {lines:?}"
        );
        assert_eq!(lines, vec!["ああ", "あ。"]);
    }

    #[test]
    fn wrap_prefers_break_after_cjk_period() {
        // Prefer ending the first line on 。 rather than after the next kana.
        let lines = wrap_line("あい。うえお", 4);
        assert_eq!(lines, vec!["あい。", "うえお"]);
    }

    #[test]
    fn wrap_prefers_break_after_middle_dot() {
        let lines = wrap_line("赤・青・緑いろ", 4);
        assert_eq!(lines, vec!["赤・青・", "緑いろ"]);
    }

    #[test]
    fn wrap_avoids_line_start_with_middle_dot() {
        let lines = wrap_line("あいう・え", 3);
        assert!(lines.iter().all(|l| !l.starts_with('・')), "{lines:?}");
    }

    #[test]
    fn wrap_avoids_line_start_with_fullwidth_question() {
        let lines = wrap_line("あああ？", 3);
        assert!(
            lines.iter().all(|l| !l.starts_with('？')),
            "fullwidth ? must not start a line: {lines:?}"
        );
        assert_eq!(lines, vec!["ああ", "あ？"]);
    }

    #[test]
    fn wrap_avoids_line_end_with_opening_bracket() {
        // Without kinsoku, max=2 on "あ「いう" yields ["あ「", "いう"].
        let lines = wrap_line("あ「いう", 2);
        assert!(
            lines.iter().all(|l| !l.ends_with('「')),
            "'「' must not end a line: {lines:?}"
        );
        assert_eq!(lines, vec!["あ", "「い", "う"]);
    }

    #[test]
    fn wrap_avoids_line_start_with_closing_paren() {
        let lines = wrap_line("abc)", 3);
        assert!(
            lines.iter().all(|l| !l.starts_with(')')),
            "')' must not start a line: {lines:?}"
        );
        assert_eq!(lines, vec!["ab", "c)"]);
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
    fn wrap_single_kinsoku_char_still_emits() {
        // A lone forbidden-start char must still appear (no infinite shrink).
        assert_eq!(wrap_line("。", 1), vec!["。".to_string()]);
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
            PlacedItem::Image { y_mm, .. } => *y_mm >= DocFrame::A4.bottom_mm,
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
