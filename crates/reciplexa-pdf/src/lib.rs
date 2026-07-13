//! Minimal PDF emitter for [`reciplexa_scene::Document`].
//!
//! Hand-rolled PDF-1.4: filled circles/rects, stroked lines, and Helvetica
//! text (ASCII / WinAnsi). Japanese text needs a later font package (JLReq).

#![forbid(unsafe_code)]

use reciplexa_scene::{Affine, Color, Document, Page, Shape};

/// Errors while building a PDF.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfError {
    EmptyDocument,
    InvalidPage(String),
    InvalidShape(String),
    Write(String),
}

/// Render a scene document to PDF bytes.
pub fn document_to_pdf(doc: &Document) -> Result<Vec<u8>, PdfError> {
    if doc.pages.is_empty() {
        return Err(PdfError::EmptyDocument);
    }

    let mut page_contents = Vec::with_capacity(doc.pages.len());
    let mut page_sizes = Vec::with_capacity(doc.pages.len());

    for (i, page) in doc.pages.iter().enumerate() {
        if !page.paper.is_positive() {
            return Err(PdfError::InvalidPage(format!(
                "page {i}: non-positive paper size"
            )));
        }
        let w_pt = mm_to_pt(page.paper.width_mm);
        let h_pt = mm_to_pt(page.paper.height_mm);
        page_sizes.push((w_pt, h_pt));
        page_contents.push(render_page_content(page, i)?);
    }

    Ok(assemble_pdf(&page_sizes, &page_contents))
}

pub fn write_document(doc: &Document, mut w: impl std::io::Write) -> Result<(), PdfError> {
    let bytes = document_to_pdf(doc)?;
    w.write_all(&bytes)
        .map_err(|e| PdfError::Write(e.to_string()))?;
    Ok(())
}

fn render_page_content(page: &Page, index: usize) -> Result<String, PdfError> {
    let mut ops = String::new();
    for (si, shape) in page.shapes.iter().enumerate() {
        ops.push_str(&render_shape(shape, &format!("page {index} shape {si}"))?);
    }
    Ok(ops)
}

fn render_shape(shape: &Shape, ctx: &str) -> Result<String, PdfError> {
    match shape {
        Shape::Circle(c) => {
            if !c.is_drawable() {
                return Err(PdfError::InvalidShape(format!(
                    "{ctx}: circle not drawable"
                )));
            }
            Ok(circle_path_ops(c.x_mm, c.y_mm, c.radius_mm, c.fill))
        }
        Shape::Rect(r) => {
            if !r.is_drawable() {
                return Err(PdfError::InvalidShape(format!("{ctx}: rect not drawable")));
            }
            Ok(rect_path_ops(
                r.x_mm,
                r.y_mm,
                r.width_mm,
                r.height_mm,
                r.fill,
            ))
        }
        Shape::Ellipse(e) => {
            if !e.is_drawable() {
                return Err(PdfError::InvalidShape(format!(
                    "{ctx}: ellipse not drawable"
                )));
            }
            Ok(ellipse_path_ops(e.x_mm, e.y_mm, e.rx_mm, e.ry_mm, e.fill))
        }
        Shape::Ring(r) => {
            if !r.is_drawable() {
                return Err(PdfError::InvalidShape(format!("{ctx}: ring not drawable")));
            }
            Ok(ring_path_ops(
                r.x_mm,
                r.y_mm,
                r.radius_mm,
                r.width_mm,
                r.stroke,
            ))
        }
        Shape::Frame(f) => {
            if !f.is_drawable() {
                return Err(PdfError::InvalidShape(format!("{ctx}: frame not drawable")));
            }
            Ok(frame_path_ops(
                f.x_mm,
                f.y_mm,
                f.width_mm,
                f.height_mm,
                f.stroke_width_mm,
                f.stroke,
            ))
        }
        Shape::Text(t) => {
            if !t.is_drawable() {
                return Err(PdfError::InvalidShape(format!("{ctx}: text not drawable")));
            }
            text_ops(t.x_mm, t.y_mm, t.size_mm, &t.content, t.fill)
        }
        Shape::Line(l) => {
            if !l.is_drawable() {
                return Err(PdfError::InvalidShape(format!("{ctx}: line not drawable")));
            }
            Ok(line_ops(
                l.x1_mm, l.y1_mm, l.x2_mm, l.y2_mm, l.stroke, l.width_mm,
            ))
        }
        Shape::Group {
            transform,
            children,
        } => {
            if !transform.is_finite() {
                return Err(PdfError::InvalidShape(format!(
                    "{ctx}: non-finite transform"
                )));
            }
            let mut ops = String::from("q\n");
            ops.push_str(&affine_cm_ops(*transform));
            for (i, child) in children.iter().enumerate() {
                ops.push_str(&render_shape(child, &format!("{ctx}/{i}"))?);
            }
            ops.push_str("Q\n");
            Ok(ops)
        }
    }
}

