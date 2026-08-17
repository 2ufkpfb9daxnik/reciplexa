//! llvm-cov tip: graphics_bridge adapter + From/error paths.

use std::io::{Cursor, Read};
use std::path::PathBuf;

use reciplexa_eval::{document_from_graphics_value, RuntimeValue};
use reciplexa_package::{
    debug_layout_summary, document_from_live_layout_entry, document_from_live_layout_source,
    document_from_live_layout_value_with_style, document_from_package_entry,
    document_from_package_source, elaborate_with_packages, preview_doc_text_metrics,
    GraphicsBridgeError, LocalPackageIndex, PackageLoadError,
};
use reciplexa_scene::Shape;
use reciplexa_std::math::EstimateStyle;

fn workspace_packages() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages")
}

fn index() -> LocalPackageIndex {
    LocalPackageIndex::discover(&[workspace_packages()]).expect("discover packages/")
}

fn scratch() -> PathBuf {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.tmp/pkg-bridge-cov");
    let _ = std::fs::create_dir_all(&base);
    let dir = base.join(format!(
        "t-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn graphics_bridge_from_impls() {
    let load: GraphicsBridgeError = PackageLoadError::NotFound("nf".into()).into();
    assert!(matches!(load, GraphicsBridgeError::Load(s) if s.contains("nf")));

    let gve = document_from_graphics_value(&RuntimeValue::Int(42)).unwrap_err();
    let bridge: GraphicsBridgeError = gve.into();
    assert!(matches!(bridge, GraphicsBridgeError::Bridge(_)));
}

#[test]
fn document_from_package_source_happy_path() {
    let idx = index();
    let source = r#"(import graphics/shapes only circle fill)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main
  (page a4
    (fill (circle 105 148.5 40) black)))
"#;
    let doc = document_from_package_source(source, "bridge_src", &idx).expect("happy path");
    match &doc.pages[0].shapes[0] {
        Shape::Circle(c) => assert_eq!(c.radius_mm, 40.0),
        other => panic!("expected circle, got {other:?}"),
    }
}

#[test]
fn document_from_package_entry_load_error_bad_import() {
    let idx = index();
    let dir = scratch();
    let entry = dir.join("bad_import.rpx");
    std::fs::write(
        &entry,
        r#"(import no/such/pkg)
(val main 1)
"#,
    )
    .unwrap();
    let err = document_from_package_entry(&entry, &idx).unwrap_err();
    assert!(matches!(err, GraphicsBridgeError::Load(_)), "{err:?}");
}

#[test]
fn document_from_package_entry_eval_error() {
    let idx = index();
    let dir = scratch();
    let entry = dir.join("eval_err.rpx");
    std::fs::write(
        &entry,
        r#"(import graphics/shapes only circle)
(val main (circle (+ 1 "not-a-number") 2 3))
"#,
    )
    .unwrap();
    let err = document_from_package_entry(&entry, &idx).unwrap_err();
    assert!(
        matches!(
            err,
            GraphicsBridgeError::Eval(_) | GraphicsBridgeError::Load(_)
        ),
        "{err:?}"
    );
    assert!(err.to_string().contains("expected numeric value"));
}

#[test]
fn document_from_package_entry_bridge_error_main_is_int() {
    let idx = index();
    let dir = scratch();
    let entry = dir.join("bridge_err.rpx");
    std::fs::write(&entry, "(val main 42)\n").unwrap();
    let err = document_from_package_entry(&entry, &idx).unwrap_err();
    assert!(matches!(err, GraphicsBridgeError::Bridge(_)), "{err:?}");
}

#[test]
fn document_from_package_entry_missing_elaborated_unit_for_stem() {
    // Directory entry: elaborated units are file stems inside, not the directory name.
    let idx = index();
    let dir = scratch().join("moddir");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.rpx"), "(val x 1)\n").unwrap();
    let err = document_from_package_entry(&dir, &idx).unwrap_err();
    match &err {
        GraphicsBridgeError::Load(s) => {
            assert!(s.contains("missing elaborated unit `moddir`"), "{s}");
        }
        other => panic!("expected Load, got {other:?}"),
    }
}

#[test]
fn document_from_package_source_fs_fail_create_dir() {
    let idx = index();
    reciplexa_package::graphics_bridge::test_set_fail_create_dir(true);
    let err = document_from_package_source("(val main 1)", "fs_fail_dir", &idx);
    reciplexa_package::graphics_bridge::test_set_fail_create_dir(false);
    let err = err.expect_err("create_dir_all fail");
    assert!(matches!(err, GraphicsBridgeError::Load(s) if s.contains("TEST_FAIL_CREATE_DIR")));
}

#[test]
fn document_from_package_source_fs_fail_write() {
    let idx = index();
    reciplexa_package::graphics_bridge::test_set_fail_write(true);
    let err = document_from_package_source("(val main 1)", "fs_fail_write", &idx);
    reciplexa_package::graphics_bridge::test_set_fail_write(false);
    let err = err.expect_err("write fail");
    assert!(matches!(err, GraphicsBridgeError::Load(s) if s.contains("TEST_FAIL_WRITE")));
}

#[test]
fn elaborate_with_packages_populates_interface_exports_for_graphics() {
    // Hits load.rs interface_exports.insert via package import with .rpi boundary.
    let idx = index();
    let dir = scratch();
    let entry = dir.join("iface.rpx");
    std::fs::write(
        &entry,
        r#"(import graphics/shapes only circle)
(val main (circle 1 2 3))
"#,
    )
    .unwrap();
    let units = elaborate_with_packages(&entry, &idx).unwrap();
    assert!(units.iter().any(|u| u.name == "graphics/shapes"));
    assert!(units.iter().any(|u| u.name == "iface"));
}

#[test]
fn document_package_entry_lowers_doc_page() {
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_document.rpx");
    let idx = index();
    let doc = document_from_package_entry(&entry, &idx).expect("pkg_document bridge");
    assert_eq!(doc.pages.len(), 1);
    assert_eq!(doc.pages[0].paper.width_mm, 210.0);
    let texts: Vec<_> = doc.pages[0]
        .shapes
        .iter()
        .filter_map(|s| match s {
            Shape::Text(t) => Some(t.content.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        texts.iter().any(|t| t.contains("Document surface")),
        "expected heading text, got {texts:?}"
    );
    assert!(
        texts.iter().any(|t| t.contains("Native document")),
        "expected paragraph text, got {texts:?}"
    );
}

/// Wave 11 E2: long JA `doc-paragraph` soft-wraps to multiple Text shapes via bridge.
#[test]
fn package_document_long_ja_paragraph_emits_multiple_text_shapes() {
    let long = "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん";
    assert!(long.chars().count() > 40);
    let expected = reciplexa_std::japanese::break_line(long, 40.0);
    assert!(
        expected.len() > 1,
        "fixture must soft-wrap to multiple lines"
    );

    let source = format!(
        r#"(import document/page only
  a4 page flow section heading paragraph
  block-paragraph)
(val title (heading 1 "題"))
(val body (paragraph "{long}"))
(val main
  (page a4
    (flow
      (list
        (section title
          (list
            (block-paragraph body)))))))
"#
    );
    let idx = index();
    let doc = document_from_package_source(&source, "ja_long_para", &idx)
        .expect("long JA paragraph bridge");
    let texts: Vec<_> = doc.pages[0]
        .shapes
        .iter()
        .filter_map(|s| match s {
            Shape::Text(t) => Some(t.content.clone()),
            _ => None,
        })
        .collect();
    // Section title is one Text; wrapped paragraph must appear as consecutive shapes.
    let found = texts
        .windows(expected.len())
        .any(|w| w == expected.as_slice());
    assert!(
        found,
        "expected consecutive break_line lines among shapes {texts:?}, want {expected:?}"
    );
}

/// HC2: `preview_doc_text_metrics` reports multi-line counts for long JA paragraphs.
#[test]
fn preview_doc_text_metrics_long_ja_paragraph() {
    let long = "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん";
    let expected_lines = reciplexa_std::japanese::break_line(long, 40.0);
    assert!(expected_lines.len() > 1);

    let source = format!(
        r#"(import document/page only
  a4 page flow section heading paragraph
  block-paragraph)
(val title (heading 1 "題"))
(val body (paragraph "{long}"))
(val main
  (page a4
    (flow
      (list
        (section title
          (list
            (block-paragraph body)))))))
"#
    );
    let idx = index();
    let m = preview_doc_text_metrics(&source, &idx).expect("preview metrics");
    // Title + wrapped paragraph lines.
    assert!(
        m.line_count > 1 && m.text_shape_count == m.line_count,
        "expected multi-line preview, got {m:?}"
    );
    assert!(m.line_count >= expected_lines.len());
    assert!(m.max_line_width_em > 0.0);
    assert!(m.approx_block_height_em > 0.0);
    assert_eq!(m.ruby_count, 0);
    assert_eq!(m.tate_chu_yoko_count, 0);
    assert!(m.diagnostic_note().contains("line"));
}

/// HC12: preview metrics count ruby / tate nodes when present in the eval tree.
#[test]
fn preview_doc_text_metrics_counts_ruby_and_tate() {
    let source = r#"(import document/page only
  a4 page flow section heading paragraph
  block-paragraph)
(val ruby (ruby-box "漢" "かん"))
(val tate (record (tag "ja-tate-chu-yoko") (body "12")))
(val title (heading 1 "題"))
(val body (paragraph "本文"))
(val main
  (record
    (tag "ja-host-preview")
    (ruby ruby)
    (tate tate)
    (page
      (page a4
        (flow
          (list
            (section title
              (list
                (block-paragraph body)))))))))
"#;
    let idx = index();
    let m = preview_doc_text_metrics(source, &idx).expect("preview with ruby/tate");
    assert_eq!(m.ruby_count, 1, "expected one ruby-box, got {m:?}");
    assert_eq!(
        m.tate_chu_yoko_count, 1,
        "expected one tate node, got {m:?}"
    );
    assert!(m.line_count >= 1);
    assert!(m.diagnostic_note().contains("ruby=1"));
    assert!(m.diagnostic_note().contains("tate-chu-yoko=1"));
}

