//! JLReq Profile v1 line engine (font-backed). Stub APIs in `reciplexa-std` stay unchanged.

use reciplexa_scene::{Shape, Text};
use reciplexa_std::japanese::{CharClass, ParagraphSceneLayout};

use crate::error::LayoutError;
use crate::font::LoadedFont;
use crate::ja_tables::{
    class_of, hang_em, hangable, line_end_prohibited, line_head_prohibited, pair_break,
    trim_em as table_trim_em, trimmable_line_end, trimmable_line_head,
};
use crate::position::{positioned_line_to_glyph_shapes, PositionedLine};
use crate::shape::{shape_run, ShapedGlyph, ShapedRun};

pub use crate::ja_tables::JLREQ_PROFILE_V1_TABLES;

/// Declared JLReq Profile v1 repertoire and §C subset.
///
/// Repertoire: ASCII printable, hiragana, katakana, listed CJK punctuation,
/// and the fixture ideograph set. Vertical writing / `vert` is Step 8 item 2
/// (`layout_vertical_run`). Tate-chu-yoko and bou remain later follow-ups.
///
/// §C subset: versioned class-level pair matrix [`JLREQ_PROFILE_V1_TABLES`]
/// (cl-01..cl-30 buckets), cl-08 inseparable glue, hangable cl-06/07 with
/// hang-width tolerance, line-head/end kinsoku, line-head/end trimming, and
/// allowed-gap justification.
pub const JLREQ_PROFILE_V1: &str = "jlreq-profile-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakReason {
    Measure,
    Hang,
    InseparableGlue,
    Forced,
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineSegment {
    pub text: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub natural_width_em: f64,
    pub hang_em: f64,
    pub trim_em: f64,
    pub reason: BreakReason,
    pub glyphs: Vec<ShapedGlyph>,
}

/// Font-backed advance in em. Distinct from stub `char_em_width` for fixture 'A' / '。'.
pub fn glyph_advance_em(font: &LoadedFont, c: char) -> Result<f64, LayoutError> {
    font.hor_advance_em(c)
}

pub fn measure_run_em(font: &LoadedFont, text: &str) -> Result<f64, LayoutError> {
    Ok(shape_run(font, text)?.width_em())
}

/// Break `text` to `max_em` using font advances and the std class matrix.
pub fn break_line_font(
    font: &LoadedFont,
    text: &str,
    max_em: f64,
) -> Result<Vec<LineSegment>, LayoutError> {
    let run = shape_run(font, text)?;
    break_shaped_run(&run, max_em)
}

pub fn break_shaped_run(run: &ShapedRun, max_em: f64) -> Result<Vec<LineSegment>, LayoutError> {
    let glyphs = &run.glyphs;
    if glyphs.is_empty() {
        return Ok(Vec::new());
    }
    if max_em.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
        return Ok(vec![segment(run, 0, glyphs.len(), 0.0, BreakReason::End)]);
    }

    let chars: Vec<char> = glyphs.iter().map(|g| g.ch).collect();
    let mut out = Vec::new();
    let mut start = 0usize;
    while start < glyphs.len() {
        let mut end = start;
        let mut used = 0.0;
        let mut reason = BreakReason::Measure;
        while end < glyphs.len() {
            let w = glyphs[end].advance_em;
            if end > start && used + w > max_em {
                if hangable(class_of(chars[end])) {
                    let hang = hang_em(class_of(chars[end]));
                    if used + w <= max_em + hang {
                        end += 1;
                        reason = BreakReason::Hang;
                    }
                }
                break;
            }
            used += w;
            end += 1;
        }
        let glued = extend_cl08(&chars, start, end);
        if glued > end {
            reason = BreakReason::InseparableGlue;
            end = glued;
        }
        if end >= glyphs.len() {
            out.push(segment(
                run,
                start,
                glyphs.len(),
                hang_of(&chars, start, glyphs.len()),
                BreakReason::End,
            ));
            break;
        }
        let mut cut = choose_cut(&chars, start, end);
        while cut > start + 1 && line_end_prohibited(class_of(chars[cut - 1])) {
            cut -= 1;
        }
        while cut > start + 1 && cut < chars.len() && line_head_prohibited(class_of(chars[cut])) {
            cut -= 1;
        }
        cut = snap_cl08(&chars, start, cut);
        if cut <= start {
            cut = start + 1;
            reason = BreakReason::Forced;
        }
        let hang = hang_of(&chars, start, cut);
        out.push(segment(run, start, cut, hang, reason));
        start = cut;
    }
    Ok(out)
}