fn affine_cm_ops(t: Affine) -> String {
    let s = 72.0 / 25.4;
    format!(
        "{:.6} {:.6} {:.6} {:.6} {:.6} {:.6} cm\n",
        t.a,
        t.b,
        t.c,
        t.d,
        t.e * s,
        t.f * s
    )
}

fn circle_path_ops(x_mm: f64, y_mm: f64, r_mm: f64, fill: Color) -> String {
    let k = 0.552_284_749_8;
    let cx = mm_to_pt(x_mm);
    let cy = mm_to_pt(y_mm);
    let r = mm_to_pt(r_mm);
    let kr = r * k;

    format!(
        "{r_col:.4} {g_col:.4} {b_col:.4} rg\n\
         {x0:.4} {y0:.4} m\n\
         {x1:.4} {y1:.4} {x2:.4} {y2:.4} {x3:.4} {y3:.4} c\n\
         {x4:.4} {y4:.4} {x5:.4} {y5:.4} {x6:.4} {y6:.4} c\n\
         {x7:.4} {y7:.4} {x8:.4} {y8:.4} {x9:.4} {y9:.4} c\n\
         {x10:.4} {y10:.4} {x11:.4} {y11:.4} {x12:.4} {y12:.4} c\n\
         f\n",
        r_col = fill.r,
        g_col = fill.g,
        b_col = fill.b,
        x0 = cx + r,
        y0 = cy,
        x1 = cx + r,
        y1 = cy + kr,
        x2 = cx + kr,
        y2 = cy + r,
        x3 = cx,
        y3 = cy + r,
        x4 = cx - kr,
        y4 = cy + r,
        x5 = cx - r,
        y5 = cy + kr,
        x6 = cx - r,
        y6 = cy,
        x7 = cx - r,
        y7 = cy - kr,
        x8 = cx - kr,
        y8 = cy - r,
        x9 = cx,
        y9 = cy - r,
        x10 = cx + kr,
        y10 = cy - r,
        x11 = cx + r,
        y11 = cy - kr,
        x12 = cx + r,
        y12 = cy,
    )
}

fn rect_path_ops(x_mm: f64, y_mm: f64, w_mm: f64, h_mm: f64, fill: Color) -> String {
    let x = mm_to_pt(x_mm);
    let y = mm_to_pt(y_mm);
    let w = mm_to_pt(w_mm);
    let h = mm_to_pt(h_mm);
    format!(
        "{r:.4} {g:.4} {b:.4} rg\n{x:.4} {y:.4} {w:.4} {h:.4} re\nf\n",
        r = fill.r,
        g = fill.g,
        b = fill.b,
    )
}

fn ellipse_path_ops(x_mm: f64, y_mm: f64, rx_mm: f64, ry_mm: f64, fill: Color) -> String {
    let k = 0.552_284_749_8;
    let cx = mm_to_pt(x_mm);
    let cy = mm_to_pt(y_mm);
    let rx = mm_to_pt(rx_mm);
    let ry = mm_to_pt(ry_mm);
    let kx = rx * k;
    let ky = ry * k;
    format!(
        "{r_col:.4} {g_col:.4} {b_col:.4} rg\n\
         {x0:.4} {y0:.4} m\n\
         {x1:.4} {y1:.4} {x2:.4} {y2:.4} {x3:.4} {y3:.4} c\n\
         {x4:.4} {y4:.4} {x5:.4} {y5:.4} {x6:.4} {y6:.4} c\n\
         {x7:.4} {y7:.4} {x8:.4} {y8:.4} {x9:.4} {y9:.4} c\n\
         {x10:.4} {y10:.4} {x11:.4} {y11:.4} {x12:.4} {y12:.4} c\n\
         f\n",
        r_col = fill.r,
        g_col = fill.g,
        b_col = fill.b,
        x0 = cx + rx,
        y0 = cy,
        x1 = cx + rx,
        y1 = cy + ky,
        x2 = cx + kx,
        y2 = cy + ry,
        x3 = cx,
        y3 = cy + ry,
        x4 = cx - kx,
        y4 = cy + ry,
        x5 = cx - rx,
        y5 = cy + ky,
        x6 = cx - rx,
        y6 = cy,
        x7 = cx - rx,
        y7 = cy - ky,
        x8 = cx - kx,
        y8 = cy - ry,
        x9 = cx,
        y9 = cy - ry,
        x10 = cx + kx,
        y10 = cy - ry,
        x11 = cx + rx,
        y11 = cy - ky,
        x12 = cx + rx,
        y12 = cy,
    )
}

