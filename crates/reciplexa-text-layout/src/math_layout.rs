//! OpenType MATH Profile v1 box engine.

use reciplexa_std::math::{
    class_spacing_em, EstimateStyle, MathAccentKind, MathAtom, MathBox, MathClass, MathMatrixKind,
    MathStackKind,
};
use ttf_parser::GlyphId;

use crate::error::LayoutError;
use crate::font::{LoadedFont, MathKernCorner};
use crate::shape::{shape_run, ShapedGlyph};

/// Declared Math Profile v1: symbols, rows, scripts, fractions, radicals,
/// stretchy delimiter variants (GID recorded and painted via scene GlyphRun),
/// hat/tilde/dot/vec marks, bar/underline rules, display big operators,
/// matrices, aligned/stack. Check/breve/acute/grave/ring marks error rather
/// than ASCII substitution. Stretchy delimiters use MATH variants, then
/// `GlyphAssembly` (no visual scale). Scripts, limits, and adjacent nuclei
/// apply MATH italic correction and corner kern from the font table.
/// Slice 3 reads remaining layout MATH constants from the font (gaps, bars,
/// radical degree, stack, limits, delimited min height, space after script,
/// display-style fraction shifts); glyph ink uses glyf bbox.
pub const MATH_PROFILE_V1: &str = "math-profile-v1";

