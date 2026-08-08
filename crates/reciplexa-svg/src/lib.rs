//! SVG export from a scene [`Document`] via the Phase 7 backend pipeline.
//!
//! This crate is a thin host-facing adapter over `reciplexa-backend`. The
//! legacy flatten→SVG path has been replaced so CLI and GUI share planning,
//! emission, and artifact validation.

#![forbid(unsafe_code)]

use std::io::{self, Write};

use reciplexa_backend::{
    export_scene_to_svg_with_hints, BackendCapability, ExportError, OutputProfile,
};
use reciplexa_scene::Document;
use reciplexa_visual_ir::NodeSourceHint;

/// Write one SVG document through the planned backend pipeline.
pub fn write_document(doc: &Document, mut out: impl Write) -> io::Result<()> {
    let svg = document_to_svg(doc).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    out.write_all(svg.as_bytes())
}

/// Write SVG with paint-order source provenance hints.
pub fn write_document_with_hints(
    doc: &Document,
    hints: &[Option<NodeSourceHint>],
    mut out: impl Write,
) -> io::Result<()> {
    let svg = document_to_svg_with_hints(doc, hints)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    out.write_all(svg.as_bytes())
}

/// Render `doc` to an SVG string (millimeter units) via planning + validation.
pub fn document_to_svg(doc: &Document) -> Result<String, String> {
    document_to_svg_with_hints(doc, &[])
}

/// Render with paint-order provenance hints threaded into artifact provenance.
pub fn document_to_svg_with_hints(
    doc: &Document,
    hints: &[Option<NodeSourceHint>],
) -> Result<String, String> {
    let artifact = export_scene_to_svg_with_hints(
        doc,
        &BackendCapability::svg_default(),
        &OutputProfile::svg_default(),
        hints,
    )
    .map_err(fmt_export_error)?;
    Ok(artifact.svg)
}

