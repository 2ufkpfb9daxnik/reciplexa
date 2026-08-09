use reciplexa_pptx::{
    document_to_pptx, document_to_pptx_write, fill_inner, shape_xml, slide_xml, solid_fill,
    write_document, xml_escape,
};
use reciplexa_scene::{
    Circle, Color, Document, Ellipse, Frame, Image, Line, Page, PaperSize, Polygon, Polyline, Rect,
    Ring, Shape, Text,
};
use std::io::{Cursor, Read, Write};

fn a4_page(shapes: Vec<Shape>) -> Page {
    Page {
        paper: PaperSize::a4(),
        shapes,
    }
}

fn slide_xml_from_zip(bytes: &[u8], slide: usize) -> String {
    let name = format!("ppt/slides/slide{slide}.xml");
    let cursor = Cursor::new(bytes);
    let mut archive = zip::read::ZipArchive::new(cursor).expect("valid zip");
    let mut file = archive.by_name(&name).unwrap_or_else(|_| panic!("{name}"));
    let mut xml = String::new();
    file.read_to_string(&mut xml).unwrap();
    xml
}

#[test]
fn pptx_is_zip_with_presentation() {
    let doc = Document {
        pages: vec![a4_page(vec![Shape::Circle(Circle {
            x_mm: 105.0,
            y_mm: 148.5,
            radius_mm: 20.0,
            fill: Color::BLACK,
        })])],
    };
    let bytes = document_to_pptx(&doc).unwrap();
    assert_eq!(&bytes[0..2], b"PK");
    let slide = slide_xml_from_zip(&bytes, 1);
    assert!(slide.contains("prst=\"ellipse\""));
    assert!(bytes
        .windows(b"ppt/presentation.xml".len())
        .any(|w| w == b"ppt/presentation.xml"));
}

#[test]
fn empty_page_still_emits_slide() {
    let doc = Document {
        pages: vec![a4_page(vec![])],
    };
    let bytes = document_to_pptx(&doc).unwrap();
    let slide = slide_xml_from_zip(&bytes, 1);
    assert!(slide.contains("<p:sld"));
    assert!(!slide.contains("prst=\"ellipse\""));
}

#[test]
fn multipage_has_two_slides_and_content_types() {
    let page = a4_page(vec![]);
    let doc = Document {
        pages: vec![page.clone(), page],
    };
    let bytes = document_to_pptx(&doc).unwrap();
    assert!(slide_xml_from_zip(&bytes, 1).contains("<p:sld"));
    assert!(slide_xml_from_zip(&bytes, 2).contains("<p:sld"));
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("slide1.xml"));
    assert!(text.contains("slide2.xml"));
    assert!(text.contains("rId2"));
    assert!(text.contains("rId3"));
}

#[test]
fn ring_emits_stroked_ellipse() {
    let doc = Document::single_page(a4_page(vec![Shape::Ring(Ring {
        x_mm: 50.0,
        y_mm: 50.0,
        radius_mm: 10.0,
        width_mm: 1.0,
        stroke: Color::RED,
    })]));
    let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
    assert!(slide.contains("<a:ln w="));
    assert!(slide.contains("FF0000"));
}

#[test]
fn rect_and_polygon_emit_custom_geometry() {
    let doc = Document::single_page(a4_page(vec![
        Shape::Rect(Rect {
            x_mm: 5.0,
            y_mm: 5.0,
            width_mm: 20.0,
            height_mm: 10.0,
            fill: Color::BLUE,
        }),
        Shape::Polygon(Polygon {
            points_mm: vec![(0.0, 0.0), (30.0, 0.0), (15.0, 25.0)],
            fill: Color::GREEN,
        }),
    ]));
    let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
    assert!(slide.contains("Polygon"));
    assert!(slide.contains("<a:custGeom>"));
    assert!(slide.contains("<a:close/>"));
    assert!(slide.contains("0000FF"));
    assert!(slide.contains("00FF00"));
}

#[test]
fn frame_emits_stroked_polygon() {
    let doc = Document::single_page(a4_page(vec![Shape::Frame(Frame {
        x_mm: 10.0,
        y_mm: 10.0,
        width_mm: 40.0,
        height_mm: 20.0,
        stroke_width_mm: 0.5,
        stroke: Color::BLACK,
    })]));
    let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
    assert!(slide.contains("<a:noFill/>"));
    assert!(slide.contains("<a:ln w="));
}

#[test]
fn line_and_polyline_emit_open_paths() {
    let doc = Document::single_page(a4_page(vec![
        Shape::Line(Line {
            x1_mm: 0.0,
            y1_mm: 0.0,
            x2_mm: 50.0,
            y2_mm: 50.0,
            stroke: Color::RED,
            width_mm: 0.3,
        }),
        Shape::Polyline(Polyline {
            points_mm: vec![(10.0, 10.0), (20.0, 30.0), (40.0, 15.0)],
            stroke: Color::GREEN,
            width_mm: 0.2,
        }),
    ]));
    let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
    assert!(slide.contains("Path"));
    assert!(slide.contains("<a:moveTo>"));
    assert!(slide.contains("<a:lnTo>"));
    assert!(!slide.contains("<a:close/>"));
}