/// HC10 follow-on: compact layout summary for inspect / GUI tooltips.
#[test]
fn debug_layout_summary_long_ja_paragraph() {
    let long = "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん";
    let source = format!(
        r#"(import document/page only
  a4 page flow section heading paragraph
  block-paragraph)
(val title (heading 1 "題"))
(val body (paragraph "{long}"))
(val main
  (page a4
    (flow
      (list
        (section title
          (list
            (block-paragraph body)))))))
"#
    );
    let summary = debug_layout_summary(&source, &index()).expect("layout summary");
    assert!(
        summary.contains("line(s)") && summary.contains("text shape(s)"),
        "unexpected summary: {summary}"
    );
    let line_n: usize = summary
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    assert!(line_n > 1, "expected multi-line summary, got {summary}");
}

/// HC7: bridged long JA paragraph PDF emits multiple text showing ops (CJK font).
#[test]
fn package_long_ja_paragraph_pdf_emits_multiple_text_ops() {
    if reciplexa_pdf::system_cjk_font_path().is_none() {
        return;
    }
    let long = "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん";
    let expected = reciplexa_std::japanese::break_line(long, 40.0);
    assert!(expected.len() > 1);

    let source = format!(
        r#"(import document/page only
  a4 page flow section heading paragraph
  block-paragraph)
(val title (heading 1 "題"))
(val body (paragraph "{long}"))
(val main
  (page a4
    (flow
      (list
        (section title
          (list
            (block-paragraph body)))))))
"#
    );
    let idx = index();
    let doc = document_from_package_source(&source, "ja_long_pdf", &idx)
        .expect("long JA paragraph bridge");
    let text_shape_count = doc.pages[0]
        .shapes
        .iter()
        .filter(|s| matches!(s, Shape::Text(_)))
        .count();
    assert!(
        text_shape_count > 1,
        "expected multiple Text shapes before PDF, got {text_shape_count}"
    );

    let bytes = reciplexa_pdf::document_to_pdf(&doc).expect("pdf from package scene");
    let pdf = String::from_utf8_lossy(&bytes);
    let tj = pdf.matches(" Tj\n").count();
    assert!(
        tj >= 2,
        "expected multiple PDF text ops from wrapped JA paragraph, got {tj} Tj (shapes={text_shape_count})"
    );
}

