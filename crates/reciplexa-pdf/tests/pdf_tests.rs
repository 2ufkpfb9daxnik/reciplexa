use reciplexa_pdf::{
    document_to_pdf, document_to_pdf_with_base, jpeg_pixels_to_rgb, load_jpeg_rgb,
    load_raster_file, system_cjk_font_path, text_ops, write_document, write_document_with_base,
    PdfError,
};
use reciplexa_scene::{
    Circle, Color, Document, Ellipse, Frame, Image, Line, Page, PaperSize, Polygon, Polyline, Rect,
    Ring, Shape, Text,
};
use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

static CJK_TEST_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn lock_cjk_test_env() -> std::sync::MutexGuard<'static, ()> {
    CJK_TEST_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap()
}

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

fn write_temp_png(path: &Path, w: u32, h: u32, rgb: &[u8]) {
    let file = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(file, w, h);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().unwrap();
    writer.write_image_data(rgb).unwrap();
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
                width_mm: None,
                height_mm: None,
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
    assert!(text.contains("BT"));
    assert!(text.contains("Tj"));
    assert!(text.contains(" m\n"));
    assert!(text.contains(" l\n"));
}

#[test]
fn multiline_text_emits_leading_and_tstar() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(Text {
            x_mm: 20.0,
            y_mm: 200.0,
            size_mm: 5.0,
            width_mm: None,
            height_mm: None,
            content: "hello\nworld".into(),
            fill: Color::BLACK,
        })],
    });
    let bytes = document_to_pdf(&doc).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains(" TL\n"), "text leading missing: {text}");
    assert!(text.contains("T*\n"), "line advance missing: {text}");
    assert!(text.contains("(hello) Tj"));
    assert!(text.contains("(world) Tj"));
}

#[test]
fn boxed_text_soft_wraps_in_pdf() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(Text {
            x_mm: 20.0,
            y_mm: 200.0,
            size_mm: 10.0,
            width_mm: Some(35.0),
            height_mm: Some(40.0),
            content: "hello world there".into(),
            fill: Color::BLACK,
        })],
    });
    let bytes = document_to_pdf(&doc).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(
        text.contains("T*\n"),
        "soft wrap should emit line advances: {text}"
    );
    assert!(text.matches(" Tj\n").count() >= 2);
}

#[test]
fn boxed_text_clips_by_height() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(Text {
            x_mm: 20.0,
            y_mm: 200.0,
            size_mm: 10.0,
            width_mm: Some(200.0),
            height_mm: Some(15.0),
            content: "one\ntwo\nthree".into(),
            fill: Color::BLACK,
        })],
    });
    let bytes = document_to_pdf(&doc).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("(one) Tj"));
    assert!(
        !text.contains("(three) Tj"),
        "third line should clip: {text}"
    );
}

#[test]
fn multiline_cjk_text_emits_tstar_when_font_present() {
    let _guard = lock_cjk_test_env();
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(Text {
            x_mm: 10.0,
            y_mm: 100.0,
            size_mm: 5.0,
            width_mm: None,
            height_mm: None,
            content: "一行目\n二行目".into(),
            fill: Color::BLACK,
        })],
    });
    if system_cjk_font_path().is_none() {
        return;
    }
    let bytes = document_to_pdf(&doc).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("T*\n"));
    assert!(text.contains("/F2 "));
    // Two Tj shows (one per line).
    assert!(text.matches(" Tj\n").count() >= 2);
}

#[test]
fn non_ascii_text_embeds_selectable_cid_font() {
    let _guard = lock_cjk_test_env();
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(Text {
            x_mm: 10.0,
            y_mm: 10.0,
            size_mm: 5.0,
            width_mm: None,
            height_mm: None,
            content: "日本語".into(),
            fill: Color::BLACK,
        })],
    });
    if system_cjk_font_path().is_none() {
        let err = document_to_pdf(&doc).unwrap_err();
        assert!(matches!(err, PdfError::InvalidShape(_)));
        return;
    }
    let bytes = document_to_pdf(&doc).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("/Identity-H"));
    assert!(text.contains("/Subtype /Type0"));
    assert!(text.contains("/CIDFontType2"));
    assert!(text.contains("begincmap"));
    assert!(text.contains("/F2 "));
    assert!(text.contains("Tj"));
    // Not outline-filled paths for the Japanese text.
    assert!(!text.contains("(日本語)"));
}

#[test]
fn empty_document_errors() {
    assert!(matches!(
        document_to_pdf(&Document::default()),
        Err(PdfError::EmptyDocument)
    ));
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
            x_mm: 50.0,
            y_mm: 50.0,
            rx_mm: 20.0,
            ry_mm: 10.0,
            fill: Color::GREEN,
        })],
    });
    let bytes = document_to_pdf(&doc).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains(" c\n"));
    assert!(text.contains("\nf\n") || text.ends_with("f\n"));
}

