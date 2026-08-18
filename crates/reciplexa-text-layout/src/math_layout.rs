//! OpenType MATH Profile v1 box engine.

use reciplexa_std::math::{
    class_spacing_em, EstimateStyle, MathAccentKind, MathAtom, MathBox, MathClass, MathMatrixKind,
    MathStackKind,
};
use ttf_parser::GlyphId;

use crate::error::LayoutError;
use crate::font::LoadedFont;
use crate::shape::{shape_run, ShapedGlyph};

/// Declared Math Profile v1: symbols, rows, scripts, fractions, radicals,
/// stretchy delimiter variants (GID recorded; scene size carries stretch),
/// hat/tilde/dot/vec marks, bar/underline rules, display big operators,
/// matrices, aligned/stack. Check/breve/acute/grave/ring marks error rather
/// than ASCII substitution. Full glyph assembly recipes remain OPEN.
pub const MATH_PROFILE_V1: &str = "math-profile-v1";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MathConstantsEm {
    pub script_percent_scale_down: f64,
    pub script_script_percent_scale_down: f64,
    pub axis_height_em: f64,
    pub superscript_shift_up_em: f64,
    pub subscript_shift_down_em: f64,
    pub fraction_rule_thickness_em: f64,
    pub fraction_num_shift_up_em: f64,
    pub fraction_den_shift_down_em: f64,
    pub radical_rule_thickness_em: f64,
    pub display_operator_min_height_em: f64,
}