/// HC9: bridged long JA paragraph SVG emits multiple `<text` elements.
#[test]
fn package_long_ja_paragraph_svg_emits_multiple_text_elements() {
    let long = "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん";
    let expected = reciplexa_std::japanese::break_line(long, 40.0);
    assert!(expected.len() > 1);

    let source = format!(
        r#"(import document/page only
  a4 page flow section heading paragraph
  block-paragraph)
(val title (heading 1 "題"))
(val body (paragraph "{long}"))
(val main
  (page a4
    (flow
      (list
        (section title
          (list
            (block-paragraph body)))))))
"#
    );
    let idx = index();
    let doc = document_from_package_source(&source, "ja_long_svg", &idx)
        .expect("long JA paragraph bridge");
    let text_shape_count = doc.pages[0]
        .shapes
        .iter()
        .filter(|s| matches!(s, Shape::Text(_)))
        .count();
    assert!(
        text_shape_count > 1,
        "expected multiple Text shapes before SVG, got {text_shape_count}"
    );

    let svg = reciplexa_svg::document_to_svg(&doc).expect("svg from package scene");
    let text_elems = svg.matches("<text").count();
    assert!(
        text_elems >= 2,
        "expected multiple SVG text elements from wrapped JA paragraph, got {text_elems} <text (shapes={text_shape_count})"
    );
}

