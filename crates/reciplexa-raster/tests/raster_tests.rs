use reciplexa_raster::*;
use reciplexa_scene::{
    Circle, Color, Document, Image, Line, Page, PaperSize, Polygon, Polyline, Rect, Shape, Text,
};

fn a4_doc(shapes: Vec<Shape>) -> Document {
    Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes,
    })
}

fn tiny_doc(shapes: Vec<Shape>) -> Document {
    Document::single_page(Page {
        paper: PaperSize {
            width_mm: 0.01,
            height_mm: 0.01,
        },
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
fn reports_image_loss_explicitly() {
    let doc = a4_doc(vec![Shape::Image(Image {
        path: "photo.png".into(),
        x_mm: 10.0,
        y_mm: 10.0,
        width_mm: 40.0,
        height_mm: 30.0,
    })]);
    let frame = rasterize_page(&doc, 0, &RasterOptions::default()).unwrap();
    assert!(frame.losses.contains(&RasterLoss::ImageSkipped));
    assert!(!frame.losses.contains(&RasterLoss::PathAsPolyline));
}

#[test]
fn path_polyline_stroke_reports_loss_and_draws() {
    let doc = a4_doc(vec![
        Shape::Line(Line {
            x1_mm: 20.0,
            y1_mm: 20.0,
            x2_mm: 80.0,
            y2_mm: 60.0,
            stroke: Color::BLACK,
            width_mm: 1.0,
        }),
        Shape::Polyline(Polyline {
            points_mm: vec![(30.0, 100.0), (50.0, 120.0), (70.0, 100.0), (90.0, 130.0)],
            stroke: Color::RED,
            width_mm: 2.0,
        }),
    ]);
    let frame = rasterize_page(&doc, 0, &RasterOptions::default()).unwrap();
    assert!(frame.losses.contains(&RasterLoss::PathAsPolyline));
    // Stroke should paint something other than pure white background.
    assert!(frame.rgb.chunks(3).any(|px| px != [255, 255, 255]));
}

#[test]
fn stroke_width_edges_thin_thick_and_degenerate_segments() {
    let doc = a4_doc(vec![
        // Thin stroke → width_px*0.5 clamped up to 0.5px radius.
        Shape::Line(Line {
            x1_mm: 5.0,
            y1_mm: 5.0,
            x2_mm: 15.0,
            y2_mm: 5.0,
            stroke: Color::BLUE,
            width_mm: 0.01,
        }),
        // Thick stroke.
        Shape::Line(Line {
            x1_mm: 40.0,
            y1_mm: 40.0,
            x2_mm: 60.0,
            y2_mm: 40.0,
            stroke: Color::GREEN,
            width_mm: 8.0,
        }),
        // Zero-length segment still advances one dab (steps.max(1)).
        Shape::Polyline(Polyline {
            points_mm: vec![(100.0, 100.0), (100.0, 100.0), (110.0, 100.0)],
            stroke: Color::BLACK,
            width_mm: 1.0,
        }),
        // <2 points → stroke_polyline early return.
        Shape::Polyline(Polyline {
            points_mm: vec![(1.0, 1.0)],
            stroke: Color::RED,
            width_mm: 1.0,
        }),
        Shape::Polyline(Polyline {
            points_mm: vec![],
            stroke: Color::RED,
            width_mm: 1.0,
        }),
    ]);
    let frame = rasterize_page(&doc, 0, &RasterOptions::default()).unwrap();
    assert!(frame.losses.contains(&RasterLoss::PathAsPolyline));
}

#[test]
fn zero_radius_circle_is_noop() {
    let doc = a4_doc(vec![Shape::Circle(Circle {
        x_mm: 50.0,
        y_mm: 50.0,
        radius_mm: 0.0,
        fill: Color::RED,
    })]);
    let frame = rasterize_page(&doc, 0, &RasterOptions::default()).unwrap();
    assert!(frame.rgb.chunks(3).all(|px| px == [255, 255, 255]));
}

#[test]
fn negative_radius_circle_is_noop() {
    let doc = a4_doc(vec![Shape::Circle(Circle {
        x_mm: 50.0,
        y_mm: 50.0,
        radius_mm: -3.0,
        fill: Color::RED,
    })]);
    let frame = rasterize_page(&doc, 0, &RasterOptions::default()).unwrap();
    assert!(frame.rgb.chunks(3).all(|px| px == [255, 255, 255]));
}

#[test]
fn degenerate_polygon_skipped() {
    let doc = a4_doc(vec![
        Shape::Polygon(Polygon {
            points_mm: vec![],
            fill: Color::RED,
        }),
        Shape::Polygon(Polygon {
            points_mm: vec![(10.0, 10.0)],
            fill: Color::RED,
        }),
        Shape::Polygon(Polygon {
            points_mm: vec![(10.0, 10.0), (20.0, 10.0)],
            fill: Color::RED,
        }),
    ]);
    let frame = rasterize_page(&doc, 0, &RasterOptions::default()).unwrap();
    assert!(frame.rgb.chunks(3).all(|px| px == [255, 255, 255]));
}

#[test]
fn off_page_geometry_clips_pixels() {
    // Circle and stroke entirely outside / straddling the tiny page → put_pixel OOB.
    let doc = tiny_doc(vec![
        Shape::Circle(Circle {
            x_mm: 50.0,
            y_mm: 50.0,
            radius_mm: 5.0,
            fill: Color::RED,
        }),
        Shape::Line(Line {
            x1_mm: -10.0,
            y1_mm: -10.0,
            x2_mm: -5.0,
            y2_mm: -5.0,
            stroke: Color::BLACK,
            width_mm: 2.0,
        }),
    ]);
    let opts = RasterOptions {
        px_per_mm: 10.0,
        background: Color::WHITE,
    };
    let frame = rasterize_page(&doc, 0, &opts).unwrap();
    assert_eq!(frame.width, 1);
    assert_eq!(frame.height, 1);
    assert!(frame.losses.contains(&RasterLoss::PathAsPolyline));
}

#[test]
fn color_channel_clamp_and_custom_background() {
    let doc = a4_doc(vec![Shape::Circle(Circle {
        x_mm: 30.0,
        y_mm: 30.0,
        radius_mm: 4.0,
        fill: Color::new(2.0, -1.0, 0.5),
    })]);
    let opts = RasterOptions {
        px_per_mm: 2.0,
        background: Color::new(-0.5, 1.5, 0.25),
    };
    let frame = rasterize_page(&doc, 0, &opts).unwrap();
    // Background after clamp: r=0, g=255, b≈64
    assert_eq!(&frame.rgb[0..3], &[0, 255, 64]);
}

#[test]
fn loss_dedup_across_mixed_shapes() {
    let doc = a4_doc(vec![
        Shape::Text(Text {
            x_mm: 1.0,
            y_mm: 1.0,
            size_mm: 10.0,
            width_mm: None,
            height_mm: None,
            content: "a".into(),
            fill: Color::BLACK,
        }),
        Shape::Text(Text {
            x_mm: 2.0,
            y_mm: 2.0,
            size_mm: 10.0,
            width_mm: None,
            height_mm: None,
            content: "b".into(),
            fill: Color::BLACK,
        }),
        Shape::Image(Image {
            path: "a.png".into(),
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 10.0,
            height_mm: 10.0,
        }),
        Shape::Image(Image {
            path: "b.png".into(),
            x_mm: 20.0,
            y_mm: 20.0,
            width_mm: 10.0,
            height_mm: 10.0,
        }),
        Shape::Line(Line {
            x1_mm: 0.0,
            y1_mm: 0.0,
            x2_mm: 5.0,
            y2_mm: 5.0,
            stroke: Color::BLACK,
            width_mm: 0.5,
        }),
        Shape::Line(Line {
            x1_mm: 10.0,
            y1_mm: 10.0,
            x2_mm: 15.0,
            y2_mm: 15.0,
            stroke: Color::BLACK,
            width_mm: 0.5,
        }),
    ]);
    let frame = rasterize_page(&doc, 0, &RasterOptions::default()).unwrap();
    assert_eq!(frame.losses.len(), 3);
    assert!(frame.losses.contains(&RasterLoss::TextSkipped));
    assert!(frame.losses.contains(&RasterLoss::ImageSkipped));
    assert!(frame.losses.contains(&RasterLoss::PathAsPolyline));
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
    opts.px_per_mm = f64::INFINITY;
    assert!(matches!(
        rasterize_page(&doc, 0, &opts),
        Err(RasterError::BadOptions(_))
    ));
    opts.px_per_mm = 0.0;
    assert!(matches!(
        rasterize_page(&doc, 0, &opts),
        Err(RasterError::BadOptions(_))
    ));
    opts.px_per_mm = -1.0;
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

#[test]
fn document_page_to_png_propagates_rasterize_errors() {
    let empty = Document { pages: vec![] };
    assert!(matches!(
        document_page_to_png(&empty, 0, &RasterOptions::default()),
        Err(RasterError::EmptyDocument)
    ));
    let doc = a4_doc(vec![]);
    let mut opts = RasterOptions::default();
    opts.px_per_mm = -2.0;
    assert!(matches!(
        document_page_to_png(&doc, 0, &opts),
        Err(RasterError::BadOptions(_))
    ));
    assert!(matches!(
        document_page_to_png(&doc, 3, &RasterOptions::default()),
        Err(RasterError::PageOutOfRange(3))
    ));
}

#[test]
fn frame_to_png_encode_rejects_mismatched_buffer() {
    let bad = RasterFrame {
        width: 4,
        height: 4,
        rgb: vec![0; 3], // too short for 4×4 RGB8
        losses: vec![],
    };
    assert!(matches!(frame_to_png(&bad), Err(RasterError::Encode(_))));

    let zero = RasterFrame {
        width: 0,
        height: 0,
        rgb: vec![],
        losses: vec![],
    };
    assert!(matches!(frame_to_png(&zero), Err(RasterError::Encode(_))));
}

#[test]
fn raster_options_default_and_error_debug() {
    let d = RasterOptions::default();
    assert!(d.px_per_mm > 0.0);
    assert_eq!(d.background, Color::WHITE);
    let _ = format!("{:?}", RasterError::EmptyDocument);
    let _ = format!("{:?}", RasterLoss::ImageSkipped);
    let _ = format!("{:?}", RasterOptions::default());
}