impl MathConstantsEm {
    pub fn from_font(font: &LoadedFont) -> Result<Self, LayoutError> {
        let face = font.face();
        let math = face
            .tables()
            .math
            .ok_or_else(|| LayoutError::MissingMathTable {
                font_id: font.id.as_key(),
                detail: "MATH table missing".into(),
            })?;
        let c = math
            .constants
            .ok_or_else(|| LayoutError::MissingMathTable {
                font_id: font.id.as_key(),
                detail: "MATH constants missing".into(),
            })?;
        let upem = f64::from(font.units_per_em().max(1));
        Ok(Self {
            script_percent_scale_down: f64::from(c.script_percent_scale_down()) / 100.0,
            script_script_percent_scale_down: f64::from(c.script_script_percent_scale_down())
                / 100.0,
            axis_height_em: f64::from(c.axis_height().value) / upem,
            superscript_shift_up_em: f64::from(c.superscript_shift_up().value) / upem,
            subscript_shift_down_em: f64::from(c.subscript_shift_down().value) / upem,
            fraction_rule_thickness_em: f64::from(c.fraction_rule_thickness().value) / upem,
            fraction_num_shift_up_em: f64::from(c.fraction_numerator_shift_up().value) / upem,
            fraction_den_shift_down_em: f64::from(c.fraction_denominator_shift_down().value) / upem,
            radical_rule_thickness_em: f64::from(c.radical_rule_thickness().value) / upem,
            display_operator_min_height_em: f64::from(c.display_operator_min_height()) / upem,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PositionedMathGlyph {
    pub glyph: ShapedGlyph,
    pub x_em: f64,
    pub y_em: f64,
    pub scale: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PositionedMath {
    pub metrics: MathBox,
    pub glyphs: Vec<PositionedMathGlyph>,
    /// Rule segments in em, y up from baseline.
    pub rules: Vec<MathRule>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MathRule {
    pub x0_em: f64,
    pub y0_em: f64,
    pub x1_em: f64,
    pub y1_em: f64,
    pub thickness_em: f64,
}

/// Layout a math atom with OpenType MATH constants and font advances.
pub fn layout_math_atom(
    font: &LoadedFont,
    atom: &MathAtom,
    style: EstimateStyle,
) -> Result<PositionedMath, LayoutError> {
    let c = MathConstantsEm::from_font(font)?;
    layout_atom(font, &c, atom, style, 1.0)
}

fn script_scale(c: &MathConstantsEm, style: EstimateStyle) -> f64 {
    match style {
        EstimateStyle::Display => c.script_percent_scale_down,
        EstimateStyle::Text => c.script_script_percent_scale_down,
    }
}

fn layout_atom(
    font: &LoadedFont,
    c: &MathConstantsEm,
    atom: &MathAtom,
    style: EstimateStyle,
    scale: f64,
) -> Result<PositionedMath, LayoutError> {
    match atom {
        MathAtom::Symbol { glyph, .. } => layout_symbol(font, glyph, scale),
        MathAtom::Row { children, .. } => layout_row(font, c, children, style, scale),
        MathAtom::Fraction {
            numerator,
            denominator,
            ..
        } => layout_fraction(font, c, numerator, denominator, style, scale),
        MathAtom::Scripts {
            base,
            superscript,
            subscript,
            ..
        } => layout_scripts(
            font,
            c,
            base,
            superscript.as_deref(),
            subscript.as_deref(),
            style,
            scale,
        ),
        MathAtom::Delimiter {
            left,
            right,
            body,
            stretch_factor,
            ..
        } => layout_delimiter(font, c, left, right, body, *stretch_factor, style, scale),
        MathAtom::Radical {
            index, radicand, ..
        } => layout_radical(font, c, index.as_deref(), radicand, style, scale),
        MathAtom::Accent { kind, base, .. } => layout_accent(font, c, *kind, base, style, scale),
        MathAtom::BigOp {
            operator,
            lower,
            upper,
            body,
            ..
        } => layout_bigop(
            font,
            c,
            operator,
            lower.as_deref(),
            upper.as_deref(),
            body.as_deref(),
            style,
            scale,
        ),
        MathAtom::Matrix {
            kind,
            rows,
            left,
            right,
            ..
        } => layout_matrix(
            font,
            c,
            *kind,
            rows,
            left.as_deref(),
            right.as_deref(),
            style,
            scale,
        ),
        MathAtom::Aligned { rows, .. } => layout_stack_rows(font, c, rows, style, scale, false),
        MathAtom::Stack { kind, children, .. } => {
            layout_stack(font, c, *kind, children, style, scale)
        }
    }
}

fn layout_symbol(
    font: &LoadedFont,
    glyph: &str,
    scale: f64,
) -> Result<PositionedMath, LayoutError> {
    let run = shape_run(font, glyph)?;
    let mut x = 0.0;
    let mut glyphs = Vec::new();
    for g in run.glyphs {
        let w = g.advance_em * scale;
        glyphs.push(PositionedMathGlyph {
            glyph: g,
            x_em: x,
            y_em: 0.0,
            scale,
        });
        x += w;
    }
    Ok(PositionedMath {
        metrics: MathBox {
            width: x,
            height: 0.7 * scale,
            depth: 0.2 * scale,
        },
        glyphs,
        rules: Vec::new(),
    })
}

fn layout_row(
    font: &LoadedFont,
    c: &MathConstantsEm,
    children: &[MathAtom],
    style: EstimateStyle,
    scale: f64,
) -> Result<PositionedMath, LayoutError> {
    let mut out = PositionedMath {
        metrics: MathBox {
            width: 0.0,
            height: 0.0,
            depth: 0.0,
        },
        glyphs: Vec::new(),
        rules: Vec::new(),
    };
    let mut prev_class: Option<MathClass> = None;
    for child in children {
        let laid = layout_atom(font, c, child, style, scale)?;
        let space = match (prev_class, atom_class(child)) {
            (Some(l), Some(r)) => class_spacing_em(l, r) * scale,
            _ => 0.0,
        };
        let dx = out.metrics.width + space;
        append_shifted(&mut out, laid, dx, 0.0);
        prev_class = atom_class(child).or(prev_class);
    }
    Ok(out)
}

fn atom_class(atom: &MathAtom) -> Option<MathClass> {
    match atom {
        MathAtom::Symbol { class, .. } => Some(*class),
        _ => None,
    }
}

fn layout_scripts(
    font: &LoadedFont,
    c: &MathConstantsEm,
    base: &MathAtom,
    sup: Option<&MathAtom>,
    sub: Option<&MathAtom>,
    style: EstimateStyle,
    scale: f64,
) -> Result<PositionedMath, LayoutError> {
    let base_l = layout_atom(font, c, base, style, scale)?;
    let ss = script_scale(c, style) * scale;
    let script_style = EstimateStyle::Text;
    let mut out = base_l;
    let base_w = out.metrics.width;
    if let Some(sup) = sup {
        let s = layout_atom(font, c, sup, script_style, ss)?;
        let dy = c.superscript_shift_up_em * scale;
        let italic = out
            .glyphs
            .last()
            .map(|g| g.glyph.italic_correction_em * g.scale)
            .unwrap_or(0.0);
        append_shifted(&mut out, s, base_w + italic, dy);
    }
    if let Some(sub) = sub {
        let s = layout_atom(font, c, sub, script_style, ss)?;
        let dy = -c.subscript_shift_down_em * scale;
        append_shifted(&mut out, s, base_w, dy);
    }
    if sup.is_some() || sub.is_some() {
        // Width: base + max(script widths) after the last append already shifted.
        // Recompute width from glyph extents.
        out.metrics.width = extent_width(&out);
        out.metrics.height = out
            .metrics
            .height
            .max(c.superscript_shift_up_em * scale + 0.4 * ss);
        if sub.is_some() {
            out.metrics.depth = out
                .metrics
                .depth
                .max(c.subscript_shift_down_em * scale + 0.3 * ss);
        }
    }
    Ok(out)
}

fn layout_fraction(
    font: &LoadedFont,
    c: &MathConstantsEm,
    num: &MathAtom,
    den: &MathAtom,
    style: EstimateStyle,
    scale: f64,
) -> Result<PositionedMath, LayoutError> {
    let n = layout_atom(font, c, num, style, scale)?;
    let d = layout_atom(font, c, den, style, scale)?;
    let n_w = n.metrics.width;
    let d_w = d.metrics.width;
    let n_h = n.metrics.height;
    let d_d = d.metrics.depth;
    let width = n_w.max(d_w);
    let axis = c.axis_height_em * scale;
    let rule = c.fraction_rule_thickness_em * scale;
    let num_y = axis + c.fraction_num_shift_up_em * scale;
    let den_y = axis - c.fraction_den_shift_down_em * scale;
    let mut out = PositionedMath {
        metrics: MathBox {
            width,
            height: num_y + n_h,
            depth: (axis - den_y) + d_d,
        },
        glyphs: Vec::new(),
        rules: vec![MathRule {
            x0_em: 0.0,
            y0_em: axis,
            x1_em: width,
            y1_em: axis,
            thickness_em: rule,
        }],
    };
    append_shifted(&mut out, n, (width - n_w) * 0.5, num_y);
    append_shifted(&mut out, d, (width - d_w) * 0.5, den_y);
    Ok(out)
}

fn layout_radical(
    font: &LoadedFont,
    c: &MathConstantsEm,
    index: Option<&MathAtom>,
    radicand: &MathAtom,
    style: EstimateStyle,
    scale: f64,
) -> Result<PositionedMath, LayoutError> {
    let body = layout_atom(font, c, radicand, style, scale)?;
    let surd = layout_symbol(font, "√", scale)?;
    let surd_w = surd.metrics.width;
    let body_w = body.metrics.width;
    let mut width = surd_w + body_w;
    let rule_th = c.radical_rule_thickness_em * scale;
    let rule_y = body.metrics.height + rule_th + 0.05 * scale;
    let mut out = PositionedMath {
        metrics: MathBox {
            width,
            height: rule_y + rule_th,
            depth: body.metrics.depth,
        },
        glyphs: Vec::new(),
        rules: vec![MathRule {
            x0_em: surd_w,
            y0_em: rule_y,
            x1_em: surd_w + body_w,
            y1_em: rule_y,
            thickness_em: rule_th,
        }],
    };
    append_shifted(&mut out, surd, 0.0, 0.0);
    append_shifted(&mut out, body, surd_w, 0.0);
    if let Some(index) = index {
        let idx = layout_atom(
            font,
            c,
            index,
            EstimateStyle::Text,
            script_scale(c, style) * scale,
        )?;
        width = width.max(idx.metrics.width + 0.1);
        append_shifted(&mut out, idx, 0.0, rule_y * 0.5);
        out.metrics.width = width;
    }
    out.metrics.width = extent_width(&out);
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn layout_delimiter(
    font: &LoadedFont,
    c: &MathConstantsEm,
    left: &str,
    right: &str,
    body: &MathAtom,
    stretch: f64,
    style: EstimateStyle,
    scale: f64,
) -> Result<PositionedMath, LayoutError> {
    let body_l = layout_atom(font, c, body, style, scale)?;
    let target = (body_l.metrics.total_height() * stretch).max(0.5);
    let mut out = PositionedMath {
        metrics: MathBox {
            width: 0.0,
            height: body_l.metrics.height.max(target * 0.5),
            depth: body_l.metrics.depth.max(target * 0.5),
        },
        glyphs: Vec::new(),
        rules: Vec::new(),
    };
    let _ = c;
    if !left.is_empty() {
        append_shifted(
            &mut out,
            stretchy_delim(font, left, target, scale)?,
            0.0,
            0.0,
        );
    }
    let x = out.metrics.width;
    append_shifted(&mut out, body_l, x, 0.0);
    if !right.is_empty() {
        let x = out.metrics.width;
        append_shifted(
            &mut out,
            stretchy_delim(font, right, target, scale)?,
            x,
            0.0,
        );
    }
    Ok(out)
}

fn stretchy_delim(
    font: &LoadedFont,
    delim: &str,
    target_em: f64,
    scale: f64,
) -> Result<PositionedMath, LayoutError> {
    let ch = delim.chars().next().unwrap_or('(');
    let gid = font.glyph_id(ch)?;
    let face = font.face();
    let upem = f64::from(font.units_per_em().max(1));
    let mut chosen = gid;
    let mut chosen_adv = font.hor_advance_units(gid)?;
    let mut chosen_h = 1.0;
    if let Some(math) = face.tables().math {
        if let Some(var) = math.variants {
            if let Some(cons) = var.vertical_constructions.get(GlyphId(gid)) {
                for i in 0..cons.variants.len() {
                    if let Some(v) = cons.variants.get(i) {
                        let h = f64::from(v.advance_measurement) / upem;
                        chosen = v.variant_glyph.0;
                        chosen_adv = font.hor_advance_units(chosen).unwrap_or(chosen_adv);
                        chosen_h = h.max(0.25);
                        if h * scale >= target_em {
                            break;
                        }
                    }
                }
            }
        }
    }
    let g = ShapedGlyph {
        gid: chosen,
        ch,
        cluster_start: 0,
        cluster_end: delim.len(),
        advance_em: f64::from(chosen_adv) / upem,
        italic_correction_em: 0.0,
    };
    // Scene still emits Unicode `ch`, so size must carry stretch when the
    // variant GID cannot be painted.
    let visual_scale = scale * (target_em / chosen_h).max(1.0);
    let w = g.advance_em * visual_scale;
    Ok(PositionedMath {
        metrics: MathBox {
            width: w,
            height: target_em * 0.5,
            depth: target_em * 0.5,
        },
        glyphs: vec![PositionedMathGlyph {
            glyph: g,
            x_em: 0.0,
            y_em: 0.0,
            scale: visual_scale,
        }],
        rules: Vec::new(),
    })
}

fn layout_accent(
    font: &LoadedFont,
    c: &MathConstantsEm,
    kind: MathAccentKind,
    base: &MathAtom,
    style: EstimateStyle,
    scale: f64,
) -> Result<PositionedMath, LayoutError> {
    let b = layout_atom(font, c, base, style, scale)?;
    let mut out = b;
    match kind {
        MathAccentKind::Underline | MathAccentKind::Underbar => {
            let y = -out.metrics.depth - 0.08 * scale;
            out.rules.push(MathRule {
                x0_em: 0.0,
                y0_em: y,
                x1_em: out.metrics.width.max(0.3 * scale),
                y1_em: y,
                thickness_em: 0.06 * scale,
            });
            out.metrics.depth += 0.15 * scale;
            return Ok(out);
        }
        MathAccentKind::Overline | MathAccentKind::Bar => {
            let y = out.metrics.height + 0.08 * scale;
            out.rules.push(MathRule {
                x0_em: 0.0,
                y0_em: y,
                x1_em: out.metrics.width.max(0.3 * scale),
                y1_em: y,
                thickness_em: 0.06 * scale,
            });
            out.metrics.height += 0.15 * scale;
            return Ok(out);
        }
        _ => {}
    }
    let mark = match kind {
        MathAccentKind::Hat | MathAccentKind::WideHat => "^",
        MathAccentKind::Tilde | MathAccentKind::WideTilde => "~",
        MathAccentKind::Dot => ".",
        MathAccentKind::Ddot => "..",
        MathAccentKind::Vec => "→",
        MathAccentKind::Check
        | MathAccentKind::Breve
        | MathAccentKind::Acute
        | MathAccentKind::Grave
        | MathAccentKind::Ring => {
            return Err(LayoutError::Engine {
                detail: format!(
                    "math accent `{}` has no Profile v1 mark; refusing ASCII substitution",
                    kind.as_str()
                ),
            });
        }
        MathAccentKind::Underline
        | MathAccentKind::Underbar
        | MathAccentKind::Overline
        | MathAccentKind::Bar => unreachable!("handled as rules"),
    };
    let acc = layout_symbol(font, mark, scale * 0.7)?;
    let dy = out.metrics.height + 0.1 * scale;
    let acc_x = (out.metrics.width - acc.metrics.width).max(0.0) * 0.5;
    append_shifted(&mut out, acc, acc_x, dy);
    out.metrics.height += 0.35 * scale;
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn layout_bigop(
    font: &LoadedFont,
    c: &MathConstantsEm,
    operator: &str,
    lower: Option<&MathAtom>,
    upper: Option<&MathAtom>,
    body: Option<&MathAtom>,
    style: EstimateStyle,
    scale: f64,
) -> Result<PositionedMath, LayoutError> {
    let op = if operator.chars().count() == 1 && style == EstimateStyle::Display {
        stretchy_delim(
            font,
            operator,
            (c.display_operator_min_height_em * scale).max(scale),
            scale,
        )?
    } else {
        layout_symbol(font, operator, scale)?
    };
    let mut out = op;
    let ss = script_scale(c, style) * scale;
    let st = EstimateStyle::Text;
    if let Some(u) = upper {
        let u = layout_atom(font, c, u, st, ss)?;
        let dx = (out.metrics.width - u.metrics.width).max(0.0) * 0.5;
        append_shifted(
            &mut out,
            u,
            dx,
            c.display_operator_min_height_em * 0.35 * scale,
        );
    }
    if let Some(l) = lower {
        let l = layout_atom(font, c, l, st, ss)?;
        let dx = (out.metrics.width - l.metrics.width).max(0.0) * 0.5;
        append_shifted(&mut out, l, dx, -c.subscript_shift_down_em * scale);
    }
    if let Some(b) = body {
        let b = layout_atom(font, c, b, style, scale)?;
        let bx = out.metrics.width + 0.15 * scale;
        append_shifted(&mut out, b, bx, 0.0);
    }
    out.metrics.width = extent_width(&out);
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn layout_matrix(
    font: &LoadedFont,
    c: &MathConstantsEm,
    kind: MathMatrixKind,
    rows: &[Vec<MathAtom>],
    left: Option<&str>,
    right: Option<&str>,
    style: EstimateStyle,
    scale: f64,
) -> Result<PositionedMath, LayoutError> {
    let inner = layout_stack_rows(font, c, rows, style, scale, true)?;
    let (l, r) = match kind {
        MathMatrixKind::BMatrix => (left.unwrap_or("["), right.unwrap_or("]")),
        MathMatrixKind::PMatrix => (left.unwrap_or("("), right.unwrap_or(")")),
        MathMatrixKind::VMatrix => (left.unwrap_or("|"), right.unwrap_or("|")),
        _ => (left.unwrap_or(""), right.unwrap_or("")),
    };
    if l.is_empty() && r.is_empty() {
        return Ok(inner);
    }
    let h = inner.metrics.total_height();
    let mut out = PositionedMath {
        metrics: MathBox {
            width: 0.0,
            height: inner.metrics.height,
            depth: inner.metrics.depth,
        },
        glyphs: Vec::new(),
        rules: Vec::new(),
    };
    if !l.is_empty() {
        append_shifted(&mut out, stretchy_delim(font, l, h, scale)?, 0.0, 0.0);
    }
    let x = out.metrics.width;
    append_shifted(&mut out, inner, x, 0.0);
    if !r.is_empty() {
        let x = out.metrics.width;
        append_shifted(&mut out, stretchy_delim(font, r, h, scale)?, x, 0.0);
    }
    Ok(out)
}

fn layout_stack(
    font: &LoadedFont,
    c: &MathConstantsEm,
    kind: MathStackKind,
    children: &[MathAtom],
    style: EstimateStyle,
    scale: f64,
) -> Result<PositionedMath, LayoutError> {
    if children.is_empty() {
        return Ok(PositionedMath {
            metrics: MathBox {
                width: 0.0,
                height: 0.0,
                depth: 0.0,
            },
            glyphs: Vec::new(),
            rules: Vec::new(),
        });
    }
    if kind == MathStackKind::Stackrel && children.len() >= 2 {
        let rows = vec![vec![children[0].clone()], vec![children[1].clone()]];
        return layout_stack_rows(font, c, &rows, style, scale, false);
    }
    let rows: Vec<Vec<MathAtom>> = children.iter().cloned().map(|a| vec![a]).collect();
    layout_stack_rows(font, c, &rows, style, scale, false)
}

fn layout_stack_rows(
    font: &LoadedFont,
    c: &MathConstantsEm,
    rows: &[Vec<MathAtom>],
    style: EstimateStyle,
    scale: f64,
    as_matrix: bool,
) -> Result<PositionedMath, LayoutError> {
    let mut laid_rows: Vec<Vec<PositionedMath>> = Vec::new();
    let mut col_w: Vec<f64> = Vec::new();
    for row in rows {
        let mut cells = Vec::new();
        for (i, cell) in row.iter().enumerate() {
            let l = layout_atom(font, c, cell, style, scale)?;
            if col_w.len() <= i {
                col_w.resize(i + 1, 0.0);
            }
            col_w[i] = col_w[i].max(l.metrics.width);
            cells.push(l);
        }
        laid_rows.push(cells);
    }
    let gap = if as_matrix {
        0.35 * scale
    } else {
        0.25 * scale
    };
    let mut out = PositionedMath {
        metrics: MathBox {
            width: col_w.iter().sum::<f64>() + gap * col_w.len().saturating_sub(1) as f64,
            height: 0.0,
            depth: 0.0,
        },
        glyphs: Vec::new(),
        rules: Vec::new(),
    };
    let mut y = 0.0;
    for (ri, row) in laid_rows.into_iter().enumerate() {
        let row_h = row
            .iter()
            .map(|c| c.metrics.height)
            .fold(0.4 * scale, f64::max);
        let row_d = row
            .iter()
            .map(|c| c.metrics.depth)
            .fold(0.1 * scale, f64::max);
        if ri == 0 {
            out.metrics.height = row_h;
            y = 0.0;
        } else {
            y -= row_h + row_d + gap;
        }
        let mut x = 0.0;
        for (i, cell) in row.into_iter().enumerate() {
            append_shifted(&mut out, cell, x, y);
            x += col_w.get(i).copied().unwrap_or(0.0) + gap;
        }
        out.metrics.depth = (-y) + row_d;
    }
    out.metrics.width = extent_width(&out);
    Ok(out)
}

fn append_shifted(out: &mut PositionedMath, src: PositionedMath, dx: f64, dy: f64) {
    for mut g in src.glyphs {
        g.x_em += dx;
        g.y_em += dy;
        out.glyphs.push(g);
    }
    for r in src.rules {
        out.rules.push(MathRule {
            x0_em: r.x0_em + dx,
            y0_em: r.y0_em + dy,
            x1_em: r.x1_em + dx,
            y1_em: r.y1_em + dy,
            thickness_em: r.thickness_em,
        });
    }
    out.metrics.width = out.metrics.width.max(dx + src.metrics.width);
    out.metrics.height = out.metrics.height.max(src.metrics.height + dy.max(0.0));
    if dy < 0.0 {
        out.metrics.depth = out.metrics.depth.max(src.metrics.depth - dy);
    } else {
        out.metrics.depth = out.metrics.depth.max(src.metrics.depth);
    }
}

fn extent_width(m: &PositionedMath) -> f64 {
    m.glyphs
        .iter()
        .map(|g| g.x_em + g.glyph.advance_em * g.scale)
        .fold(m.metrics.width, f64::max)
}