fn slide1_xml_from_pptx(bytes: &[u8]) -> String {
    let cursor = Cursor::new(bytes);
    let mut archive = zip::read::ZipArchive::new(cursor).expect("valid pptx zip");
    let mut file = archive
        .by_name("ppt/slides/slide1.xml")
        .expect("slide1.xml");
    let mut xml = String::new();
    file.read_to_string(&mut xml).expect("read slide1");
    xml
}

/// HC11: bridged long JA paragraph PPTX emits multiple `<a:t>` text runs.
#[test]
fn package_long_ja_paragraph_pptx_emits_multiple_a_t() {
    let long = "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん";
    let expected = reciplexa_std::japanese::break_line(long, 40.0);
    assert!(expected.len() > 1);

    let source = format!(
        r#"(import document/page only
  a4 page flow section heading paragraph
  block-paragraph)
(val title (heading 1 "題"))
(val body (paragraph "{long}"))
(val main
  (page a4
    (flow
      (list
        (section title
          (list
            (block-paragraph body)))))))
"#
    );
    let idx = index();
    let doc = document_from_package_source(&source, "ja_long_pptx", &idx)
        .expect("long JA paragraph bridge");
    let text_shape_count = doc.pages[0]
        .shapes
        .iter()
        .filter(|s| matches!(s, Shape::Text(_)))
        .count();
    assert!(
        text_shape_count > 1,
        "expected multiple Text shapes before PPTX, got {text_shape_count}"
    );

    let bytes = reciplexa_pptx::document_to_pptx(&doc).expect("pptx from package scene");
    let slide = slide1_xml_from_pptx(&bytes);
    let a_t = slide.matches("<a:t>").count();
    assert!(
        a_t >= 2,
        "expected multiple PPTX text runs from wrapped JA paragraph, got {a_t} <a:t> (shapes={text_shape_count})"
    );
}

/// Wave 24 R1: `columns` / `block-columns` → side-by-side Text via measure_columns.
#[test]
fn package_document_columns_places_side_by_side_texts() {
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_columns.rpx");
    let idx = index();
    let doc = document_from_package_entry(&entry, &idx).expect("pkg_columns bridge");
    let texts: Vec<_> = doc.pages[0]
        .shapes
        .iter()
        .filter_map(|s| match s {
            Shape::Text(t) => Some(t),
            _ => None,
        })
        .collect();
    let left = texts
        .iter()
        .find(|t| t.content == "左カラム本文")
        .expect("left column");
    let right = texts
        .iter()
        .find(|t| t.content == "右カラム本文")
        .expect("right column");
    // total-em 21, gutter 1 → col_w 10, xs [0, 11]; size_mm 4 → Δx = 44mm
    assert!(
        (right.x_mm - left.x_mm - 11.0 * 4.0).abs() < 1e-6,
        "expected Δx=44, left={} right={}",
        left.x_mm,
        right.x_mm
    );
    assert!((left.y_mm - right.y_mm).abs() < 1e-9);
}

/// Wave 15 I2: `paragraph-indented` → first Text x offset via indent-em.
#[test]
fn package_document_indent_em_offsets_first_paragraph_text() {
    let entry =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_document_indent.rpx");
    let idx = index();
    let doc = document_from_package_entry(&entry, &idx).expect("indent example bridge");
    let texts: Vec<_> = doc.pages[0]
        .shapes
        .iter()
        .filter_map(|s| match s {
            Shape::Text(t) => Some(t),
            _ => None,
        })
        .collect();
    let body = texts
        .iter()
        .find(|t| t.content == "インデント付き段落。")
        .expect("indented body text");
    // BASE_X 20 + 1em * paragraph size 4.0 = 24.0
    assert!(
        (body.x_mm - 24.0).abs() < 1e-9,
        "expected indent x=24, got {}",
        body.x_mm
    );
    let title = texts
        .iter()
        .find(|t| t.content == "字下げ")
        .expect("title text");
    assert!((title.x_mm - 20.0).abs() < 1e-9);
}

