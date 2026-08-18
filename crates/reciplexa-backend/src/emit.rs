//! SVG emission strictly from planning IR.

use std::fmt::Write as _;

use reciplexa_scene::Color;
use reciplexa_visual_ir::{RenderDocument, RenderNode};

use crate::plan::{BackendPlan, PlannedNode, Representation};

/// Emission refused to invent geometry outside the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmitError {
    PlanNodeMismatch {
        render_id: u64,
        representation: String,
    },
    MissingRenderNode {
        render_id: u64,
    },
}

/// Emit SVG from a validated plan — no unplanned fallbacks.
pub fn emit_svg_from_plan(
    plan: &BackendPlan,
    render: &RenderDocument,
) -> Result<String, EmitError> {
    const GAP_MM: f64 = 10.0;
    let mut total_w = 0.0_f64;
    let mut total_h = 0.0_f64;
    let mut page_offsets = Vec::new();
    for (i, page) in render.pages.iter().enumerate() {
        total_w = total_w.max(page.width_mm);
        if i > 0 {
            total_h += GAP_MM;
        }
        page_offsets.push(total_h);
        total_h += page.height_mm;
    }
    if render.pages.is_empty() {
        total_w = 210.0;
        total_h = 297.0;
    }

    let mut s = String::new();
    let _ = writeln!(s, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    let _ = writeln!(
        s,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{total_w}mm" height="{total_h}mm" viewBox="0 0 {total_w} {total_h}">"#
    );
    let _ = writeln!(s, r#"  <title>reciplexa</title>"#);

    for (page_index, page) in render.pages.iter().enumerate() {
        let oy = page_offsets[page_index];
        let _ = writeln!(
            s,
            r#"  <g id="page-{}" transform="translate(0 {})">"#,
            page_index + 1,
            fmt_num(oy)
        );
        let _ = writeln!(
            s,
            r##"    <rect x="0" y="0" width="{}" height="{}" fill="#ffffff" stroke="#cccccc" stroke-width="0.2"/>"##,
            fmt_num(page.width_mm),
            fmt_num(page.height_mm)
        );
        let _ = writeln!(
            s,
            r#"    <g transform="matrix(1 0 0 -1 0 {})">"#,
            fmt_num(page.height_mm)
        );
        let planned: Vec<&PlannedNode> = plan
            .nodes
            .iter()
            .filter(|n| n.page_index == page_index)
            .collect();
        for pn in planned {
            let node = page.nodes.iter().find(|n| n.id() == pn.render_id).ok_or(
                EmitError::MissingRenderNode {
                    render_id: pn.render_id.0,
                },
            )?;
            write_planned_node(&mut s, pn, node)?;
        }
        let _ = writeln!(s, "    </g>");
        let _ = writeln!(s, "  </g>");
    }
    let _ = writeln!(s, "</svg>");
    Ok(s)
}

fn write_planned_node(
    s: &mut String,
    planned: &PlannedNode,
    node: &RenderNode,
) -> Result<(), EmitError> {
    let id = &planned.artifact_element_id;
    match (planned.representation, node) {
        (
            Representation::SvgRect,
            RenderNode::Rect {
                x_mm,
                y_mm,
                width_mm,
                height_mm,
                fill,
                stroke_width_mm,
                alpha,
                ..
            },
        ) => {
            if let Some(sw) = stroke_width_mm {
                let _ = writeln!(
                    s,
                    r#"      <rect id="{id}" x="{}" y="{}" width="{}" height="{}" fill="none" stroke="{}" stroke-width="{}" opacity="{}"/>"#,
                    fmt_num(*x_mm),
                    fmt_num(*y_mm),
                    fmt_num(*width_mm),
                    fmt_num(*height_mm),
                    color_hex(*fill),
                    fmt_num(*sw),
                    fmt_num(*alpha)
                );
            } else {
                let _ = writeln!(
                    s,
                    r#"      <rect id="{id}" x="{}" y="{}" width="{}" height="{}" {}/>"#,
                    fmt_num(*x_mm),
                    fmt_num(*y_mm),
                    fmt_num(*width_mm),
                    fmt_num(*height_mm),
                    color_attr(*fill, *alpha)
                );
            }
            Ok(())
        }
        (
            Representation::SvgCircle,
            RenderNode::Circle {
                x_mm,
                y_mm,
                radius_mm,
                fill,
                stroke_width_mm,
                alpha,
                ..
            },
        ) => {
            match stroke_width_mm {
                None => {
                    let _ = writeln!(
                        s,
                        r#"      <circle id="{id}" cx="{}" cy="{}" r="{}" {}/>"#,
                        fmt_num(*x_mm),
                        fmt_num(*y_mm),
                        fmt_num(*radius_mm),
                        color_attr(*fill, *alpha)
                    );
                }
                Some(w) => {
                    let _ = writeln!(
                        s,
                        r#"      <circle id="{id}" cx="{}" cy="{}" r="{}" fill="none" stroke="{}" stroke-width="{}" opacity="{}"/>"#,
                        fmt_num(*x_mm),
                        fmt_num(*y_mm),
                        fmt_num(*radius_mm),
                        color_hex(*fill),
                        fmt_num(*w),
                        fmt_num(*alpha)
                    );
                }
            }
            Ok(())
        }
        (
            Representation::SvgPolygon,
            RenderNode::Polygon {
                points_mm,
                fill,
                stroke_width_mm,
                alpha,
                ..
            },
        ) => {
            let pts: String = points_mm
                .iter()
                .map(|(x, y)| format!("{},{}", fmt_num(*x), fmt_num(*y)))
                .collect::<Vec<_>>()
                .join(" ");
            match stroke_width_mm {
                None => {
                    let _ = writeln!(
                        s,
                        r#"      <polygon id="{id}" points="{pts}" {}/>"#,
                        color_attr(*fill, *alpha)
                    );
                }
                Some(w) => {
                    let _ = writeln!(
                        s,
                        r#"      <polygon id="{id}" points="{pts}" fill="none" stroke="{}" stroke-width="{}" opacity="{}"/>"#,
                        color_hex(*fill),
                        fmt_num(*w),
                        fmt_num(*alpha)
                    );
                }
            }
            Ok(())
        }
        (
            Representation::SvgText,
            RenderNode::Text {
                x_mm,
                y_mm,
                size_mm,
                rotation_deg,
                content,
                fill,
                alpha,
                ..
            },
        ) => {
            // Inside Y-flip group; counter-flip text so glyphs stay upright.
            let font = planned.font_family.as_deref().unwrap_or("sans-serif");
            let escape = xml_escape(content);
            let angle = -*rotation_deg;
            let _ = writeln!(
                s,
                r#"      <text id="{id}" x="0" y="0" font-size="{}" font-family="{font}" {} transform="translate({} {}) scale(1 -1) rotate({})">{}</text>"#,
                fmt_num(*size_mm),
                color_attr(*fill, *alpha),
                fmt_num(*x_mm),
                fmt_num(*y_mm),
                fmt_num(angle),
                escape
            );
            Ok(())
        }
        (
            Representation::SvgText,
            RenderNode::GlyphRun {
                x_mm,
                y_mm,
                size_mm,
                content,
                fill,
                alpha,
                font_label,
                ..
            },
        ) => {
            let font = if font_label.is_empty() {
                planned.font_family.as_deref().unwrap_or("sans-serif")
            } else {
                font_label.as_str()
            };
            let escape = xml_escape(content);
            let _ = writeln!(
                s,
                r#"      <text id="{id}" x="0" y="0" font-size="{}" font-family="{font}" {} transform="translate({} {}) scale(1 -1) rotate(0)">{}</text>"#,
                fmt_num(*size_mm),
                color_attr(*fill, *alpha),
                fmt_num(*x_mm),
                fmt_num(*y_mm),
                escape
            );
            Ok(())
        }
        (
            Representation::SvgPath,
            RenderNode::Path {
                points_mm,
                stroke,
                width_mm,
                closed,
                alpha,
                ..
            },
        ) => {
            if points_mm.is_empty() {
                return Ok(());
            }
            let mut d = format!("M {} {}", fmt_num(points_mm[0].0), fmt_num(points_mm[0].1));
            for (x, y) in &points_mm[1..] {
                d.push_str(&format!(" L {} {}", fmt_num(*x), fmt_num(*y)));
            }
            if *closed {
                d.push('Z');
            }
            let _ = writeln!(
                s,
                r#"      <path id="{id}" d="{d}" fill="none" stroke="{}" stroke-width="{}" opacity="{}"/>"#,
                color_hex(*stroke),
                fmt_num(*width_mm),
                fmt_num(*alpha)
            );
            Ok(())
        }
        (
            Representation::SvgImage,
            RenderNode::Image {
                path,
                corners_mm,
                alpha,
                ..
            },
        ) => {
            let min_x = corners_mm.iter().map(|c| c.0).fold(f64::INFINITY, f64::min);
            let max_x = corners_mm
                .iter()
                .map(|c| c.0)
                .fold(f64::NEG_INFINITY, f64::max);
            let min_y = corners_mm.iter().map(|c| c.1).fold(f64::INFINITY, f64::min);
            let max_y = corners_mm
                .iter()
                .map(|c| c.1)
                .fold(f64::NEG_INFINITY, f64::max);
            let href = xml_escape(path);
            let _ = writeln!(
                s,
                r#"      <image id="{id}" href="{href}" opacity="{}" x="{}" y="{}" width="{}" height="{}" preserveAspectRatio="none"/>"#,
                fmt_num(*alpha),
                fmt_num(min_x),
                fmt_num(min_y),
                fmt_num((max_x - min_x).max(0.1)),
                fmt_num((max_y - min_y).max(0.1))
            );
            Ok(())
        }
        (repr, _) => Err(EmitError::PlanNodeMismatch {
            render_id: planned.render_id.0,
            representation: format!("{repr:?}"),
        }),
    }
}

fn fmt_num(n: f64) -> String {
    if (n - n.round()).abs() < 1e-9 {
        format!("{}", n.round() as i64)
    } else {
        format!("{n:.3}")
    }
}

fn color_hex(c: Color) -> String {
    let r = (c.r.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (c.g.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (c.b.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn color_attr(c: Color, alpha: f64) -> String {
    format!(
        r#"fill="{}" opacity="{}""#,
        color_hex(c),
        fmt_num(alpha.clamp(0.0, 1.0))
    )
}

fn xml_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".into(),
            '<' => "&lt;".into(),
            '>' => "&gt;".into(),
            '"' => "&quot;".into(),
            '\'' => "&apos;".into(),
            c => c.to_string(),
        })
        .collect()
}