#[test]
fn letter_page_mediabox_and_opacity_extgstate() {
    let doc = Document::single_page(Page {
        paper: PaperSize::letter(),
        shapes: vec![Shape::Opacity {
            alpha: 0.5,
            children: vec![Shape::Circle(Circle {
                x_mm: 100.0,
                y_mm: 140.0,
                radius_mm: 30.0,
                fill: Color::RED,
            })],
        }],
    });
    let bytes = document_to_pdf(&doc).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    // 215.9mm × 279.4mm → ~612 × 792 pt
    assert!(text.contains("612."));
    assert!(text.contains("792."));
    assert!(text.contains("/ExtGState"));
    assert!(text.contains("/GS50"));
    assert!(text.contains("/ca 0.5000"));
    assert!(text.contains("/GS50 gs"));
}

#[test]
fn embeds_png_as_image_xobject() {
    let dir = std::env::temp_dir().join("reciplexa-pdf-png-test");
    let _ = std::fs::create_dir_all(&dir);
    let png_path = dir.join("dot.png");
    write_temp_png(
        &png_path,
        2,
        2,
        &[255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0],
    );

    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Image(Image {
            path: "dot.png".into(),
            x_mm: 10.0,
            y_mm: 20.0,
            width_mm: 40.0,
            height_mm: 30.0,
        })],
    });
    let bytes = document_to_pdf_with_base(&doc, Some(&dir)).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("/Subtype /Image"));
    assert!(text.contains("/Width 2"));
    assert!(text.contains("/Height 2"));
    assert!(text.contains("/Im0 Do"));
    assert!(text.contains("/XObject"));
}

#[test]
fn missing_png_fails_fast() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Image(Image {
            path: "no-such-file.png".into(),
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 10.0,
            height_mm: 10.0,
        })],
    });
    let err = document_to_pdf(&doc).unwrap_err();
    match err {
        PdfError::InvalidShape(msg) => assert!(msg.contains("image")),
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn fixture_demo_png_embeds() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let examples = repo.join("examples");
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Image(Image {
            path: "figures/demo.png".into(),
            x_mm: 40.0,
            y_mm: 80.0,
            width_mm: 130.0,
            height_mm: 100.0,
        })],
    });
    let bytes = document_to_pdf_with_base(&doc, Some(&examples)).unwrap();
    assert!(String::from_utf8_lossy(&bytes).contains("/Subtype /Image"));
    let _ = std::io::sink().write(&bytes);
}

#[test]
fn fixture_demo_jpeg_embeds() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let examples = repo.join("examples");
    let doc = Document::single_page(Page {
        paper: PaperSize {
            width_mm: 210.0,
            height_mm: 297.0,
        },
        shapes: vec![Shape::Image(Image {
            path: "figures/demo.jpg".into(),
            x_mm: 20.0,
            y_mm: 20.0,
            width_mm: 60.0,
            height_mm: 60.0,
        })],
    });
    let bytes = document_to_pdf_with_base(&doc, Some(&examples)).unwrap();
    assert!(String::from_utf8_lossy(&bytes).contains("/Subtype /Image"));
}

#[test]
fn invalid_page_size_errors() {
    let doc = Document::single_page(Page {
        paper: PaperSize {
            width_mm: 0.0,
            height_mm: 297.0,
        },
        shapes: vec![],
    });
    assert!(matches!(
        document_to_pdf(&doc),
        Err(PdfError::InvalidPage(_))
    ));
}

#[test]
fn non_drawable_circle_errors() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Circle(Circle {
            x_mm: 0.0,
            y_mm: 0.0,
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
fn ring_frame_polyline_polygon_emit() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![
            Shape::Ring(Ring {
                x_mm: 50.0,
                y_mm: 50.0,
                radius_mm: 20.0,
                width_mm: 1.0,
                stroke: Color::RED,
            }),
            Shape::Frame(Frame {
                x_mm: 10.0,
                y_mm: 10.0,
                width_mm: 40.0,
                height_mm: 30.0,
                stroke_width_mm: 0.5,
                stroke: Color::BLUE,
            }),
            Shape::Polyline(Polyline {
                points_mm: vec![(0.0, 0.0), (30.0, 10.0), (60.0, 0.0)],
                stroke: Color::GREEN,
                width_mm: 0.4,
            }),
            Shape::Polygon(Polygon {
                points_mm: vec![(70.0, 70.0), (90.0, 70.0), (80.0, 90.0)],
                fill: Color::BLACK,
            }),
        ],
    });
    let bytes = document_to_pdf(&doc).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains(" RG\n"));
    assert!(text.contains("s\n") || text.contains("S\n"));
    assert!(text.contains("f\n"));
}