/// Indent example → PDF smoke (CJK font gated).
#[test]
fn package_document_indent_pdf_smoke() {
    if reciplexa_pdf::system_cjk_font_path().is_none() {
        return;
    }
    let entry =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_document_indent.rpx");
    let doc = document_from_package_entry(&entry, &index()).expect("indent bridge");
    let bytes = reciplexa_pdf::document_to_pdf(&doc).expect("indent pdf");
    let pdf = String::from_utf8_lossy(&bytes);
    let tj = pdf.matches(" Tj\n").count();
    assert!(
        tj >= 1,
        "expected at least one PDF text op from indent example, got {tj}"
    );
}

/// Indent example → SVG smoke (content strings embedded).
#[test]
fn package_document_indent_svg_smoke() {
    let entry =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_document_indent.rpx");
    let doc = document_from_package_entry(&entry, &index()).expect("indent bridge");
    let svg = reciplexa_svg::document_to_svg(&doc).expect("indent svg");
    assert!(
        svg.contains("インデント付き段落。") || svg.contains("<text"),
        "expected indented body in SVG"
    );
    assert!(
        svg.matches("<text").count() >= 2,
        "expected title + body text elements, got {}",
        svg.matches("<text").count()
    );
}

/// Wave 18 follow-on: vertical place → PDF multiple Tj when CJK fonts allow.
#[test]
fn vertical_place_pdf_smoke_when_cjk_font() {
    if reciplexa_pdf::system_cjk_font_path().is_none() {
        return;
    }
    use reciplexa_scene::{Color, Document, Page, PaperSize, Shape};
    use reciplexa_std::japanese::{
        break_line_vertical, lines_to_vertical_text_shapes, KihonHanmen,
    };

    let lines = break_line_vertical("春夏秋冬一二三四", 4.0);
    assert!(lines.len() >= 2, "vertical wrap: {lines:?}");
    let pitch = KihonHanmen::default_vertical().line_pitch_em() * 12.0;
    let texts = lines_to_vertical_text_shapes(&lines, 100.0, 40.0, 12.0, pitch, Color::BLACK);
    let doc = Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: texts.into_iter().map(Shape::Text).collect(),
    });
    let bytes = reciplexa_pdf::document_to_pdf(&doc).expect("vertical place pdf");
    let pdf = String::from_utf8_lossy(&bytes);
    let tj = pdf.matches(" Tj\n").count();
    assert!(
        tj >= 2,
        "expected multiple PDF text ops from vertical columns, got {tj}"
    );
}