/// Profile v1 line-head / line-end trim plus allowed-gap justification.
pub fn justify_line_font(
    font: &LoadedFont,
    chars: &[char],
    target_em: f64,
) -> Result<Vec<(char, f64)>, LayoutError> {
    if chars.is_empty() {
        return Ok(Vec::new());
    }
    let widths: Vec<f64> = chars
        .iter()
        .copied()
        .map(|c| font.hor_advance_em(c))
        .collect::<Result<_, _>>()?;
    let mut natural: f64 = widths.iter().sum();
    let mut trim = 0.0;
    if let Some(&first) = chars.first() {
        let class = class_of(first);
        if trimmable_line_head(class) {
            let t = table_trim_em(class).min(widths[0]);
            natural -= t;
            trim += t;
        }
    }
    if chars.len() > 1 {
        if let Some(&last) = chars.last() {
            let class = class_of(last);
            if trimmable_line_end(class) {
                let t = table_trim_em(class).min(*widths.last().unwrap_or(&0.0));
                natural -= t;
                trim += t;
            }
        }
    }
    let _ = (trim, font);
    let mut gap_extra = vec![0.0_f64; chars.len().saturating_sub(1)];
    if target_em > natural && chars.len() > 1 {
        let mut allowed_idx = Vec::new();
        for i in 0..chars.len() - 1 {
            if pair_break(chars[i], chars[i + 1]).may_break() {
                allowed_idx.push(i);
            }
        }
        if !allowed_idx.is_empty() {
            let extra = (target_em - natural) / allowed_idx.len() as f64;
            for i in allowed_idx {
                gap_extra[i] = extra;
            }
        }
    }
    let mut x = 0.0;
    if let Some(&first) = chars.first() {
        let class = class_of(first);
        if trimmable_line_head(class) {
            x -= table_trim_em(class).min(widths[0]);
        }
    }
    let mut out = Vec::with_capacity(chars.len());
    for i in 0..chars.len() {
        out.push((chars[i], x));
        x += widths[i];
        if i < gap_extra.len() {
            x += gap_extra[i];
        }
    }
    Ok(out)
}

fn extend_cl08(chars: &[char], start: usize, mut end: usize) -> usize {
    if end <= start || end == 0 {
        return end;
    }
    while end < chars.len()
        && class_of(chars[end]) == CharClass::Inseparable
        && class_of(chars[end - 1]) == CharClass::Inseparable
    {
        end += 1;
    }
    end
}

fn snap_cl08(chars: &[char], start: usize, mut cut: usize) -> usize {
    while cut > start + 1
        && cut < chars.len()
        && class_of(chars[cut - 1]) == CharClass::Inseparable
        && class_of(chars[cut]) == CharClass::Inseparable
    {
        cut -= 1;
    }
    cut
}

fn choose_cut(chars: &[char], start: usize, end: usize) -> usize {
    let mut cut = end;
    let mut found = false;
    let mut space_cut: Option<usize> = None;
    for cand in (start + 1..=end).rev() {
        if cand < chars.len() && pair_break(chars[cand - 1], chars[cand]).may_break() {
            if !found {
                cut = cand;
                found = true;
            }
            if chars[cand].is_ascii_whitespace() {
                space_cut = Some(cand);
                break;
            }
        }
    }
    if let Some(s) = space_cut {
        s
    } else if found {
        cut
    } else {
        end
    }
}

fn hang_of(chars: &[char], start: usize, end: usize) -> f64 {
    if end <= start {
        return 0.0;
    }
    let last = chars[end - 1];
    if hangable(class_of(last)) {
        hang_em(class_of(last))
    } else {
        0.0
    }
}

fn segment(
    run: &ShapedRun,
    start: usize,
    end: usize,
    hang_em_val: f64,
    reason: BreakReason,
) -> LineSegment {
    let glyphs = run.glyphs[start..end].to_vec();
    let text: String = glyphs.iter().map(|g| g.ch).collect();
    let start_byte = glyphs.first().map(|g| g.cluster_start).unwrap_or(0);
    let end_byte = glyphs.last().map(|g| g.cluster_end).unwrap_or(start_byte);
    let natural_width_em = glyphs.iter().map(|g| g.advance_em).sum();
    let mut trim = 0.0;
    if let Some(g) = glyphs.first() {
        let class = class_of(g.ch);
        if trimmable_line_head(class) {
            trim += table_trim_em(class);
        }
    }
    if glyphs.len() > 1 {
        if let Some(g) = glyphs.last() {
            let class = class_of(g.ch);
            if trimmable_line_end(class) {
                trim += table_trim_em(class);
            }
        }
    }
    LineSegment {
        text,
        start_byte,
        end_byte,
        natural_width_em,
        hang_em: hang_em_val,
        trim_em: trim,
        reason,
        glyphs,
    }
}