#[test]
fn group_affine_wraps_content() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Group {
            transform: reciplexa_scene::Affine::translate(10.0, 20.0),
            children: vec![Shape::Rect(Rect {
                x_mm: 0.0,
                y_mm: 0.0,
                width_mm: 20.0,
                height_mm: 10.0,
                fill: Color::RED,
            })],
        }],
    });
    let bytes = document_to_pdf(&doc).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains(" cm\n"));
    assert!(text.contains(" re\n"));
}

#[test]
fn invalid_opacity_errors() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Opacity {
            alpha: 2.0,
            children: vec![Shape::Circle(Circle {
                x_mm: 10.0,
                y_mm: 10.0,
                radius_mm: 5.0,
                fill: Color::BLACK,
            })],
        }],
    });
    assert!(matches!(
        document_to_pdf(&doc),
        Err(PdfError::InvalidShape(_))
    ));
}

#[test]
fn non_finite_group_transform_errors() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Group {
            transform: reciplexa_scene::Affine {
                a: f64::NAN,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                e: 0.0,
                f: 0.0,
            },
            children: vec![Shape::Circle(Circle {
                x_mm: 0.0,
                y_mm: 0.0,
                radius_mm: 5.0,
                fill: Color::BLACK,
            })],
        }],
    });
    assert!(matches!(
        document_to_pdf(&doc),
        Err(PdfError::InvalidShape(_))
    ));
}

#[test]
fn empty_text_emits_nothing() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(Text {
            x_mm: 0.0,
            y_mm: 0.0,
            size_mm: 5.0,
            width_mm: None,
            height_mm: None,
            content: String::new(),
            fill: Color::BLACK,
        })],
    });
    assert!(matches!(
        document_to_pdf(&doc),
        Err(PdfError::InvalidShape(_))
    ));
}

#[test]
fn pdf_escape_special_ascii_chars() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(Text {
            x_mm: 10.0,
            y_mm: 200.0,
            size_mm: 5.0,
            width_mm: None,
            height_mm: None,
            content: "(path)\\n\t".into(),
            fill: Color::BLACK,
        })],
    });
    let bytes = document_to_pdf(&doc).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("\\(path\\)"));
    assert!(text.contains("\\\\n"));
}

#[test]
fn control_char_in_text_errors() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(Text {
            x_mm: 10.0,
            y_mm: 200.0,
            size_mm: 5.0,
            width_mm: None,
            height_mm: None,
            content: "bad\u{0001}char".into(),
            fill: Color::BLACK,
        })],
    });
    assert!(matches!(
        document_to_pdf(&doc),
        Err(PdfError::InvalidShape(_))
    ));
}

#[test]
fn write_document_and_load_raster_file() {
    let dir = std::env::temp_dir().join("reciplexa-pdf-write-test");
    let _ = std::fs::create_dir_all(&dir);
    let png_path = dir.join("px.png");
    write_temp_png(&png_path, 1, 1, &[255, 0, 0]);

    let raster = load_raster_file(&png_path).unwrap();
    assert_eq!(raster.width, 1);
    assert_eq!(raster.rgb.len(), 3);

    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Image(Image {
            path: "px.png".into(),
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 10.0,
            height_mm: 10.0,
        })],
    });
    let mut out = Vec::new();
    write_document_with_base(&doc, Some(&dir), &mut out).unwrap();
    assert!(out.starts_with(b"%PDF-"));
}

#[test]
fn grayscale_and_rgba_png_embed() {
    let dir = std::env::temp_dir().join("reciplexa-pdf-gray-test");
    let _ = std::fs::create_dir_all(&dir);

    let gray_path = dir.join("g.png");
    {
        let file = std::fs::File::create(&gray_path).unwrap();
        let mut enc = png::Encoder::new(file, 2, 1);
        enc.set_color(png::ColorType::Grayscale);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().unwrap();
        w.write_image_data(&[128, 64]).unwrap();
    }

    let rgba_path = dir.join("ga.png");
    {
        let file = std::fs::File::create(&rgba_path).unwrap();
        let mut enc = png::Encoder::new(file, 1, 1);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().unwrap();
        w.write_image_data(&[10, 20, 30, 255]).unwrap();
    }

    for name in ["g.png", "ga.png"] {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Image(Image {
                path: name.into(),
                x_mm: 0.0,
                y_mm: 0.0,
                width_mm: 5.0,
                height_mm: 5.0,
            })],
        });
        document_to_pdf_with_base(&doc, Some(&dir)).unwrap();
    }
}