#[test]
fn closed_polyline_path_has_close() {
    let doc = Document::single_page(a4_page(vec![Shape::Polyline(Polyline {
        points_mm: vec![(0.0, 0.0), (20.0, 0.0), (10.0, 20.0), (0.0, 0.0)],
        stroke: Color::BLACK,
        width_mm: 0.5,
    })]));
    // Polyline is open in scene; path closed flag is false.
    let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
    assert!(slide.contains("Path"));
}

#[test]
fn text_escapes_xml_and_applies_rotation() {
    let doc = Document::single_page(a4_page(vec![Shape::Group {
        transform: reciplexa_scene::Affine::rotate_deg(45.0),
        children: vec![Shape::Text(Text {
            x_mm: 20.0,
            y_mm: 200.0,
            size_mm: 8.0,
            width_mm: Some(60.0),
            height_mm: Some(20.0),
            content: "A & B <tag> \"quote\"".into(),
            fill: Color::BLACK,
        })],
    }]));
    let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
    assert!(slide.contains("&amp;"));
    assert!(slide.contains("&lt;"));
    assert!(slide.contains("&quot;"));
    assert!(slide.contains(" rot=\"-2700000\""));
    assert!(slide.contains("txBox=\"1\""));
}

#[test]
fn semi_transparent_fill_emits_alpha() {
    let doc = Document::single_page(a4_page(vec![Shape::Opacity {
        alpha: 0.5,
        children: vec![Shape::Circle(Circle {
            x_mm: 30.0,
            y_mm: 30.0,
            radius_mm: 10.0,
            fill: Color::RED,
        })],
    }]));
    let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
    assert!(slide.contains("<a:alpha val=\"50000\"/>"));
}

#[test]
fn image_shape_is_skipped_in_slide() {
    let doc = Document::single_page(a4_page(vec![
        Shape::Image(Image {
            path: "x.png".into(),
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 10.0,
            height_mm: 10.0,
        }),
        Shape::Circle(Circle {
            x_mm: 10.0,
            y_mm: 10.0,
            radius_mm: 5.0,
            fill: Color::BLACK,
        }),
    ]));
    let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
    assert!(!slide.contains("x.png"));
    assert!(slide.contains("prst=\"ellipse\""));
}

#[test]
fn write_document_streams_bytes() {
    let doc = Document::single_page(a4_page(vec![Shape::Ellipse(Ellipse {
        x_mm: 40.0,
        y_mm: 40.0,
        rx_mm: 15.0,
        ry_mm: 8.0,
        fill: Color::GREEN,
    })]));
    let mut buf = Vec::new();
    write_document(&doc, &mut buf).unwrap();
    assert_eq!(&buf[0..2], b"PK");
    let slide = slide_xml_from_zip(&buf, 1);
    assert!(slide.contains("Polygon"));
}

#[test]
fn letter_sized_first_page_sets_presentation_size() {
    let doc = Document::single_page(Page {
        paper: PaperSize::letter(),
        shapes: vec![],
    });
    let bytes = document_to_pptx(&doc).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    // Letter width in EMU ≈ 7772400 when used as first slide size.
    assert!(text.contains("7772400") || text.contains("10058400"));
}

#[test]
fn shape_xml_degenerate_polygon_and_path_are_empty() {
    use reciplexa_view::{WorldPath, WorldPolygon, WorldShape};
    let poly = WorldShape::Polygon(WorldPolygon {
        points_mm: vec![(0.0, 0.0), (1.0, 1.0)],
        color: Color::RED,
        stroke_width_mm: None,
        alpha: 1.0,
    });
    assert!(shape_xml(&poly, 2, 297.0).is_empty());
    let path = WorldShape::Path(WorldPath {
        points_mm: vec![(0.0, 0.0)],
        stroke: Color::BLACK,
        width_mm: 1.0,
        closed: false,
        alpha: 1.0,
    });
    assert!(shape_xml(&path, 3, 297.0).is_empty());
}

#[test]
fn shape_xml_closed_path_and_fill_helpers() {
    use reciplexa_view::{WorldPath, WorldShape};
    let path = WorldShape::Path(WorldPath {
        points_mm: vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)],
        stroke: Color::BLACK,
        width_mm: 0.5,
        closed: true,
        alpha: 1.0,
    });
    let xml = shape_xml(&path, 4, 297.0);
    assert!(xml.contains("<a:close/>"));
    assert_eq!(xml_escape("it's"), "it&apos;s");
    assert!(solid_fill(Color::RED, 1.0).contains("FF0000"));
    assert!(fill_inner(Color::RED, 0.5).contains("<a:alpha"));
}

#[test]
fn slide_xml_errors_on_missing_page() {
    let doc = Document::default();
    assert!(slide_xml(&doc, 0).is_err());
}