fn math_em(v: ttf_parser::math::MathValue<'_>, upem: f64) -> f64 {
    f64::from(v.value) / upem
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MathConstantsEm {
    pub script_percent_scale_down: f64,
    pub script_script_percent_scale_down: f64,
    pub delimited_sub_formula_min_height_em: f64,
    pub axis_height_em: f64,
    pub accent_base_height_em: f64,
    pub superscript_shift_up_em: f64,
    pub subscript_shift_down_em: f64,
    pub space_after_script_em: f64,
    pub upper_limit_gap_min_em: f64,
    pub upper_limit_baseline_rise_min_em: f64,
    pub lower_limit_gap_min_em: f64,
    pub lower_limit_baseline_drop_min_em: f64,
    pub stack_gap_min_em: f64,
    pub stack_display_style_gap_min_em: f64,
    pub fraction_rule_thickness_em: f64,
    pub fraction_num_shift_up_em: f64,
    pub fraction_num_display_style_shift_up_em: f64,
    pub fraction_den_shift_down_em: f64,
    pub fraction_den_display_style_shift_down_em: f64,
    pub overbar_vertical_gap_em: f64,
    pub overbar_rule_thickness_em: f64,
    pub overbar_extra_ascender_em: f64,
    pub underbar_vertical_gap_em: f64,
    pub underbar_rule_thickness_em: f64,
    pub underbar_extra_descender_em: f64,
    pub radical_rule_thickness_em: f64,
    pub radical_vertical_gap_em: f64,
    pub radical_display_style_vertical_gap_em: f64,
    pub radical_extra_ascender_em: f64,
    pub radical_kern_before_degree_em: f64,
    pub radical_kern_after_degree_em: f64,
    pub radical_degree_bottom_raise_percent: f64,
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
            delimited_sub_formula_min_height_em: f64::from(c.delimited_sub_formula_min_height())
                / upem,
            axis_height_em: math_em(c.axis_height(), upem),
            accent_base_height_em: math_em(c.accent_base_height(), upem),
            superscript_shift_up_em: math_em(c.superscript_shift_up(), upem),
            subscript_shift_down_em: math_em(c.subscript_shift_down(), upem),
            space_after_script_em: math_em(c.space_after_script(), upem),
            upper_limit_gap_min_em: math_em(c.upper_limit_gap_min(), upem),
            upper_limit_baseline_rise_min_em: math_em(c.upper_limit_baseline_rise_min(), upem),
            lower_limit_gap_min_em: math_em(c.lower_limit_gap_min(), upem),
            lower_limit_baseline_drop_min_em: math_em(c.lower_limit_baseline_drop_min(), upem),
            stack_gap_min_em: math_em(c.stack_gap_min(), upem),
            stack_display_style_gap_min_em: math_em(c.stack_display_style_gap_min(), upem),
            fraction_rule_thickness_em: math_em(c.fraction_rule_thickness(), upem),
            fraction_num_shift_up_em: math_em(c.fraction_numerator_shift_up(), upem),
            fraction_num_display_style_shift_up_em: math_em(
                c.fraction_numerator_display_style_shift_up(),
                upem,
            ),
            fraction_den_shift_down_em: math_em(c.fraction_denominator_shift_down(), upem),
            fraction_den_display_style_shift_down_em: math_em(
                c.fraction_denominator_display_style_shift_down(),
                upem,
            ),
            overbar_vertical_gap_em: math_em(c.overbar_vertical_gap(), upem),
            overbar_rule_thickness_em: math_em(c.overbar_rule_thickness(), upem),
            overbar_extra_ascender_em: math_em(c.overbar_extra_ascender(), upem),
            underbar_vertical_gap_em: math_em(c.underbar_vertical_gap(), upem),
            underbar_rule_thickness_em: math_em(c.underbar_rule_thickness(), upem),
            underbar_extra_descender_em: math_em(c.underbar_extra_descender(), upem),
            radical_rule_thickness_em: math_em(c.radical_rule_thickness(), upem),
            radical_vertical_gap_em: math_em(c.radical_vertical_gap(), upem),
            radical_display_style_vertical_gap_em: math_em(
                c.radical_display_style_vertical_gap(),
                upem,
            ),
            radical_extra_ascender_em: math_em(c.radical_extra_ascender(), upem),
            radical_kern_before_degree_em: math_em(c.radical_kern_before_degree(), upem),
            radical_kern_after_degree_em: math_em(c.radical_kern_after_degree(), upem),
            radical_degree_bottom_raise_percent: f64::from(c.radical_degree_bottom_raise_percent())
                / 100.0,
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
    let mut height: f64 = 0.0;
    let mut depth: f64 = 0.0;
    for g in run.glyphs {
        let (h, d) = font.glyph_ink_em(g.gid)?;
        height = height.max(h * scale);
        depth = depth.max(d * scale);
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
            height,
            depth,
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
    let mut carry_italic = 0.0;
    for child in children {
        let laid = layout_atom(font, c, child, style, scale)?;
        let space = match (prev_class, atom_class(child)) {
            (Some(l), Some(r)) => class_spacing_em(l, r) * scale,
            _ => 0.0,
        };
        let next_carry = match child {
            MathAtom::Symbol { .. } => trailing_italic_em(&laid),
            _ => 0.0,
        };
        let dx = out.metrics.width + space + carry_italic;
        append_shifted(&mut out, laid, dx, 0.0);
        prev_class = atom_class(child).or(prev_class);
        carry_italic = next_carry;
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
    let italic = trailing_italic_em(&out);
    let base_gid = trailing_gid(&out);
    if let Some(sup) = sup {
        let s = layout_atom(font, c, sup, script_style, ss)?;
        let dy = c.superscript_shift_up_em * scale;
        let kern = corner_kern(font, base_gid, MathKernCorner::TopRight, dy)
            + corner_kern(font, leading_gid(&s), MathKernCorner::TopLeft, dy);
        append_shifted(&mut out, s, base_w + italic + kern, dy);
    }
    if let Some(sub) = sub {
        let s = layout_atom(font, c, sub, script_style, ss)?;
        let dy = -c.subscript_shift_down_em * scale;
        let kern = corner_kern(font, base_gid, MathKernCorner::BottomRight, -dy)
            + corner_kern(font, leading_gid(&s), MathKernCorner::BottomLeft, -dy);
        append_shifted(&mut out, s, base_w + kern, dy);
    }
    if sup.is_some() || sub.is_some() {
        out.metrics.width = extent_width(&out) + c.space_after_script_em * scale;
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
    let (num_shift, den_shift) = if style == EstimateStyle::Display {
        (
            c.fraction_num_display_style_shift_up_em,
            c.fraction_den_display_style_shift_down_em,
        )
    } else {
        (c.fraction_num_shift_up_em, c.fraction_den_shift_down_em)
    };
    let num_y = axis + num_shift * scale;
    let den_y = axis - den_shift * scale;
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
    let gap = if style == EstimateStyle::Display {
        c.radical_display_style_vertical_gap_em
    } else {
        c.radical_vertical_gap_em
    } * scale;
    let extra = c.radical_extra_ascender_em * scale;
    let rule_y = body.metrics.height + gap;
    let mut out = PositionedMath {
        metrics: MathBox {
            width,
            height: rule_y + rule_th + extra,
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
        let before = c.radical_kern_before_degree_em * scale;
        let after = c.radical_kern_after_degree_em * scale;
        let raise = (rule_y + extra) * c.radical_degree_bottom_raise_percent;
        let idx_w = idx.metrics.width;
        append_shifted(&mut out, idx, before, raise);
        width = width.max(before + idx_w + after + surd_w + body_w);
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
    let min_h = c.delimited_sub_formula_min_height_em * scale;
    let target = (body_l.metrics.total_height() * stretch).max(min_h);
    let mut out = PositionedMath {
        metrics: MathBox {
            width: 0.0,
            height: body_l.metrics.height.max(target * 0.5),
            depth: body_l.metrics.depth.max(target * 0.5),
        },
        glyphs: Vec::new(),
        rules: Vec::new(),
    };
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
    if let Some(math) = face.tables().math {
        if let Some(var) = math.variants {
            if let Some(cons) = var.vertical_constructions.get(GlyphId(gid)) {
                for i in 0..cons.variants.len() {
                    if let Some(v) = cons.variants.get(i) {
                        let h = f64::from(v.advance_measurement) / upem;
                        if h * scale + 1e-12 >= target_em {
                            return single_stretchy_part(
                                font,
                                ch,
                                delim,
                                v.variant_glyph.0,
                                h * scale,
                                scale,
                            );
                        }
                    }
                }
                if let Some(assembly) = cons.assembly {
                    return assemble_vertical_delim(
                        font,
                        ch,
                        delim,
                        &assembly,
                        var.min_connector_overlap,
                        target_em,
                        scale,
                    );
                }
                return Err(LayoutError::Engine {
                    detail: format!("MATH glyph assembly missing for stretchy `{delim}`"),
                });
            }
        }
    }
    let adv = font.hor_advance_units(gid)?;
    let (ink_h, ink_d) = font.glyph_ink_em(gid)?;
    let h = (ink_h + ink_d) * scale;
    let g = ShapedGlyph {
        font_id: font.id.clone(),
        gid,
        ch,
        cluster_start: 0,
        cluster_end: delim.len(),
        advance_em: f64::from(adv) / upem,
        italic_correction_em: font.italic_correction_gid_em(gid),
    };
    Ok(positioned_delim_box(
        vec![PositionedMathGlyph {
            glyph: g,
            x_em: 0.0,
            y_em: 0.0,
            scale,
        }],
        h,
    ))
}

fn single_stretchy_part(
    font: &LoadedFont,
    ch: char,
    delim: &str,
    gid: u16,
    height_em: f64,
    scale: f64,
) -> Result<PositionedMath, LayoutError> {
    let upem = f64::from(font.units_per_em().max(1));
    let adv = font.hor_advance_units(gid)?;
    let g = ShapedGlyph {
        font_id: font.id.clone(),
        gid,
        ch,
        cluster_start: 0,
        cluster_end: delim.len(),
        advance_em: f64::from(adv) / upem,
        italic_correction_em: font.italic_correction_gid_em(gid),
    };
    Ok(positioned_delim_box(
        vec![PositionedMathGlyph {
            glyph: g,
            x_em: 0.0,
            y_em: 0.0,
            scale,
        }],
        height_em,
    ))
}

fn positioned_delim_box(glyphs: Vec<PositionedMathGlyph>, height_em: f64) -> PositionedMath {
    let width = glyphs
        .iter()
        .map(|g| g.glyph.advance_em * g.scale)
        .fold(0.0_f64, f64::max);
    PositionedMath {
        metrics: MathBox {
            width,
            height: height_em * 0.5,
            depth: height_em * 0.5,
        },
        glyphs,
        rules: Vec::new(),
    }
}

fn assemble_vertical_delim(
    font: &LoadedFont,
    ch: char,
    delim: &str,
    assembly: &ttf_parser::math::GlyphAssembly<'_>,
    min_overlap: u16,
    target_em: f64,
    scale: f64,
) -> Result<PositionedMath, LayoutError> {
    let upem = f64::from(font.units_per_em().max(1));
    let overlap_em = f64::from(min_overlap) / upem;
    let n = assembly.parts.len();
    if n == 0 {
        return Err(LayoutError::Engine {
            detail: format!("MATH glyph assembly empty for stretchy `{delim}`"),
        });
    }
    let mut parts = Vec::with_capacity(n as usize);
    for i in 0..n {
        let p = assembly.parts.get(i).ok_or_else(|| LayoutError::Engine {
            detail: format!("MATH glyph assembly part {i} missing for `{delim}`"),
        })?;
        parts.push(p);
    }
    let has_extender = parts.iter().any(|p| p.part_flags.extender());
    let mut ext_repeats: u32 = 1;
    let height = loop {
        let h = assembly_stack_height(&parts, ext_repeats, overlap_em, upem);
        if h + 1e-9 >= target_em {
            break h;
        }
        if !has_extender || ext_repeats >= 64 {
            return Err(LayoutError::Engine {
                detail: format!(
                    "MATH glyph assembly cannot reach {target_em} em for stretchy `{delim}`"
                ),
            });
        }
        ext_repeats += 1;
    };
    let mut glyphs = Vec::new();
    let mut y = -height * 0.5 * scale;
    let mut first = true;
    for p in &parts {
        let copies = if p.part_flags.extender() {
            ext_repeats
        } else {
            1
        };
        let adv_em = f64::from(p.full_advance) / upem;
        for _ in 0..copies {
            if !first {
                y -= overlap_em * scale;
            }
            first = false;
            let gid = p.glyph_id.0;
            let hor = font.hor_advance_units(gid)?;
            glyphs.push(PositionedMathGlyph {
                glyph: ShapedGlyph {
                    font_id: font.id.clone(),
                    gid,
                    ch,
                    cluster_start: 0,
                    cluster_end: delim.len(),
                    advance_em: f64::from(hor) / upem,
                    italic_correction_em: font.italic_correction_gid_em(gid),
                },
                x_em: 0.0,
                y_em: y + adv_em * 0.5 * scale,
                scale,
            });
            y += adv_em * scale;
        }
    }
    if let Some(last) = glyphs.last_mut() {
        let asm_ic = f64::from(assembly.italics_correction.value) / upem;
        if asm_ic.abs() > 1e-12 {
            last.glyph.italic_correction_em = asm_ic;
        }
    }
    Ok(positioned_delim_box(glyphs, height * scale))
}

fn assembly_stack_height(
    parts: &[ttf_parser::math::GlyphPart],
    ext_repeats: u32,
    overlap_em: f64,
    upem: f64,
) -> f64 {
    let mut count = 0u32;
    let mut sum_em = 0.0;
    for p in parts {
        let copies = if p.part_flags.extender() {
            ext_repeats
        } else {
            1
        };
        let adv_em = f64::from(p.full_advance) / upem;
        for _ in 0..copies {
            if count > 0 {
                sum_em -= overlap_em;
            }
            sum_em += adv_em;
            count += 1;
        }
    }
    sum_em
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
            let gap = c.underbar_vertical_gap_em * scale;
            let th = c.underbar_rule_thickness_em * scale;
            let extra = c.underbar_extra_descender_em * scale;
            let y = -out.metrics.depth - gap;
            out.rules.push(MathRule {
                x0_em: 0.0,
                y0_em: y,
                x1_em: out.metrics.width.max(th),
                y1_em: y,
                thickness_em: th,
            });
            out.metrics.depth += gap + th + extra;
            return Ok(out);
        }
        MathAccentKind::Overline | MathAccentKind::Bar => {
            let gap = c.overbar_vertical_gap_em * scale;
            let th = c.overbar_rule_thickness_em * scale;
            let extra = c.overbar_extra_ascender_em * scale;
            let y = out.metrics.height + gap;
            out.rules.push(MathRule {
                x0_em: 0.0,
                y0_em: y,
                x1_em: out.metrics.width.max(th),
                y1_em: y,
                thickness_em: th,
            });
            out.metrics.height += gap + th + extra;
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
    let acc = layout_symbol(font, mark, scale * c.script_percent_scale_down)?;
    let dy = out.metrics.height.max(c.accent_base_height_em * scale);
    let acc_x = (out.metrics.width - acc.metrics.width).max(0.0) * 0.5;
    append_shifted(&mut out, acc, acc_x, dy);
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
            c.display_operator_min_height_em * scale,
            scale,
        )?
    } else {
        layout_symbol(font, operator, scale)?
    };
    let mut out = op;
    let ss = script_scale(c, style) * scale;
    let st = EstimateStyle::Text;
    let italic = trailing_italic_em(&out);
    let op_gid = trailing_gid(&out);
    let op_w = out.metrics.width;
    if let Some(u) = upper {
        let u = layout_atom(font, c, u, st, ss)?;
        let rise = (out.metrics.height + u.metrics.depth + c.upper_limit_gap_min_em * scale)
            .max(c.upper_limit_baseline_rise_min_em * scale);
        let kern = corner_kern(font, op_gid, MathKernCorner::TopRight, rise)
            + corner_kern(font, leading_gid(&u), MathKernCorner::TopLeft, rise);
        let dx = (op_w - u.metrics.width).max(0.0) * 0.5 + italic * 0.5 + kern;
        append_shifted(&mut out, u, dx, rise);
    }
    if let Some(l) = lower {
        let l = layout_atom(font, c, l, st, ss)?;
        let drop = (out.metrics.depth + l.metrics.height + c.lower_limit_gap_min_em * scale)
            .max(c.lower_limit_baseline_drop_min_em * scale);
        let kern = corner_kern(font, op_gid, MathKernCorner::BottomRight, drop)
            + corner_kern(font, leading_gid(&l), MathKernCorner::BottomLeft, drop);
        let dx = (op_w - l.metrics.width).max(0.0) * 0.5 - italic * 0.5 + kern;
        append_shifted(&mut out, l, dx, -drop);
    }
    if let Some(b) = body {
        let b = layout_atom(font, c, b, style, scale)?;
        let bx = out.metrics.width + c.space_after_script_em * scale;
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
    let gap = if as_matrix || style == EstimateStyle::Display {
        c.stack_display_style_gap_min_em
    } else {
        c.stack_gap_min_em
    } * scale;
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
            .map(|cell| cell.metrics.height)
            .fold(0.0, f64::max);
        let row_d = row
            .iter()
            .map(|cell| cell.metrics.depth)
            .fold(0.0, f64::max);
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

fn trailing_italic_em(m: &PositionedMath) -> f64 {
    m.glyphs
        .last()
        .map(|g| g.glyph.italic_correction_em * g.scale)
        .unwrap_or(0.0)
}

fn trailing_gid(m: &PositionedMath) -> Option<u16> {
    m.glyphs.last().map(|g| g.glyph.gid)
}

fn leading_gid(m: &PositionedMath) -> Option<u16> {
    m.glyphs.first().map(|g| g.glyph.gid)
}

fn corner_kern(font: &LoadedFont, gid: Option<u16>, corner: MathKernCorner, height_em: f64) -> f64 {
    gid.map(|id| font.math_kern_em(id, corner, height_em))
        .unwrap_or(0.0)
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
