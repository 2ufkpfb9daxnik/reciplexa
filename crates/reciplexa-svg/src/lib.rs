//! SVG export from a scene [`Document`] via flattened world shapes.
//!
//! Page Y is up in scene space; SVG Y grows down, so we flip within each page
//! viewBox of `0 0 width_mm height_mm`.

#![forbid(unsafe_code)]

use std::fmt::Write as _;
use std::io::{self, Write};

use reciplexa_scene::{Color, Document};
use reciplexa_view::{flatten_page, WorldShape};

/// Write one SVG document: pages are stacked vertically with a small gap.
pub fn write_document(doc: &Document, mut out: impl Write) -> io::Result<()> {
    let svg = document_to_svg(doc);
    out.write_all(svg.as_bytes())
}

/// Render `doc` to an SVG string (millimeter units).
pub fn document_to_svg(doc: &Document) -> String {
    const GAP_MM: f64 = 10.0;
    let mut total_w = 0.0_f64;
    let mut total_h = 0.0_f64;
    let mut page_offsets = Vec::new();
    for (i, page) in doc.pages.iter().enumerate() {
        let w = page.paper.width_mm;
        let h = page.paper.height_mm;
        total_w = total_w.max(w);
        if i > 0 {
            total_h += GAP_MM;
        }
        page_offsets.push(total_h);
        total_h += h;
    }
    if doc.pages.is_empty() {
        total_w = 210.0;
        total_h = 297.0;
    }

    let mut s = String::new();
    let _ = writeln!(s, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    let _ = writeln!(
        s,
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{total_w}mm" height="{total_h}mm" viewBox="0 0 {total_w} {total_h}">"#
    );
    let _ = writeln!(s, r#"  <title>reciplexa</title>"#);

    for (i, page) in doc.pages.iter().enumerate() {
        let Some((_, shapes)) = flatten_page(doc, i) else {
            continue;
        };
        let w = page.paper.width_mm;
        let h = page.paper.height_mm;
        let oy = page_offsets[i];
        let _ = writeln!(
            s,
            r#"  <g id="page-{}" transform="translate(0 {})">"#,
            i + 1,
            fmt_num(oy)
        );
        let _ = writeln!(
            s,
            r##"    <rect x="0" y="0" width="{}" height="{}" fill="#ffffff" stroke="#cccccc" stroke-width="0.2"/>"##,
            fmt_num(w),
            fmt_num(h)
        );
        // Flip Y: scene y-up → SVG y-down within the paper.
        let _ = writeln!(
            s,
            r#"    <g transform="matrix(1 0 0 -1 0 {})">"#,
            fmt_num(h)
        );
        for shape in &shapes {
            write_shape(&mut s, shape);
        }
        let _ = writeln!(s, "    </g>");
        let _ = writeln!(s, "  </g>");
    }

    let _ = writeln!(s, "</svg>");
    s
}

fn write_shape(s: &mut String, shape: &WorldShape) {
    match shape {
        WorldShape::Circle(c) => {
            let fill = css_color(c.color, c.alpha);
            match c.stroke_width_mm {
                None => {
                    let _ = writeln!(
                        s,
                        r#"      <circle cx="{}" cy="{}" r="{}" fill="{fill}"/>"#,
                        fmt_num(c.x_mm),
                        fmt_num(c.y_mm),
                        fmt_num(c.radius_mm),
                    );
                }
                Some(w) => {
                    let _ = writeln!(
                        s,
                        r#"      <circle cx="{}" cy="{}" r="{}" fill="none" stroke="{fill}" stroke-width="{}"/>"#,
                        fmt_num(c.x_mm),
                        fmt_num(c.y_mm),
                        fmt_num(c.radius_mm),
                        fmt_num(w),
                    );
                }
            }
        }
        WorldShape::Polygon(p) => {
            if p.points_mm.len() < 3 {
                return;
            }
            let pts = points_attr(&p.points_mm);
            let fill = css_color(p.color, p.alpha);
            match p.stroke_width_mm {
                None => {
                    let _ = writeln!(s, r#"      <polygon points="{pts}" fill="{fill}"/>"#);
                }
                Some(w) => {
                    let _ = writeln!(
                        s,
                        r#"      <polygon points="{pts}" fill="none" stroke="{fill}" stroke-width="{}"/>"#,
                        fmt_num(w),
                    );
                }
            }
        }
        WorldShape::Path(p) => {
            if p.points_mm.len() < 2 {
                return;
            }
            let d = path_d(&p.points_mm, p.closed);
            let stroke = css_color(p.stroke, p.alpha);
            let _ = writeln!(
                s,
                r#"      <path d="{d}" fill="none" stroke="{stroke}" stroke-width="{}"/>"#,
                fmt_num(p.width_mm),
            );
        }
        WorldShape::Text(t) => {
            // Inside Y-flip group; counter-flip text so glyphs stay upright.
            let fill = css_color(t.fill, t.alpha);
            let escape = xml_escape(&t.content);
            let angle = -t.rotation_deg; // undo page CCW after Y flip
            let _ = writeln!(
                s,
                r#"      <text x="{}" y="{}" font-size="{}" fill="{fill}" transform="translate({} {}) scale(1 -1) rotate({})" font-family="sans-serif">{}</text>"#,
                fmt_num(0.0),
                fmt_num(0.0),
                fmt_num(t.size_mm),
                fmt_num(t.x_mm),
                fmt_num(t.y_mm),
                fmt_num(angle),
                escape,
            );
        }
        WorldShape::Image(img) => {
            // Map image quad to a parallelogram via SVG transform if axis-aligned;
            // otherwise fall back to a stroked outline (full texture needs link path).
            let [(x0, y0), (x1, y1), (x2, y2), (x3, y3)] = img.corners_mm;
            let min_x = x0.min(x1).min(x2).min(x3);
            let max_x = x0.max(x1).max(x2).max(x3);
            let min_y = y0.min(y1).min(y2).min(y3);
            let max_y = y0.max(y1).max(y2).max(y3);
            let w = (max_x - min_x).max(0.1);
            let h = (max_y - min_y).max(0.1);
            let href = xml_escape(&img.path);
            let opacity = if (img.alpha - 1.0).abs() < 1e-9 {
                String::new()
            } else {
                format!(r#" opacity="{}""#, fmt_num(img.alpha))
            };
            let _ = writeln!(
                s,
                r#"      <image href="{href}" x="{}" y="{}" width="{}" height="{}"{opacity} preserveAspectRatio="none"/>"#,
                fmt_num(min_x),
                fmt_num(min_y),
                fmt_num(w),
                fmt_num(h),
            );
        }
    }
}

fn points_attr(pts: &[(f64, f64)]) -> String {
    pts.iter()
        .map(|&(x, y)| format!("{} {}", fmt_num(x), fmt_num(y)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn path_d(pts: &[(f64, f64)], closed: bool) -> String {
    let mut d = String::new();
    for (i, &(x, y)) in pts.iter().enumerate() {
        if i == 0 {
            let _ = write!(d, "M {} {}", fmt_num(x), fmt_num(y));
        } else {
            let _ = write!(d, " L {} {}", fmt_num(x), fmt_num(y));
        }
    }
    if closed {
        d.push_str(" Z");
    }
    d
}

fn css_color(c: Color, alpha: f64) -> String {
    let r = (c.r * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = (c.g * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = (c.b * 255.0).round().clamp(0.0, 255.0) as u8;
    if (alpha - 1.0).abs() < 1e-9 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!("rgba({r},{g},{b},{})", fmt_num(alpha.clamp(0.0, 1.0)))
    }
}

fn fmt_num(v: f64) -> String {
    let rounded = (v * 1000.0).round() / 1000.0;
    if rounded == 0.0 {
        "0".into()
    } else {
        let mut s = format!("{rounded}");
        if s.contains('.') {
            while s.ends_with('0') {
                s.pop();
            }
            if s.ends_with('.') {
                s.pop();
            }
        }
        s
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_scene::{Circle, Color, Document, Page, PaperSize, Shape};

    #[test]
    fn black_circle_svg_has_root_and_circle() {
        let doc = Document {
            pages: vec![Page {
                paper: PaperSize::a4(),
                shapes: vec![Shape::Circle(Circle {
                    x_mm: 105.0,
                    y_mm: 148.5,
                    radius_mm: 20.0,
                    fill: Color::BLACK,
                })],
            }],
        };
        let svg = document_to_svg(&doc);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("<circle"));
        assert!(svg.contains("cx=\"105\""));
        assert!(svg.contains("r=\"20\""));
        // Stable golden substring (render IR style): one page group id.
        assert!(svg.contains("id=\"page-1\""));
    }

    #[test]
    fn multipage_stacks_vertically() {
        let page = Page {
            paper: PaperSize::a4(),
            shapes: vec![],
        };
        let doc = Document {
            pages: vec![page.clone(), page],
        };
        let svg = document_to_svg(&doc);
        assert!(svg.contains("id=\"page-1\""));
        assert!(svg.contains("id=\"page-2\""));
        assert!(svg.contains("translate(0 307)")); // 297 + 10 gap
    }
}
