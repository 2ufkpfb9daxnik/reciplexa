//! Minimal PDF emitter for [`reciplexa_scene::Document`].
//!
//! Implemented by hand (no `printpdf` / font stack) so the backend stays
//! tiny, deterministic, and easy to unit-test. Enough for filled circles
//! on ISO pages; richer drawing can grow behind the same API.

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

/// Emit a `cm` operator. Linear parts stay unitless; translation mm → pt.
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
    // Four cubic Béziers with κ ≈ 0.5522847498 (standard circle approx).
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

fn mm_to_pt(mm: f64) -> f64 {
    mm * 72.0 / 25.4
}

/// Assemble a minimal PDF-1.4 document with N pages.
fn assemble_pdf(page_sizes: &[(f64, f64)], contents: &[String]) -> Vec<u8> {
    assert_eq!(page_sizes.len(), contents.len());
    let n = page_sizes.len();
    // Object layout:
    // 1: Catalog
    // 2: Pages
    // 3..2+n: Page dicts
    // 3+n..2+2n: Content streams
    let page_obj0 = 3;
    let content_obj0 = 3 + n;

    let mut objects: Vec<Vec<u8>> = Vec::new();
    // obj 1 — catalog
    objects.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());

    // obj 2 — pages
    let kids: String = (0..n)
        .map(|i| format!("{} 0 R", page_obj0 + i))
        .collect::<Vec<_>>()
        .join(" ");
    objects.push(format!("<< /Type /Pages /Kids [{kids}] /Count {n} >>").into_bytes());

    for (i, (w, h)) in page_sizes.iter().enumerate() {
        let content_id = content_obj0 + i;
        let page_id_body = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w:.4} {h:.4}] \
             /Contents {content_id} 0 R /Resources << >> >>"
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
    offsets.push(0); // object 0 placeholder

    for (i, obj) in objects.iter().enumerate() {
        offsets.push(out.len());
        let id = i + 1;
        out.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
        out.extend_from_slice(obj);
        out.extend_from_slice(b"\nendobj\n");
    }

    let xref_pos = out.len();
    let total_objs = objects.len() + 1; // include free object 0
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
    use reciplexa_scene::{Affine, Circle, Color, Document, Page, PaperSize, Shape};

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

    // --- validity ---

    #[test]
    fn black_circle_pdf_has_header_eof_and_a4_mediabox() {
        let bytes = document_to_pdf(&sample_doc()).expect("pdf");
        assert!(bytes.starts_with(b"%PDF-"));
        assert!(bytes.windows(5).any(|w| w == b"%%EOF"));
        let text = String::from_utf8_lossy(&bytes);
        // A4 in points: 210mm → 595.2756…, 297mm → 841.8898…
        assert!(text.contains("595."));
        assert!(text.contains("841."));
        assert!(text.contains(" rg"));
        assert!(text.contains("\nf\n") || text.contains(" f\n"));
    }

    #[test]
    fn mm_to_pt_known_values() {
        assert!((mm_to_pt(25.4) - 72.0).abs() < 1e-9);
        assert!((mm_to_pt(210.0) - 595.275_590_551).abs() < 1e-6);
    }

    #[test]
    fn write_document_matches_bytes() {
        let mut buf = Vec::new();
        write_document(&sample_doc(), &mut buf).unwrap();
        assert_eq!(buf, document_to_pdf(&sample_doc()).unwrap());
    }

    #[test]
    fn two_pages_increment_count() {
        let mut doc = sample_doc();
        doc.pages.push(doc.pages[0].clone());
        let bytes = document_to_pdf(&doc).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("/Count 2"));
    }

    // --- defect ---

    #[test]
    fn empty_document_errors() {
        assert_eq!(
            document_to_pdf(&Document::default()),
            Err(PdfError::EmptyDocument)
        );
    }

    #[test]
    fn zero_radius_circle_errors() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Circle(Circle {
                x_mm: 10.0,
                y_mm: 10.0,
                radius_mm: 0.0,
                fill: Color::BLACK,
            })],
        });
        assert!(matches!(
            document_to_pdf(&doc),
            Err(PdfError::InvalidShape(_))
        ));
    }

    #[test]
    fn non_positive_paper_errors() {
        let doc = Document::single_page(Page {
            paper: PaperSize {
                width_mm: -1.0,
                height_mm: 10.0,
            },
            shapes: vec![],
        });
        assert!(matches!(
            document_to_pdf(&doc),
            Err(PdfError::InvalidPage(_))
        ));
    }

    #[test]
    fn transformed_colored_circle_emits_cm_and_non_black_fill() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Group {
                transform: Affine::translate(105.0, 148.5)
                    .then(Affine::rotate_deg(30.0))
                    .then(Affine::scale_uniform(1.5)),
                children: vec![Shape::Circle(Circle {
                    x_mm: 0.0,
                    y_mm: 0.0,
                    radius_mm: 20.0,
                    fill: Color::RED,
                })],
            }],
        });
        let bytes = document_to_pdf(&doc).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains(" cm\n"));
        assert!(text.contains("q\n"));
        assert!(text.contains("Q\n"));
        assert!(text.contains("1.0000 0.0000 0.0000 rg"));
    }

    #[test]
    fn non_finite_transform_errors() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Group {
                transform: Affine {
                    a: f64::INFINITY,
                    ..Affine::identity()
                },
                children: vec![],
            }],
        });
        assert!(matches!(
            document_to_pdf(&doc),
            Err(PdfError::InvalidShape(_))
        ));
    }

    #[test]
    fn rect_pdf_contains_re_operator() {
        use reciplexa_scene::Rect;
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
        assert!(text.contains("0.0000 0.0000 1.0000 rg"));
    }
}