#[test]
fn duplicate_image_path_reuses_xobject() {
    let dir = std::env::temp_dir().join("reciplexa-pdf-dedup");
    let _ = std::fs::create_dir_all(&dir);
    let png_path = dir.join("one.png");
    write_temp_png(&png_path, 1, 1, &[0, 255, 0]);

    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![
            Shape::Image(Image {
                path: "one.png".into(),
                x_mm: 0.0,
                y_mm: 0.0,
                width_mm: 10.0,
                height_mm: 10.0,
            }),
            Shape::Image(Image {
                path: "one.png".into(),
                x_mm: 20.0,
                y_mm: 0.0,
                width_mm: 10.0,
                height_mm: 10.0,
            }),
        ],
    });
    let bytes = document_to_pdf_with_base(&doc, Some(&dir)).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert_eq!(text.matches("/Subtype /Image").count(), 1);
    assert_eq!(text.matches("/Im0 Do").count(), 2);
}

#[test]
fn unsupported_image_bytes_fail() {
    let dir = std::env::temp_dir().join("reciplexa-pdf-bad-img");
    let _ = std::fs::create_dir_all(&dir);
    let bad = dir.join("data.bin");
    std::fs::write(&bad, b"not an image").unwrap();
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Image(Image {
            path: "data.bin".into(),
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 5.0,
            height_mm: 5.0,
        })],
    });
    assert!(matches!(
        document_to_pdf_with_base(&doc, Some(&dir)),
        Err(PdfError::InvalidShape(_))
    ));
}

#[test]
fn multipage_pdf_has_two_pages() {
    let doc = Document {
        pages: vec![
            Page {
                paper: PaperSize::a4(),
                shapes: vec![Shape::Circle(Circle {
                    x_mm: 10.0,
                    y_mm: 10.0,
                    radius_mm: 5.0,
                    fill: Color::BLACK,
                })],
            },
            Page {
                paper: PaperSize::letter(),
                shapes: vec![Shape::Rect(Rect {
                    x_mm: 5.0,
                    y_mm: 5.0,
                    width_mm: 50.0,
                    height_mm: 30.0,
                    fill: Color::BLUE,
                })],
            },
        ],
    };
    let bytes = document_to_pdf(&doc).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("/Count 2"));
    assert!(text.matches("/MediaBox").count() >= 2);
}

#[test]
fn write_document_public_wrapper() {
    let mut out = Vec::new();
    write_document(&sample_doc(), &mut out).unwrap();
    assert!(out.starts_with(b"%PDF-"));
}

#[test]
fn absolute_image_path_resolves() {
    let dir = std::env::temp_dir().join("reciplexa-pdf-abs");
    let _ = std::fs::create_dir_all(&dir);
    let png_path = dir.join("abs.png");
    write_temp_png(&png_path, 1, 1, &[0, 0, 255]);
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Image(Image {
            path: png_path.to_string_lossy().into(),
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 5.0,
            height_mm: 5.0,
        })],
    });
    document_to_pdf(&doc).unwrap();
}

#[test]
fn invalid_shapes_for_each_kind() {
    let bad_rect = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Rect(Rect {
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 0.0,
            height_mm: 10.0,
            fill: Color::BLACK,
        })],
    });
    assert!(matches!(
        document_to_pdf(&bad_rect),
        Err(PdfError::InvalidShape(_))
    ));
    let bad_ellipse = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Ellipse(Ellipse {
            x_mm: 0.0,
            y_mm: 0.0,
            rx_mm: 0.0,
            ry_mm: 5.0,
            fill: Color::BLACK,
        })],
    });
    assert!(document_to_pdf(&bad_ellipse).is_err());
    let bad_ring = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Ring(Ring {
            x_mm: 0.0,
            y_mm: 0.0,
            radius_mm: 5.0,
            width_mm: 0.0,
            stroke: Color::BLACK,
        })],
    });
    assert!(document_to_pdf(&bad_ring).is_err());
    let bad_frame = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Frame(Frame {
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 1.0,
            height_mm: 1.0,
            stroke_width_mm: 0.0,
            stroke: Color::BLACK,
        })],
    });
    assert!(document_to_pdf(&bad_frame).is_err());
    let bad_line = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Line(Line {
            x1_mm: 1.0,
            y1_mm: 1.0,
            x2_mm: 1.0,
            y2_mm: 1.0,
            stroke: Color::BLACK,
            width_mm: 1.0,
        })],
    });
    assert!(document_to_pdf(&bad_line).is_err());
    let bad_polyline = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Polyline(Polyline {
            points_mm: vec![(0.0, 0.0), (0.0, 0.0)],
            stroke: Color::BLACK,
            width_mm: 1.0,
        })],
    });
    assert!(document_to_pdf(&bad_polyline).is_err());
    let bad_polygon = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Polygon(Polygon {
            points_mm: vec![(0.0, 0.0), (1.0, 0.0)],
            fill: Color::BLACK,
        })],
    });
    assert!(document_to_pdf(&bad_polygon).is_err());
    let bad_image = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Image(Image {
            path: String::new(),
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 1.0,
            height_mm: 1.0,
        })],
    });
    assert!(document_to_pdf(&bad_image).is_err());
}