/// LL3: doc paragraph + sibling math-box compose onto one scene page.
#[test]
fn live_layout_doc_plus_math_sibling_shapes() {
    let entry =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_live_layout.rpx");
    let doc = document_from_live_layout_entry(&entry, &index()).expect("live layout entry");
    let texts: Vec<_> = doc.pages[0]
        .shapes
        .iter()
        .filter_map(|s| match s {
            Shape::Text(t) => Some(t.content.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        texts
            .iter()
            .any(|t| t.contains("本文") || *t == "本文の下に数式を置く。"),
        "expected doc paragraph among {texts:?}"
    );
    let joined: String = texts.concat();
    assert!(
        joined.contains('a') && joined.contains('b'),
        "expected fraction num/den glyphs among {texts:?}"
    );
    assert!(
        doc.pages[0]
            .shapes
            .iter()
            .any(|s| matches!(s, Shape::Line(_))),
        "expected fraction rule Line on live-layout page"
    );

    // Source path also works (temp entry).
    let source = std::fs::read_to_string(&entry).expect("read example");
    let via_src = document_from_live_layout_source(&source, "pkg_live_layout", &index())
        .expect("live layout source");
    assert!(!via_src.pages[0].shapes.is_empty());
}

/// LL4: live-layout demo → PDF text ops when CJK font available.
#[test]
fn live_layout_pdf_smoke_when_cjk_font() {
    if reciplexa_pdf::system_cjk_font_path().is_none() {
        return;
    }
    let entry =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_live_layout.rpx");
    let doc = document_from_live_layout_entry(&entry, &index()).expect("live layout bridge");
    let text_shape_count = doc.pages[0]
        .shapes
        .iter()
        .filter(|s| matches!(s, Shape::Text(_)))
        .count();
    assert!(
        text_shape_count >= 2,
        "expected doc + math Text shapes before PDF, got {text_shape_count}"
    );
    let bytes = reciplexa_pdf::document_to_pdf(&doc).expect("live layout pdf");
    let pdf = String::from_utf8_lossy(&bytes);
    let tj = pdf.matches(" Tj\n").count();
    assert!(
        tj >= 1,
        "expected ≥1 PDF text op from live-layout page, got {tj} Tj (shapes={text_shape_count})"
    );
}

/// LL6: live-layout demo → SVG with multiple `<text` elements (doc + math).
#[test]
fn live_layout_svg_smoke_multi_text() {
    let entry =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_live_layout.rpx");
    let doc = document_from_live_layout_entry(&entry, &index()).expect("live layout bridge");
    let text_shape_count = doc.pages[0]
        .shapes
        .iter()
        .filter(|s| matches!(s, Shape::Text(_)))
        .count();
    assert!(
        text_shape_count >= 2,
        "expected doc + math Text shapes before SVG, got {text_shape_count}"
    );
    let svg = reciplexa_svg::document_to_svg(&doc).expect("live layout svg");
    let text_elems = svg.matches("<text").count();
    assert!(
        text_elems >= 2,
        "expected multiple SVG text elements from live-layout page, got {text_elems} <text (shapes={text_shape_count})"
    );
}

/// LL6: live-layout demo → PPTX with multiple `<a:t>` runs (doc + math).
#[test]
fn live_layout_pptx_smoke_multi_text() {
    let entry =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_live_layout.rpx");
    let doc = document_from_live_layout_entry(&entry, &index()).expect("live layout bridge");
    let text_shape_count = doc.pages[0]
        .shapes
        .iter()
        .filter(|s| matches!(s, Shape::Text(_)))
        .count();
    assert!(
        text_shape_count >= 2,
        "expected doc + math Text shapes before PPTX, got {text_shape_count}"
    );
    let bytes = reciplexa_pptx::document_to_pptx(&doc).expect("live layout pptx");
    let slide = slide1_xml_from_pptx(&bytes);
    let a_t = slide.matches("<a:t>").count();
    assert!(
        a_t >= 2,
        "expected multiple PPTX text runs from live-layout page, got {a_t} <a:t> (shapes={text_shape_count})"
    );
}

/// LL16: frac + scripts + delimiter live-math example → scene shapes.
#[test]
fn live_math_frac_scripts_delim_shapes() {
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_live_math.rpx");
    let doc = document_from_live_layout_entry(&entry, &index()).expect("live math entry");
    let texts: Vec<_> = doc.pages[0]
        .shapes
        .iter()
        .filter_map(|s| match s {
            Shape::Text(t) => Some(t.content.as_str()),
            _ => None,
        })
        .collect();
    let joined: String = texts.concat();
    assert!(
        texts.iter().any(|t| t.contains('括') || t.contains("括弧")),
        "expected JA paragraph among {texts:?}"
    );
    assert!(
        joined.contains('(')
            && joined.contains(')')
            && joined.contains('a')
            && joined.contains('b'),
        "expected delim + frac glyphs among {texts:?}"
    );
    assert!(
        joined.contains('i'),
        "expected script subscript among {texts:?}"
    );
    assert!(
        doc.pages[0]
            .shapes
            .iter()
            .any(|s| matches!(s, Shape::Line(_))),
        "expected fraction rule Line"
    );
    let left = doc.pages[0].shapes.iter().find_map(|s| match s {
        Shape::Text(t) if t.content == "(" => Some(t),
        _ => None,
    });
    let left = left.expect("left fence Text");
    assert!(
        left.size_mm > 4.0,
        "stretched delimiter fence should be taller than default em, got {}",
        left.size_mm
    );
}

/// LL17: pkg_live_math → PDF text ops when CJK font available.
#[test]
fn live_math_pdf_smoke_when_cjk_font() {
    if reciplexa_pdf::system_cjk_font_path().is_none() {
        return;
    }
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_live_math.rpx");
    let doc = document_from_live_layout_entry(&entry, &index()).expect("live math bridge");
    let text_shape_count = doc.pages[0]
        .shapes
        .iter()
        .filter(|s| matches!(s, Shape::Text(_)))
        .count();
    assert!(
        text_shape_count >= 3,
        "expected doc + delim/frac/script Text shapes, got {text_shape_count}"
    );
    let bytes = reciplexa_pdf::document_to_pdf(&doc).expect("live math pdf");
    let pdf = String::from_utf8_lossy(&bytes);
    let tj = pdf.matches(" Tj\n").count();
    assert!(
        tj >= 1,
        "expected ≥1 PDF text op from live-math page, got {tj} Tj (shapes={text_shape_count})"
    );
}

/// LL23: pkg_live_math → SVG with multiple `<text` elements.
#[test]
fn live_math_svg_smoke_multi_text() {
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_live_math.rpx");
    let doc = document_from_live_layout_entry(&entry, &index()).expect("live math bridge");
    let text_shape_count = doc.pages[0]
        .shapes
        .iter()
        .filter(|s| matches!(s, Shape::Text(_)))
        .count();
    assert!(
        text_shape_count >= 3,
        "expected doc + delim/frac/script Text shapes before SVG, got {text_shape_count}"
    );
    let svg = reciplexa_svg::document_to_svg(&doc).expect("live math svg");
    let text_elems = svg.matches("<text").count();
    assert!(
        text_elems >= 3,
        "expected multiple SVG text elements from live-math page, got {text_elems} <text (shapes={text_shape_count})"
    );
}

/// LL23: pkg_live_math → PPTX with multiple `<a:t>` runs.
#[test]
fn live_math_pptx_smoke_multi_text() {
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_live_math.rpx");
    let doc = document_from_live_layout_entry(&entry, &index()).expect("live math bridge");
    let text_shape_count = doc.pages[0]
        .shapes
        .iter()
        .filter(|s| matches!(s, Shape::Text(_)))
        .count();
    assert!(
        text_shape_count >= 3,
        "expected doc + delim/frac/script Text shapes before PPTX, got {text_shape_count}"
    );
    let bytes = reciplexa_pptx::document_to_pptx(&doc).expect("live math pptx");
    let slide = slide1_xml_from_pptx(&bytes);
    let a_t = slide.matches("<a:t>").count();
    assert!(
        a_t >= 3,
        "expected multiple PPTX text runs from live-math page, got {a_t} <a:t> (shapes={text_shape_count})"
    );
}

/// LL24: pkg_live_layout / pkg_live_math open via package-entry bridge (not only live_* helpers).
#[test]
fn live_layout_demos_open_via_package_entry_bridge() {
    for name in ["pkg_live_layout.rpx", "pkg_live_math.rpx"] {
        let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples")
            .join(name);
        let doc = document_from_package_entry(&entry, &index())
            .unwrap_or_else(|e| panic!("{name} package entry bridge: {e:?}"));
        let text_count = doc.pages[0]
            .shapes
            .iter()
            .filter(|s| matches!(s, Shape::Text(_)))
            .count();
        assert!(
            text_count >= 2,
            "{name}: expected Text shapes via package path, got {text_count}"
        );
        assert!(
            doc.pages[0]
                .shapes
                .iter()
                .any(|s| matches!(s, Shape::Line(_))),
            "{name}: expected fraction Line via package path"
        );
    }
}

fn live_scripts_demo_source(style: &str) -> String {
    format!(
        r#"(import document/page only
  a4 page flow section heading paragraph
  block-paragraph)
(val title (heading 1 "S"))
(val body (paragraph "p"))
(val math
  (record
    (tag "math-row")
    (children
      (list
        (record
          (tag "math-scripts")
          (base (record (tag "math-symbol") (glyph "x") (class "ord")))
          (superscript (record (tag "math-symbol") (glyph "n") (class "ord"))))))))
(val main
  (record
    (tag "live-layout-demo")
    (style "{style}")
    (page
      (page a4
        (flow
          (list
            (section title
              (list
                (block-paragraph body)))))))
    (math math)))
"#
    )
}

fn first_math_glyph_height(doc: &reciplexa_scene::Document) -> f64 {
    doc.pages[0]
        .shapes
        .iter()
        .find_map(|s| match s {
            Shape::Text(t) if t.content == "x" || t.content == "n" => t.height_mm,
            _ => None,
        })
        .expect("row glyph height")
}

/// Host live-layout-demo `style` field reaches `layout_math_to_shapes`.
#[test]
fn live_layout_demo_style_text_vs_display() {
    let display = document_from_live_layout_source(
        &live_scripts_demo_source("display"),
        "live_style_display",
        &index(),
    )
    .expect("display demo");
    let text = document_from_live_layout_source(
        &live_scripts_demo_source("text"),
        "live_style_text",
        &index(),
    )
    .expect("text demo");
    let dh = first_math_glyph_height(&display);
    let th = first_math_glyph_height(&text);
    assert!(
        (dh - th).abs() > 1e-9,
        "live-layout-demo style Text vs Display should change glyph height_mm, {dh} vs {th}"
    );
}

/// Explicit host `layout_style` overrides a nested demo `style` field.
#[test]
fn live_layout_with_style_overrides_record_field() {
    let source = live_scripts_demo_source("display");
    let via_record = document_from_live_layout_source(&source, "live_style_override", &index())
        .expect("record display");
    let display_h = first_math_glyph_height(&via_record);

    let dir = scratch();
    let entry = dir.join("live_style_force.rpx");
    std::fs::write(&entry, &source).unwrap();
    let v = reciplexa_package::eval_package_entry_main(&entry, &index()).unwrap();
    let forced =
        document_from_live_layout_value_with_style(&v, EstimateStyle::Text).expect("forced text");
    let th = first_math_glyph_height(&forced);
    assert!(
        (display_h - th).abs() > 1e-9,
        "with_style(Text) should ignore demo style display: {display_h} vs {th}"
    );
}

fn live_math_phantom_entry() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_live_math_phantom.rpx")
}