fn ring_path_ops(x_mm: f64, y_mm: f64, r_mm: f64, width_mm: f64, stroke: Color) -> String {
    // Same Bezier circle as fill, but stroke with `S`.
    let body = circle_path_ops(x_mm, y_mm, r_mm, stroke);
    let stroked = body.replace(" rg\n", " RG\n").replace("\nf\n", "\nS\n");
    format!(
        "{w:.4} w\n{stroked}",
        w = mm_to_pt(width_mm),
        stroked = stroked
    )
}

fn frame_path_ops(
    x_mm: f64,
    y_mm: f64,
    w_mm: f64,
    h_mm: f64,
    stroke_width_mm: f64,
    stroke: Color,
) -> String {
    let x = mm_to_pt(x_mm);
    let y = mm_to_pt(y_mm);
    let w = mm_to_pt(w_mm);
    let h = mm_to_pt(h_mm);
    format!(
        "{r:.4} {g:.4} {b:.4} RG\n{sw:.4} w\n{x:.4} {y:.4} {w:.4} {h:.4} re\nS\n",
        r = stroke.r,
        g = stroke.g,
        b = stroke.b,
        sw = mm_to_pt(stroke_width_mm),
    )
}

fn line_ops(x1: f64, y1: f64, x2: f64, y2: f64, stroke: Color, width_mm: f64) -> String {
    format!(
        "{r:.4} {g:.4} {b:.4} RG\n{w:.4} w\n{x1:.4} {y1:.4} m\n{x2:.4} {y2:.4} l\nS\n",
        r = stroke.r,
        g = stroke.g,
        b = stroke.b,
        w = mm_to_pt(width_mm),
        x1 = mm_to_pt(x1),
        y1 = mm_to_pt(y1),
        x2 = mm_to_pt(x2),
        y2 = mm_to_pt(y2),
    )
}

fn text_ops(
    x_mm: f64,
    y_mm: f64,
    size_mm: f64,
    content: &str,
    fill: Color,
) -> Result<String, PdfError> {
    let escaped = pdf_escape_text(content)?;
    let size_pt = mm_to_pt(size_mm);
    Ok(format!(
        "BT\n/F1 {size:.4} Tf\n{r:.4} {g:.4} {b:.4} rg\n{x:.4} {y:.4} Td\n({escaped}) Tj\nET\n",
        size = size_pt,
        r = fill.r,
        g = fill.g,
        b = fill.b,
        x = mm_to_pt(x_mm),
        y = mm_to_pt(y_mm),
    ))
}

fn pdf_escape_text(s: &str) -> Result<String, PdfError> {
    // Helvetica built-in is WinAnsi; reject non-ASCII until a CJK font package exists.
    if !s.is_ascii() {
        return Err(PdfError::InvalidShape(
            "text contains non-ASCII; CJK fonts are not wired yet (see JLReq plan)".into(),
        ));
    }
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                return Err(PdfError::InvalidShape(format!(
                    "unsupported control char U+{:04X} in text",
                    c as u32
                )));
            }
            c => out.push(c),
        }
    }
    Ok(out)
}

fn mm_to_pt(mm: f64) -> f64 {
    mm * 72.0 / 25.4
}

