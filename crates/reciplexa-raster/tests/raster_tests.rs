use reciplexa_raster::*;
use reciplexa_scene::{Circle, Color, Document, Page, PaperSize, Rect, Shape, Text};

fn a4_doc(shapes: Vec<Shape>) -> Document {
    Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes,
    })
}

#[test]
fn rasterizes_filled_rect_and_circle() {
    let doc = a4_doc(vec![
        Shape::Rect(Rect {
            x_mm: 10.0,
            y_mm: 10.0,
            width_mm: 20.0,
            height_mm: 15.0,
            fill: Color::RED,
        }),
        Shape::Circle(Circle {
            x_mm: 50.0,
            y_mm: 50.0,
            radius_mm: 8.0,
            fill: Color::BLUE,
        }),
    ]);
    let frame = rasterize_page(&doc, 0, &RasterOptions::default()).unwrap();
    assert!(frame.width > 100);
    assert!(frame.height > 100);
    assert_eq!(frame.rgb.len(), (frame.width * frame.height * 3) as usize);
    assert!(frame.losses.is_empty());
    let png = frame_to_png(&frame).unwrap();
    assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
}

#[test]
fn reports_text_loss_explicitly() {
    let doc = a4_doc(vec![Shape::Text(Text {
        x_mm: 1.0,
        y_mm: 2.0,
        size_mm: 12.0,
        width_mm: None,
        height_mm: None,
        content: "hi".into(),
        fill: Color::BLACK,
    })]);
    let frame = rasterize_page(&doc, 0, &RasterOptions::default()).unwrap();
    assert!(frame.losses.contains(&RasterLoss::TextSkipped));
}

#[test]
fn empty_and_bad_options_error() {
    let empty = Document { pages: vec![] };
    assert!(matches!(
        rasterize_page(&empty, 0, &RasterOptions::default()),
        Err(RasterError::EmptyDocument)
    ));
    let doc = a4_doc(vec![]);
    let mut opts = RasterOptions::default();
    opts.px_per_mm = f64::NAN;
    assert!(matches!(
        rasterize_page(&doc, 0, &opts),
        Err(RasterError::BadOptions(_))
    ));
    assert!(matches!(
        rasterize_page(&doc, 9, &RasterOptions::default()),
        Err(RasterError::PageOutOfRange(9))
    ));
}

#[test]
fn document_page_to_png_roundtrip_header() {
    let doc = a4_doc(vec![Shape::Circle(Circle {
        x_mm: 30.0,
        y_mm: 30.0,
        radius_mm: 5.0,
        fill: Color::BLACK,
    })]);
    let (bytes, losses) = document_page_to_png(&doc, 0, &RasterOptions::default()).unwrap();
    assert!(bytes.starts_with(&[0x89, b'P', b'N', b'G']));
    assert!(losses.is_empty());
}