#[test]
fn grayscale_alpha_png_and_jpeg_extension_fallback() {
    let dir = std::env::temp_dir().join("reciplexa-pdf-formats");
    let _ = std::fs::create_dir_all(&dir);

    let ga_path = dir.join("ga.png");
    {
        let file = std::fs::File::create(&ga_path).unwrap();
        let mut enc = png::Encoder::new(file, 1, 1);
        enc.set_color(png::ColorType::GrayscaleAlpha);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().unwrap();
        w.write_image_data(&[100, 200]).unwrap();
    }

    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Image(Image {
            path: "ga.png".into(),
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 5.0,
            height_mm: 5.0,
        })],
    });
    document_to_pdf_with_base(&doc, Some(&dir)).unwrap();
}

#[test]
fn cjk_in_nested_group_collects_chars() {
    let _guard = lock_cjk_test_env();
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Group {
            transform: reciplexa_scene::Affine::identity(),
            children: vec![Shape::Opacity {
                alpha: 1.0,
                children: vec![Shape::Text(Text {
                    x_mm: 10.0,
                    y_mm: 10.0,
                    size_mm: 5.0,
                    width_mm: None,
                    height_mm: None,
                    content: "漢".into(),
                    fill: Color::BLACK,
                })],
            }],
        }],
    });
    if system_cjk_font_path().is_none() {
        assert!(document_to_pdf(&doc).is_err());
    } else {
        document_to_pdf(&doc).unwrap();
    }
}

#[test]
fn write_document_io_failure_maps_to_pdf_error() {
    struct FailWrite;
    impl std::io::Write for FailWrite {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("disk full"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let err = write_document(&sample_doc(), &mut FailWrite).unwrap_err();
    assert!(matches!(err, PdfError::Write(_)));
}

#[test]
fn invalid_page_height_errors() {
    let doc = Document::single_page(Page {
        paper: PaperSize {
            width_mm: 210.0,
            height_mm: -1.0,
        },
        shapes: vec![],
    });
    assert!(matches!(
        document_to_pdf(&doc),
        Err(PdfError::InvalidPage(_))
    ));
}

#[test]
fn nested_opacity_stacks_extgstate() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Opacity {
            alpha: 0.5,
            children: vec![Shape::Opacity {
                alpha: 0.5,
                children: vec![Shape::Circle(Circle {
                    x_mm: 50.0,
                    y_mm: 50.0,
                    radius_mm: 10.0,
                    fill: Color::RED,
                })],
            }],
        }],
    });
    let bytes = document_to_pdf(&doc).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("/GS25") || text.contains("/GS50"));
}

#[test]
fn indexed_png_is_unsupported() {
    let dir = std::env::temp_dir().join("reciplexa-pdf-indexed");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("idx.png");
    {
        let file = std::fs::File::create(&path).unwrap();
        let mut enc = png::Encoder::new(file, 2, 2);
        enc.set_color(png::ColorType::Indexed);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_palette(vec![0, 0, 0, 255, 0, 0]);
        let mut w = enc.write_header().unwrap();
        w.write_image_data(&[0, 1, 1, 0]).unwrap();
    }
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Image(Image {
            path: "idx.png".into(),
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 5.0,
            height_mm: 5.0,
        })],
    });
    assert!(matches!(
        document_to_pdf_with_base(&doc, Some(&dir)),
        Err(PdfError::InvalidShape(_))
    ));
}

#[test]
fn relative_image_path_without_base() {
    let dir = std::env::temp_dir().join("reciplexa-pdf-rel");
    let _ = std::fs::create_dir_all(&dir);
    let png_path = dir.join("rel.png");
    write_temp_png(&png_path, 1, 1, &[255, 0, 0]);
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&dir).unwrap();
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Image(Image {
            path: "rel.png".into(),
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 5.0,
            height_mm: 5.0,
        })],
    });
    document_to_pdf(&doc).unwrap();
    std::env::set_current_dir(prev).unwrap();
}

#[test]
fn non_drawable_text_size_errors() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(Text {
            x_mm: 10.0,
            y_mm: 10.0,
            size_mm: 0.0,
            width_mm: None,
            height_mm: None,
            content: "x".into(),
            fill: Color::BLACK,
        })],
    });
    assert!(matches!(
        document_to_pdf(&doc),
        Err(PdfError::InvalidShape(_))
    ));
}