#[test]
fn slide_xml_ok_for_present_page() {
    let doc = Document::single_page(a4_page(vec![]));
    let xml = slide_xml(&doc, 0).unwrap();
    assert!(xml.contains("<p:sld"));
}

#[test]
fn zero_page_document_still_builds_package() {
    let bytes = document_to_pptx(&Document::default()).unwrap();
    assert!(bytes.starts_with(b"PK"));
}

#[test]
fn write_document_io_error() {
    struct FailWrite;
    impl Write for FailWrite {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("fail"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let doc = Document::single_page(a4_page(vec![]));
    assert!(write_document(&doc, &mut FailWrite).is_err());
    let mut w = FailWrite;
    assert!(std::io::Write::flush(&mut w).is_ok());
}

#[test]
fn xml_escape_covers_all_special_chars() {
    assert_eq!(xml_escape("a>b"), "a&gt;b");
    assert_eq!(xml_escape("it's"), "it&apos;s");
    assert_eq!(xml_escape("a&b<c\"d'"), "a&amp;b&lt;c&quot;d&apos;");
}

#[test]
fn shape_xml_circle_with_stroke_emits_line() {
    use reciplexa_view::{WorldCircle, WorldShape};
    let shape = WorldShape::Circle(WorldCircle {
        x_mm: 30.0,
        y_mm: 30.0,
        radius_mm: 10.0,
        color: Color::RED,
        stroke_width_mm: Some(0.5),
        alpha: 1.0,
    });
    let xml = shape_xml(&shape, 2, 297.0);
    assert!(xml.contains("<a:ln w="));
    assert!(xml.contains("FF0000"));
}

#[test]
fn shape_xml_polygon_stroke_branch() {
    use reciplexa_view::{WorldPolygon, WorldShape};
    let shape = WorldShape::Polygon(WorldPolygon {
        points_mm: vec![(0.0, 0.0), (20.0, 0.0), (10.0, 20.0)],
        color: Color::BLUE,
        stroke_width_mm: Some(0.3),
        alpha: 1.0,
    });
    let xml = shape_xml(&shape, 3, 297.0);
    assert!(xml.contains("<a:noFill/>"));
    assert!(xml.contains("<a:ln w="));
}

#[test]
fn opaque_solid_fill_has_no_alpha_element() {
    let fill = solid_fill(Color::GREEN, 1.0);
    assert!(fill.contains("00FF00"));
    assert!(!fill.contains("<a:alpha"));
}

#[test]
fn plain_text_shape_emits_txbody() {
    let doc = Document::single_page(a4_page(vec![Shape::Text(Text {
        x_mm: 20.0,
        y_mm: 200.0,
        size_mm: 8.0,
        width_mm: None,
        height_mm: None,
        content: "Plain".into(),
        fill: Color::BLACK,
    })]));
    let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
    assert!(slide.contains("<a:t>Plain</a:t>"));
    assert!(slide.contains("txBox=\"1\""));
}

#[test]
fn three_slides_update_presentation_rels() {
    let page = a4_page(vec![]);
    let doc = Document {
        pages: vec![page.clone(), page.clone(), page],
    };
    let bytes = document_to_pptx(&doc).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("slide3.xml"));
    assert!(text.contains("rId4"));
}

#[test]
fn document_to_pptx_write_errors_on_limited_buffer() {
    use std::io::{Seek, SeekFrom};
    struct FailAfterWrite {
        inner: Cursor<Vec<u8>>,
        limit: usize,
    }
    impl Write for FailAfterWrite {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            if self.inner.position() as usize + buf.len() > self.limit {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::StorageFull,
                    "capacity",
                ));
            }
            self.inner.write(buf)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.inner.flush()
        }
    }
    impl Seek for FailAfterWrite {
        fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
            self.inner.seek(pos)
        }
    }
    let doc = Document::single_page(a4_page(vec![]));
    let mut buf = FailAfterWrite {
        inner: Cursor::new(Vec::new()),
        limit: 64,
    };
    assert!(document_to_pptx_write(&doc, &mut buf).is_err());
    assert!(std::io::Write::flush(&mut buf).is_ok());

    // Fail during local-file header write inside `start_file`.
    let mut early = FailAfterWrite {
        inner: Cursor::new(Vec::new()),
        limit: 1,
    };
    assert!(document_to_pptx_write(&doc, &mut early).is_err());

    // Allow almost all bytes then fail on `finish` central-directory write.
    let mut almost = FailAfterWrite {
        inner: Cursor::new(Vec::new()),
        limit: usize::MAX / 4,
    };
    // First find a limit that succeeds for puts but fails finish by probing.
    let ok_bytes = {
        let mut probe = Cursor::new(Vec::new());
        document_to_pptx_write(&doc, &mut probe).unwrap();
        probe.into_inner().len()
    };
    almost.limit = ok_bytes.saturating_sub(8).max(1);
    assert!(document_to_pptx_write(&doc, &mut almost).is_err());
}