fn page_text_contents(doc: &reciplexa_scene::Document) -> Vec<String> {
    doc.pages[0]
        .shapes
        .iter()
        .filter_map(|s| match s {
            Shape::Text(t) => Some(t.content.clone()),
            _ => None,
        })
        .collect()
}

/// Live-layout phantom example: visible glyph inks; builtin math-phantom does not.
#[test]
fn live_math_phantom_skips_sigma_ink() {
    let doc = document_from_live_layout_entry(live_math_phantom_entry(), &index())
        .expect("live math phantom entry");
    let texts = page_text_contents(&doc);
    let joined: String = texts.concat();
    assert!(
        texts.iter().any(|t| t == "x"),
        "expected visible math glyph among {texts:?}"
    );
    assert!(
        texts.iter().all(|t| t != "Σ"),
        "math-phantom Σ must not ink as its own glyph: {texts:?}"
    );
    assert!(
        joined.contains("可視") || joined.contains("本文"),
        "expected JA paragraph among {texts:?}"
    );
}

/// Package-entry bridge (GUI path) also evals ker `math-phantom` via primitive_env.
#[test]
fn live_math_phantom_opens_via_package_entry_bridge() {
    let doc = document_from_package_entry(live_math_phantom_entry(), &index())
        .expect("phantom package entry");
    let texts = page_text_contents(&doc);
    assert!(
        texts.iter().any(|t| t == "x"),
        "expected visible glyph via package path: {texts:?}"
    );
    assert!(
        texts.iter().all(|t| t != "Σ"),
        "package path must skip phantom Σ: {texts:?}"
    );
}