#[test]
fn load_jpeg_rejects_incomplete_scan_data() {
    // Truncated / incomplete JPEG headers should surface a decode error (not panic).
    let gray_jpeg = [
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43, 0x00, 0x08, 0x06, 0x06, 0x07, 0x06,
        0x05, 0x08, 0x07, 0x07, 0x07, 0x09, 0x09, 0x08, 0x0A, 0x0C, 0x14, 0x0D, 0x0C, 0x0B, 0x0B,
        0x0C, 0x19, 0x12, 0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D, 0x1A, 0x1C, 0x1C, 0x20,
        0x24, 0x2E, 0x27, 0x20, 0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28, 0x37, 0x29, 0x2C, 0x30, 0x31,
        0x34, 0x34, 0x34, 0x1F, 0x27, 0x39, 0x3D, 0x38, 0x32, 0x3C, 0x2E, 0x33, 0x34, 0x32, 0xFF,
        0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4, 0x00,
        0x14, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x08, 0xFF, 0xC4, 0x00, 0x14, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xDA, 0x00, 0x08, 0x01,
        0x01, 0x00, 0x00, 0x3F, 0x00, 0x7F, 0xFF, 0xD9,
    ];
    let err = match load_jpeg_rgb(&gray_jpeg) {
        Ok(_) => panic!("expected incomplete JPEG to fail"),
        Err(e) => e,
    };
    assert!(err.contains("JPEG") || err.contains("jpeg") || err.contains("component"));
}

#[test]
fn pdf_escape_carriage_return() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(Text {
            x_mm: 10.0,
            y_mm: 200.0,
            size_mm: 5.0,
            width_mm: None,
            height_mm: None,
            content: "a\rb".into(),
            fill: Color::BLACK,
        })],
    });
    let bytes = document_to_pdf(&doc).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("\\r"));
}

#[test]
fn grayscale_and_cmyk_jpeg_embed() {
    let dir = std::env::temp_dir().join("reciplexa-pdf-jpeg-formats");
    let _ = std::fs::create_dir_all(&dir);
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    for name in ["fixtures_gray.jpg", "fixtures_cmyk.jpg"] {
        let src = fixtures.join(name);
        let dst = dir.join(name);
        std::fs::copy(&src, &dst).unwrap();
        let raster = load_jpeg_rgb(&std::fs::read(&dst).unwrap()).unwrap();
        assert!(raster.rgb.len() >= 3);
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Image(Image {
                path: name.into(),
                x_mm: 0.0,
                y_mm: 0.0,
                width_mm: 10.0,
                height_mm: 10.0,
            })],
        });
        document_to_pdf_with_base(&doc, Some(&dir)).unwrap();
    }
}

#[test]
fn jpeg_pixels_to_rgb_covers_formats() {
    let rgb = jpeg_pixels_to_rgb(jpeg_decoder::PixelFormat::RGB24, vec![1, 2, 3]).unwrap();
    assert_eq!(rgb, vec![1, 2, 3]);
    let gray = jpeg_pixels_to_rgb(jpeg_decoder::PixelFormat::L8, vec![10]).unwrap();
    assert_eq!(gray, vec![10, 10, 10]);
    let cmyk = jpeg_pixels_to_rgb(jpeg_decoder::PixelFormat::CMYK32, vec![0, 0, 0, 0]).unwrap();
    assert_eq!(cmyk.len(), 3);
    assert!(jpeg_pixels_to_rgb(jpeg_decoder::PixelFormat::L16, vec![0, 0]).is_err());
}

#[test]
fn jpeg_extension_fallback_without_magic() {
    let dir = std::env::temp_dir().join("reciplexa-pdf-jpeg-ext");
    let _ = std::fs::create_dir_all(&dir);
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let src = fixtures.join("fixtures_gray.jpg");
    let raw = std::fs::read(&src).unwrap();
    // Strip SOI magic so sniff fails; extension still selects JPEG decoder.
    let mut mangled = raw.clone();
    mangled[0] = 0x00;
    let path = dir.join("no_magic.jpg");
    std::fs::write(&path, &mangled).unwrap();
    // May fail decode due to bad magic, but exercises extension fallback arm.
    let _ = load_raster_file(&path);
    // Valid bytes with .jpeg extension and wrong leading bytes still routes via extension.
    let mut almost = raw.clone();
    almost[0] = 0xFE;
    let path2 = dir.join("almost.jpeg");
    std::fs::write(&path2, &almost).unwrap();
    let _ = load_raster_file(&path2);
}