/// Assemble PDF-1.4 with a shared Helvetica font resource.
fn assemble_pdf(page_sizes: &[(f64, f64)], contents: &[String]) -> Vec<u8> {
    assert_eq!(page_sizes.len(), contents.len());
    let n = page_sizes.len();
    // 1 Catalog, 2 Pages, 3 Font, 4..3+n Pages, then contents
    let font_obj = 3;
    let page_obj0 = 4;
    let content_obj0 = 4 + n;

    let mut objects: Vec<Vec<u8>> = Vec::new();
    objects.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());

    let kids: String = (0..n)
        .map(|i| format!("{} 0 R", page_obj0 + i))
        .collect::<Vec<_>>()
        .join(" ");
    objects.push(format!("<< /Type /Pages /Kids [{kids}] /Count {n} >>").into_bytes());

    objects.push(
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
            .to_vec(),
    );

    for (i, (w, h)) in page_sizes.iter().enumerate() {
        let content_id = content_obj0 + i;
        let page_id_body = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w:.4} {h:.4}] \
             /Contents {content_id} 0 R \
             /Resources << /Font << /F1 {font_obj} 0 R >> >> >>"
        );
        objects.push(page_id_body.into_bytes());
    }

    for content in contents {
        let stream = content.as_bytes();
        let mut obj = format!("<< /Length {} >>\nstream\n", stream.len()).into_bytes();
        obj.extend_from_slice(stream);
        if !content.ends_with('\n') {
            obj.push(b'\n');
        }
        obj.extend_from_slice(b"endstream");
        objects.push(obj);
    }

    let mut out: Vec<u8> = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::with_capacity(objects.len() + 1);
    offsets.push(0);

    for (i, obj) in objects.iter().enumerate() {
        offsets.push(out.len());
        let id = i + 1;
        out.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
        out.extend_from_slice(obj);
        out.extend_from_slice(b"\nendobj\n");
    }

    let xref_pos = out.len();
    let total_objs = objects.len() + 1;
    out.extend_from_slice(format!("xref\n0 {total_objs}\n").as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for off in offsets.iter().skip(1) {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!("trailer\n<< /Size {total_objs} /Root 1 0 R >>\nstartxref\n{xref_pos}\n%%EOF\n")
            .as_bytes(),
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_scene::{
        Circle, Color, Document, Ellipse, Line, Page, PaperSize, Rect, Shape, Text,
    };

    fn sample_doc() -> Document {
        Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Circle(Circle {
                x_mm: 105.0,
                y_mm: 148.5,
                radius_mm: 40.0,
                fill: Color::BLACK,
            })],
        })
    }

    #[test]
    fn black_circle_pdf_has_header_eof_and_a4_mediabox() {
        let bytes = document_to_pdf(&sample_doc()).expect("pdf");
        assert!(bytes.starts_with(b"%PDF-"));
        assert!(bytes.windows(5).any(|w| w == b"%%EOF"));
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("595."));
        assert!(text.contains("841."));
        assert!(text.contains("/Helvetica"));
    }

    #[test]
    fn text_and_line_emit_operators() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![
                Shape::Text(Text {
                    x_mm: 20.0,
                    y_mm: 250.0,
                    size_mm: 5.0,
                    content: "Hello".into(),
                    fill: Color::BLACK,
                }),
                Shape::Line(Line {
                    x1_mm: 20.0,
                    y1_mm: 200.0,
                    x2_mm: 100.0,
                    y2_mm: 200.0,
                    stroke: Color::RED,
                    width_mm: 0.5,
                }),
            ],
        });
        let bytes = document_to_pdf(&doc).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("BT\n"));
        assert!(text.contains("(Hello) Tj"));
        assert!(text.contains(" l\nS\n") || text.contains(" l\r\nS"));
    }

    #[test]
    fn non_ascii_text_rejected() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Text(Text {
                x_mm: 10.0,
                y_mm: 10.0,
                size_mm: 4.0,
                content: "日本語".into(),
                fill: Color::BLACK,
            })],
        });
        assert!(matches!(
            document_to_pdf(&doc),
            Err(PdfError::InvalidShape(_))
        ));
    }

    #[test]
    fn empty_document_errors() {
        assert_eq!(
            document_to_pdf(&Document::default()),
            Err(PdfError::EmptyDocument)
        );
    }

    #[test]
    fn rect_pdf_contains_re_operator() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Rect(Rect {
                x_mm: 10.0,
                y_mm: 20.0,
                width_mm: 30.0,
                height_mm: 40.0,
                fill: Color::BLUE,
            })],
        });
        let bytes = document_to_pdf(&doc).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains(" re\n"));
    }

    #[test]
    fn ellipse_pdf_contains_curve_ops() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Ellipse(Ellipse {
                x_mm: 105.0,
                y_mm: 148.5,
                rx_mm: 60.0,
                ry_mm: 30.0,
                fill: Color::GREEN,
            })],
        });
        let bytes = document_to_pdf(&doc).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains(" c\n"));
        assert!(text.contains("\nf\n") || text.ends_with("f\n"));
    }
}