/// Font-backed wrap + indent, returning GlyphRun-bearing [`PositionedLine`]s.
///
/// Non-last lines are justified to `max_em`. Line-head trim shifts `x_mm`.
/// Scene adapters still emit one [`Text`] per line (PT-6); glyph x lives on
/// the [`PositionedLine`].
pub fn layout_wrapped_paragraph_product_lines(
    font: &LoadedFont,
    text: &str,
    max_em: f64,
    indent_em: f64,
    layout: &ParagraphSceneLayout,
) -> Result<Vec<PositionedLine>, LayoutError> {
    let segs = break_line_font(font, text, max_em)?;
    if segs.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::with_capacity(segs.len());
    let n = segs.len();
    for (i, seg) in segs.iter().enumerate() {
        let indent = if i == 0 { indent_em } else { 0.0 };
        let origin_x = layout.base_x_mm + indent * layout.size_mm;
        let origin_y = layout.start_y_mm + layout.pitch_mm * i as f64;
        let mut line = PositionedLine::from_segment(
            font.id.clone(),
            seg,
            origin_x,
            origin_y,
            layout.size_mm,
            "ja",
        );
        let chars: Vec<char> = seg.text.chars().collect();
        let target = if i + 1 < n {
            max_em
        } else {
            seg.natural_width_em
        };
        let placed = justify_line_font(font, &chars, target)?;
        for (glyph, (_, x_em)) in line.run.glyphs.iter_mut().zip(placed.iter()) {
            glyph.x_mm = origin_x + x_em * layout.size_mm;
        }
        if let Some(first) = line.run.glyphs.first() {
            line.x_mm = first.x_mm;
        }
        out.push(line);
    }
    Ok(out)
}

/// Font-backed paragraph wrap + first-line indent → one scene glyph per cluster.
pub fn layout_wrapped_paragraph_product(
    font: &LoadedFont,
    text: &str,
    max_em: f64,
    indent_em: f64,
    layout: &ParagraphSceneLayout,
) -> Result<Vec<Shape>, LayoutError> {
    let lines = layout_wrapped_paragraph_product_lines(font, text, max_em, indent_em, layout)?;
    if lines.is_empty() {
        return Ok(vec![Shape::Text(Text {
            x_mm: layout.base_x_mm,
            y_mm: layout.start_y_mm,
            size_mm: layout.size_mm,
            width_mm: None,
            height_mm: None,
            content: String::new(),
            fill: layout.fill,
        })]);
    }
    Ok(lines
        .iter()
        .flat_map(|l| positioned_line_to_glyph_shapes(l, layout.fill))
        .collect())
}

/// One paragraph per column using font-backed wrap.
pub fn layout_column_paragraph_product(
    font: &LoadedFont,
    paragraphs: &[String],
    total_em: f64,
    count: u32,
    gutter_em: f64,
    layout: &ParagraphSceneLayout,
) -> Result<(Vec<Shape>, f64), LayoutError> {
    let n = count.max(1) as f64;
    let gutter_total = gutter_em * (n - 1.0);
    let col_w = ((total_em - gutter_total) / n).max(0.0);
    let budget = if col_w > 0.0 { col_w } else { 40.0 };
    let mut shapes = Vec::new();
    let mut min_y = layout.start_y_mm;
    for (i, text) in paragraphs.iter().enumerate().take(count as usize) {
        let x_em = (col_w + gutter_em) * i as f64;
        let column_layout = ParagraphSceneLayout {
            base_x_mm: layout.base_x_mm + x_em * layout.size_mm,
            ..*layout
        };
        let col_shapes = layout_wrapped_paragraph_product(font, text, budget, 0.0, &column_layout)?;
        for s in &col_shapes {
            if let Some(y) = s.text_y_mm() {
                if y < min_y {
                    min_y = y;
                }
            }
        }
        shapes.extend(col_shapes);
    }
    Ok((shapes, min_y))
}