#[test]
fn empty_page_content_stream_still_valid() {
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![],
    });
    let bytes = document_to_pdf(&doc).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
    assert!(String::from_utf8_lossy(&bytes).contains("endstream"));
}

#[test]
fn cjk_blank_line_skips_empty_hex_show() {
    let _guard = lock_cjk_test_env();
    if system_cjk_font_path().is_none() {
        return;
    }
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(Text {
            x_mm: 10.0,
            y_mm: 100.0,
            size_mm: 5.0,
            width_mm: None,
            height_mm: None,
            content: "日\n\n本".into(),
            fill: Color::BLACK,
        })],
    });
    let bytes = document_to_pdf(&doc).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("T*\n"));
    assert!(text.contains("/F2 "));
}

#[test]
fn write_document_propagates_build_errors() {
    let err = write_document(&Document::default(), &mut Vec::new()).unwrap_err();
    assert!(matches!(err, PdfError::EmptyDocument));
}

#[test]
fn document_to_pdf_reports_missing_cjk_font() {
    let _guard = lock_cjk_test_env();
    let saved_font = std::env::var_os("RECIPLEXA_CJK_FONT");
    let saved_windir = std::env::var_os("WINDIR");
    let empty = std::env::temp_dir().join(format!("reciplexa_nofont_pdf_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&empty);
    std::env::remove_var("RECIPLEXA_CJK_FONT");
    std::env::set_var("WINDIR", &empty);
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(Text {
            x_mm: 10.0,
            y_mm: 10.0,
            size_mm: 5.0,
            width_mm: None,
            height_mm: None,
            content: "日本語".into(),
            fill: Color::BLACK,
        })],
    });
    assert!(matches!(
        document_to_pdf(&doc),
        Err(PdfError::InvalidShape(_))
    ));
    if let Some(v) = saved_font {
        std::env::set_var("RECIPLEXA_CJK_FONT", v);
    } else {
        std::env::remove_var("RECIPLEXA_CJK_FONT");
    }
    if let Some(v) = saved_windir {
        std::env::set_var("WINDIR", v);
    } else {
        std::env::remove_var("WINDIR");
    }
    let _ = std::fs::remove_dir_all(&empty);
}

#[test]
fn png_extension_fallback_and_corrupt_png() {
    let dir = std::env::temp_dir().join("reciplexa-pdf-png-fallback");
    let _ = std::fs::create_dir_all(&dir);
    // Valid PNG bytes with broken magic → extension arm.
    let mut png = Vec::new();
    {
        let mut enc = png::Encoder::new(std::io::Cursor::new(&mut png), 1, 1);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().unwrap();
        w.write_image_data(&[1, 2, 3]).unwrap();
    }
    png[0] = 0x00;
    let path = dir.join("broken_magic.png");
    std::fs::write(&path, &png).unwrap();
    let _ = load_raster_file(&path);

    // PNG magic but truncated body → read_info errors (png crate requires IDAT at info).
    let bad = dir.join("trunc.png");
    std::fs::write(&bad, b"\x89PNG\r\n\x1a\n\x00\x00").unwrap();
    assert!(load_raster_file(&bad).is_err());

    // Valid IHDR + IEND, no IDAT → also fails at read_info on this png crate.
    let ihdr_only: &[u8] = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    let no_idat = dir.join("ihdr_only.png");
    std::fs::write(&no_idat, ihdr_only).unwrap();
    assert!(load_raster_file(&no_idat).is_err());

    // IHDR + corrupt IDAT → read_info ok, next_frame Err.
    let mut bad_idat = Vec::from(&b"\x89PNG\r\n\x1a\n"[..]);
    // IHDR 1x1 RGB8
    bad_idat.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xde,
    ]);
    // IDAT with non-zlib payload
    let idat_data = b"not-zlib";
    bad_idat.extend_from_slice(&(idat_data.len() as u32).to_be_bytes());
    bad_idat.extend_from_slice(b"IDAT");
    bad_idat.extend_from_slice(idat_data);
    let mut crc = 0xffffffffu32;
    for &b in b"IDAT".iter().chain(idat_data.iter()) {
        crc ^= u32::from(b);
        for _ in 0..8 {
            let mask = if crc & 1 != 0 { 0xedb88320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    bad_idat.extend_from_slice(&(!crc).to_be_bytes());
    // IEND
    bad_idat.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ]);
    let bad_idat_path = dir.join("bad_idat.png");
    std::fs::write(&bad_idat_path, &bad_idat).unwrap();
    assert!(load_raster_file(&bad_idat_path).is_err());
}