/// pkg_live_math_phantom → SVG has paragraph/glyph text but not phantom Σ.
#[test]
fn live_math_phantom_svg_smoke_no_sigma() {
    let doc = document_from_live_layout_entry(live_math_phantom_entry(), &index())
        .expect("live math phantom bridge");
    let svg = reciplexa_svg::document_to_svg(&doc).expect("phantom svg");
    assert!(
        svg.matches("<text").count() >= 2,
        "expected paragraph + visible glyph <text, got {}",
        svg.matches("<text").count()
    );
    assert!(
        svg.contains(">x<") || svg.contains(">x</text"),
        "SVG should include visible glyph x"
    );
    assert!(
        !svg.contains(">Σ<") && !svg.contains(">Σ</text"),
        "SVG must not contain phantom Σ as its own text run"
    );
}

/// pkg_live_math_phantom → PPTX has paragraph/glyph runs but not phantom Σ.
#[test]
fn live_math_phantom_pptx_smoke_no_sigma() {
    let doc = document_from_live_layout_entry(live_math_phantom_entry(), &index())
        .expect("live math phantom bridge");
    let bytes = reciplexa_pptx::document_to_pptx(&doc).expect("phantom pptx");
    let slide = slide1_xml_from_pptx(&bytes);
    assert!(
        slide.matches("<a:t>").count() >= 2,
        "expected paragraph + visible glyph <a:t>, got {}",
        slide.matches("<a:t>").count()
    );
    assert!(
        slide.contains("<a:t>x</a:t>"),
        "PPTX should include visible glyph x"
    );
    assert!(
        !slide.contains("<a:t>Σ</a:t>"),
        "PPTX must not contain phantom Σ as its own run"
    );
}