fn fmt_export_error(e: ExportError) -> String {
    match e {
        ExportError::Render(e) => format!("render: {e:?}"),
        ExportError::Planning(e) => format!("planning: {e:?}"),
        ExportError::Emit(e) => format!("emit: {e:?}"),
        ExportError::Artifact(e) => format!("artifact: {e:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_scene::{
        Circle, Color, Document, Ellipse, Frame, Line, Page, PaperSize, Polygon, Polyline, Rect,
        Ring, Shape, Text,
    };
    use reciplexa_visual_ir::NodeSourceHint;

    fn a4(shapes: Vec<Shape>) -> Document {
        Document {
            pages: vec![Page {
                paper: PaperSize::a4(),
                shapes,
            }],
        }
    }

    #[test]
    fn black_circle_svg_has_root_and_circle() {
        let doc = a4(vec![Shape::Circle(Circle {
            x_mm: 105.0,
            y_mm: 148.5,
            radius_mm: 20.0,
            fill: Color::BLACK,
        })]);
        let svg = document_to_svg(&doc).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("<circle"));
        assert!(svg.contains("cx=\"105\""));
        assert!(svg.contains("r=\"20\""));
        assert!(svg.contains("id=\"page-1\""));
        assert!(svg.contains("id=\"rpx-"));
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
        let svg = document_to_svg(&doc).unwrap();
        assert!(svg.contains("id=\"page-1\""));
        assert!(svg.contains("id=\"page-2\""));
        assert!(svg.contains("translate(0 307)")); // 297 + 10 gap
    }

    #[test]
    fn rect_line_text_shapes_export() {
        let doc = a4(vec![
            Shape::Rect(Rect {
                x_mm: 10.0,
                y_mm: 10.0,
                width_mm: 50.0,
                height_mm: 30.0,
                fill: Color::BLUE,
            }),
            Shape::Line(Line {
                x1_mm: 0.0,
                y1_mm: 0.0,
                x2_mm: 100.0,
                y2_mm: 50.0,
                stroke: Color::RED,
                width_mm: 0.5,
            }),
            Shape::Text(Text {
                x_mm: 20.0,
                y_mm: 250.0,
                size_mm: 6.0,
                width_mm: None,
                height_mm: None,
                content: "SVG".into(),
                fill: Color::BLACK,
            }),
        ]);
        let svg = document_to_svg(&doc).unwrap();
        assert!(svg.contains("<rect"));
        assert!(svg.contains("<line") || svg.contains("<path"));
        assert!(svg.contains("SVG"));
    }

    #[test]
    fn ring_frame_polyline_polygon_export() {
        let doc = a4(vec![
            Shape::Ring(Ring {
                x_mm: 40.0,
                y_mm: 40.0,
                radius_mm: 15.0,
                width_mm: 1.0,
                stroke: Color::RED,
            }),
            Shape::Frame(Frame {
                x_mm: 5.0,
                y_mm: 5.0,
                width_mm: 30.0,
                height_mm: 20.0,
                stroke_width_mm: 0.4,
                stroke: Color::GREEN,
            }),
            Shape::Polyline(Polyline {
                points_mm: vec![(0.0, 0.0), (20.0, 10.0)],
                stroke: Color::BLACK,
                width_mm: 0.3,
            }),
            Shape::Polygon(Polygon {
                points_mm: vec![(60.0, 60.0), (80.0, 60.0), (70.0, 80.0)],
                fill: Color::BLUE,
            }),
        ]);
        let svg = document_to_svg(&doc).unwrap();
        assert!(svg.contains("stroke"));
        assert!(svg.contains("fill"));
    }

    #[test]
    fn ellipse_and_opacity_export() {
        let doc = a4(vec![
            Shape::Ellipse(Ellipse {
                x_mm: 50.0,
                y_mm: 50.0,
                rx_mm: 20.0,
                ry_mm: 10.0,
                fill: Color::GREEN,
            }),
            Shape::Opacity {
                alpha: 0.25,
                children: vec![Shape::Circle(Circle {
                    x_mm: 100.0,
                    y_mm: 100.0,
                    radius_mm: 10.0,
                    fill: Color::RED,
                })],
            },
        ]);
        let svg = document_to_svg(&doc).unwrap();
        assert!(svg.contains("opacity") || svg.contains("fill-opacity"));
    }

    #[test]
    fn write_document_streams_svg() {
        let doc = a4(vec![Shape::Circle(Circle {
            x_mm: 10.0,
            y_mm: 10.0,
            radius_mm: 5.0,
            fill: Color::BLACK,
        })]);
        let mut buf = Vec::new();
        write_document(&doc, &mut buf).unwrap();
        let text = String::from_utf8_lossy(&buf);
        assert!(text.contains("<svg"));
    }

    #[test]
    fn write_document_with_provenance_hints() {
        use reciplexa_identity::document::StableNodeId;
        let doc = a4(vec![Shape::Rect(Rect {
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 10.0,
            height_mm: 10.0,
            fill: Color::RED,
        })]);
        let hints = vec![Some(NodeSourceHint {
            stable_node_id: StableNodeId(1),
            source_byte_start: 0,
            source_byte_end: 12,
        })];
        let mut buf = Vec::new();
        super::write_document_with_hints(&doc, &hints, &mut buf).unwrap();
        assert!(String::from_utf8_lossy(&buf).contains("<rect"));
        let svg = document_to_svg_with_hints(&doc, &hints).unwrap();
        assert!(svg.contains("<rect"));
    }

    #[test]
    fn fmt_export_error_formats_all_variants() {
        use reciplexa_backend::{
            capability::CapabilityMismatch, emit::EmitError, verify::ArtifactValidationError,
            PlanningError,
        };
        use reciplexa_visual_ir::RenderValidationError;
        assert!(
            fmt_export_error(ExportError::Render(RenderValidationError::EmptyDocument))
                .contains("render")
        );
        assert!(
            fmt_export_error(ExportError::Planning(PlanningError::CapabilityMismatch(
                vec![CapabilityMismatch {
                    feature: "text".into(),
                    required_by_profile: true,
                }]
            )))
            .contains("planning")
        );
        assert!(
            fmt_export_error(ExportError::Emit(EmitError::PlanNodeMismatch {
                render_id: 1,
                representation: "SvgCircle".into(),
            }))
            .contains("emit")
        );
        assert!(
            fmt_export_error(ExportError::Artifact(ArtifactValidationError::ContainsNaN))
                .contains("artifact")
        );
    }

    #[test]
    fn write_document_io_error_after_valid_svg() {
        struct FailWrite;
        impl std::io::Write for FailWrite {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "fail"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let doc = a4(vec![]);
        let err = write_document(&doc, FailWrite).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::Other);
    }
}