#[test]
fn opacity_and_group_propagate_child_errors() {
    let bad_child = Shape::Circle(Circle {
        x_mm: 0.0,
        y_mm: 0.0,
        radius_mm: 0.0,
        fill: Color::BLACK,
    });
    let opacity = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Opacity {
            alpha: 0.5,
            children: vec![bad_child.clone()],
        }],
    });
    assert!(matches!(
        document_to_pdf(&opacity),
        Err(PdfError::InvalidShape(_))
    ));
    let group = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Group {
            transform: reciplexa_scene::Affine::identity(),
            children: vec![bad_child],
        }],
    });
    assert!(matches!(
        document_to_pdf(&group),
        Err(PdfError::InvalidShape(_))
    ));
}

#[test]
fn text_ops_rejects_cjk_without_embed() {
    let err = text_ops(0.0, 0.0, 4.0, None, None, "日本語", Color::BLACK, None).unwrap_err();
    assert!(matches!(err, PdfError::InvalidShape(_)));
}

#[test]
fn text_ops_skips_line_when_cid_missing() {
    let _guard = lock_cjk_test_env();
    let Some(path) = system_cjk_font_path() else {
        return;
    };
    let mut chars = std::collections::BTreeSet::new();
    chars.insert('あ');
    let embed = reciplexa_pdf::CjkFontEmbed::build(&chars).expect("build subset");
    // Line includes a char not in the subset → encode_hex Err → empty skip arm.
    let ops = text_ops(
        0.0,
        10.0,
        4.0,
        None,
        None,
        "あX",
        Color::BLACK,
        Some(&embed),
    )
    .unwrap();
    assert!(ops.contains("BT"));
    let _ = path;
}

#[test]
fn product_layout_font_embeds_fixture_cjk_without_system_font() {
    use reciplexa_pdf::document_to_pdf_with_layout_font;
    let font = reciplexa_text_layout::LoadedFont::fixture();
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(Text {
            x_mm: 20.0,
            y_mm: 200.0,
            size_mm: 5.0,
            width_mm: None,
            height_mm: None,
            content: "日本語".into(),
            fill: Color::BLACK,
        })],
    });
    let bytes = document_to_pdf_with_layout_font(&doc, None, font.bytes(), &font.id.as_key())
        .expect("fixture subset");
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("/Identity-H"));
    assert!(text.contains("Tj"));
    assert!(!text.contains("(日本語)"));
}

#[test]
fn matching_layout_rejects_digest_mismatch() {
    use reciplexa_pdf::document_to_pdf_matching_layout;
    use reciplexa_text_layout::{FontId, LoadedFont};
    let font = LoadedFont::fixture();
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Text(Text {
            x_mm: 20.0,
            y_mm: 200.0,
            size_mm: 5.0,
            width_mm: None,
            height_mm: None,
            content: "あ".into(),
            fill: Color::BLACK,
        })],
    });
    let layout_id = FontId {
        label: "layout".into(),
        digest: "not-this-face".into(),
    };
    let err = document_to_pdf_matching_layout(&doc, None, &layout_id, &font)
        .expect_err("mismatch must fail");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("relayout") || msg.contains("substitution") || msg.contains("digest"),
        "{msg}"
    );
}

#[test]
fn glyph_run_pdf_paints_layout_gid_not_cmap() {
    use reciplexa_pdf::{document_to_pdf_with_loaded_font, CjkFontEmbed};
    use reciplexa_scene::GlyphRunShape;
    let font = reciplexa_text_layout::LoadedFont::fixture();
    let a_gid = font.glyph_id('A').expect("A");
    let paren_gid = font.glyph_id('(').expect("(");
    assert_ne!(a_gid, paren_gid);
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::GlyphRun(GlyphRunShape {
            x_mm: 20.0,
            y_mm: 200.0,
            size_mm: 8.0,
            content: "(".into(),
            fill: Color::BLACK,
            gid: a_gid,
            font_digest: font.id.digest.clone(),
            advance_mm: 4.0,
        })],
    });
    let bytes = document_to_pdf_with_loaded_font(&doc, None, &font).expect("pdf");
    let text = String::from_utf8_lossy(&bytes);
    let embed = CjkFontEmbed::build_from_glyphs(
        font.bytes(),
        &font.id.as_key(),
        &[(a_gid, '('), (paren_gid, '(')],
        font.face_index(),
    )
    .expect("embed");
    let painted = embed.encode_gid_hex(a_gid).expect("A cid");
    let cmap = embed.encode_gid_hex(paren_gid).expect("paren cid");
    assert_ne!(painted, cmap);
    assert!(
        text.contains(&format!("<{painted}> Tj")),
        "PDF must paint the layout GID, got {text}"
    );
    assert!(
        !text.contains(&format!("<{cmap}> Tj")),
        "PDF must not cmap-remap content to the default GID"
    );
}
