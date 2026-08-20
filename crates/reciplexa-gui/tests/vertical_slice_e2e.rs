//! Vertical Slice: package-shaped page with graphics/text, edit, save, reload, export.

use reciplexa::pipeline::{document_for_export, document_from_source};
use reciplexa_effect::TestHandler;
use reciplexa_lower::{
    collect_layers_authoring, collect_layers_package, collect_size_targets_package,
    delete_layer_package, insert_layer_package, insert_layer_page, nudge_layer_package,
    reorder_layer_package, set_layer_prop, set_text_content_authoring, set_text_content_package,
    PropEditContext, PropValue,
};
use reciplexa_pdf::document_to_pdf;
use reciplexa_scene::Shape;
use reciplexa_svg::document_to_svg;

const TEXT_LINE: &str = include_str!("../../../examples/text_line.rpx");

fn shape_kind(s: &Shape) -> Vec<&'static str> {
    match s {
        Shape::Text(_) | Shape::GlyphRun(_) => vec!["text"],
        Shape::Line(_) => vec!["line"],
        Shape::Circle(_) => vec!["circle"],
        Shape::Group { children, .. } | Shape::Opacity { children, .. } => {
            children.iter().flat_map(shape_kind).collect()
        }
        _ => vec!["other"],
    }
}

const PKG_DOCUMENT_INDENT: &str = include_str!("../../../examples/pkg_document_indent.rpx");
const PKG_COLUMNS: &str = include_str!("../../../examples/pkg_columns.rpx");

fn text_contents(shapes: &[Shape]) -> Vec<String> {
    let mut out = Vec::new();
    for s in shapes {
        match s {
            Shape::Text(t) => out.push(t.content.clone()),
            Shape::GlyphRun(g) => out.push(g.content.clone()),
            Shape::Group { children, .. } | Shape::Opacity { children, .. } => {
                out.extend(text_contents(children));
            }
            _ => {}
        }
    }
    out
}

#[test]
fn vertical_slice_package_shapes_text_roundtrip() {
    let src0 = TEXT_LINE;
    let doc0 = document_from_source(src0).expect("initial ingest");
    assert_eq!(doc0.pages.len(), 1);
    let kinds: Vec<&str> = doc0.pages[0].shapes.iter().flat_map(shape_kind).collect();
    assert!(kinds.contains(&"text"), "kinds={kinds:?}");
    assert!(kinds.contains(&"line"), "kinds={kinds:?}");
    assert!(kinds.contains(&"circle"), "kinds={kinds:?}");

    let layers = collect_layers_package(src0, 0).expect("layers");
    let text_idx = layers.iter().position(|l| l.kind == "text").expect("text");
    let circle_idx = layers
        .iter()
        .position(|l| l.kind == "circle")
        .expect("circle");

    let src1 = nudge_layer_package(src0, 0, text_idx, 5.0, -3.0).expect("nudge text");
    let targets = collect_size_targets_package(&src1, 0).expect("size targets");
    let src2 =
        reciplexa_lower::scale_size_target(&src1, targets[circle_idx], 1.5).expect("scale circle");
    let src3 = set_text_content_package(&src2, 0, text_idx, "Edited").expect("rename text");
    assert!(src3.contains("\"Edited\""));
    assert!(!src3.contains("\"Reciplexa\""));

    let (src4, inserted) =
        insert_layer_package(&src3, 0, "(fill (circle 40 80 10) (rgb 0.2 0.4 0.6))")
            .expect("insert");
    assert!(src4.contains("(circle 40 80 10)"));
    let src5 = reorder_layer_package(&src4, 0, inserted, 0).expect("reorder");
    let src6 = delete_layer_package(&src5, 0, 0).expect("delete inserted");
    assert!(!src6.contains("(circle 40 80 10)"));

    let dir = std::env::temp_dir().join(format!("reciplexa-vslice-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("edited.rpx");
    std::fs::write(&path, &src6).expect("save");
    let reloaded = std::fs::read_to_string(&path).expect("reload");
    assert_eq!(reloaded, src6);

    let doc_reload = document_from_source(&reloaded).expect("reload ingest");
    let texts = text_contents(&doc_reload.pages[0].shapes);
    assert!(
        texts.iter().any(|t| t.contains("Edited")),
        "texts={texts:?}"
    );

    let (doc_export, _) =
        document_for_export(&mut TestHandler::default(), &reloaded).expect("export");
    let pdf = document_to_pdf(&doc_export).expect("pdf");
    assert!(pdf.starts_with(b"%PDF-"));
    let svg = document_to_svg(&doc_export).expect("svg");
    assert!(svg.contains("Edited"));
    assert!(svg.contains("<circle") || svg.contains("ellipse") || svg.contains("cx="));

    assert!(insert_layer_page(&src6, 0, "(circle 1 2 3)").is_err());
}

fn save_reload_roundtrip(src: &str, stem: &str) -> String {
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-doc-page-{}-{}",
        std::process::id(),
        stem
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join(format!("{stem}.rpx"));
    std::fs::write(&path, src).expect("save");
    let reloaded = std::fs::read_to_string(&path).expect("reload");
    assert_eq!(reloaded, src);
    reloaded
}

#[test]
fn document_page_paragraph_roundtrip_export() {
    let src0 = PKG_DOCUMENT_INDENT;
    let layers = collect_layers_authoring(src0, 0).expect("document layers");
    let body_idx = layers
        .iter()
        .position(|l| l.kind == "doc-paragraph")
        .expect("paragraph layer");
    let src1 = set_text_content_authoring(src0, 0, body_idx, "編集後の段落。").expect("edit body");
    assert!(src1.contains("編集後の段落。"));

    let prop_ctx = PropEditContext {
        aabb_mm: (0.0, 0.0, 100.0, 20.0),
        paper_w_mm: 210.0,
        paper_h_mm: 297.0,
    };
    let src2 = set_layer_prop(
        &src1,
        0,
        body_idx,
        "layout.indent-em",
        &PropValue::Number(2.0),
        &prop_ctx,
    )
    .expect("indent edit");
    assert!(src2.contains("(paragraph-indented \"編集後の段落。\" 2)"));

    let reloaded = save_reload_roundtrip(&src2, "indent");
    let doc = document_from_source(&reloaded).expect("ingest");
    let texts = text_contents(&doc.pages[0].shapes);
    let joined: String = texts.concat();
    assert!(joined.contains("編集後の段落"), "joined={joined:?}");

    let (doc_export, _) =
        document_for_export(&mut TestHandler::default(), &reloaded).expect("export");
    let export_joined = text_contents(&doc_export.pages[0].shapes).concat();
    assert!(
        export_joined.contains("編集後の段落"),
        "export_joined={export_joined:?}"
    );
    let pdf = document_to_pdf(&doc_export).expect("pdf");
    assert!(pdf.starts_with(b"%PDF-"));
    let svg = document_to_svg(&doc_export).expect("svg");
    assert!(svg.contains("<svg"));
}

#[test]
fn document_page_columns_roundtrip_export() {
    let layers = collect_layers_authoring(PKG_COLUMNS, 0).expect("layers");
    let cols_idx = layers
        .iter()
        .position(|l| l.kind == "doc-columns")
        .expect("columns layer");
    let left_idx = layers
        .iter()
        .position(|l| l.label.contains("左カラム"))
        .expect("left column");

    let prop_ctx = PropEditContext {
        aabb_mm: (0.0, 0.0, 100.0, 20.0),
        paper_w_mm: 210.0,
        paper_h_mm: 297.0,
    };
    let src1 = set_layer_prop(
        PKG_COLUMNS,
        0,
        cols_idx,
        "columns.gutter",
        &PropValue::Number(2.0),
        &prop_ctx,
    )
    .expect("gutter edit");
    assert!(src1.contains("(columns 2 2 21"));

    let src2 = set_text_content_authoring(&src1, 0, left_idx, "左を更新").expect("edit left");
    let reloaded = save_reload_roundtrip(&src2, "columns");
    let doc = document_from_source(&reloaded).expect("ingest");
    let joined: String = text_contents(&doc.pages[0].shapes).concat();
    assert!(joined.contains("左を更新"), "joined={joined:?}");

    let (doc_export, _) =
        document_for_export(&mut TestHandler::default(), &reloaded).expect("export");
    let export_joined = text_contents(&doc_export.pages[0].shapes).concat();
    assert!(export_joined.contains("左を更新"));
    let pdf = document_to_pdf(&doc_export).expect("pdf");
    assert!(pdf.starts_with(b"%PDF-"));
    let svg = document_to_svg(&doc_export).expect("svg");
    assert!(svg.contains("<svg"));
}

#[test]
fn document_page_columns_text_roundtrip() {
    let layers = collect_layers_authoring(PKG_COLUMNS, 0).expect("layers");
    let left_idx = layers
        .iter()
        .position(|l| l.label.contains("左カラム"))
        .expect("left column");
    let src1 = set_text_content_authoring(PKG_COLUMNS, 0, left_idx, "左を更新").expect("edit left");
    let doc = document_from_source(&src1).expect("ingest");
    let joined: String = text_contents(&doc.pages[0].shapes).concat();
    assert!(joined.contains("左を更新"), "joined={joined:?}");
}

#[test]
fn document_page_canvas_nudge_soft_refuses() {
    assert!(reciplexa_gui::canvas_sync::nudge_authoring_layers(
        PKG_DOCUMENT_INDENT,
        PKG_DOCUMENT_INDENT,
        0,
        &[0],
        1.0,
        0.0
    )
    .is_err());
}

#[test]
fn text_line_ingest_on_host_stack() {
    reciplexa::run_on_host_stack("vslice-host-stack", || {
        document_from_source(TEXT_LINE).expect("ingest");
    })
    .expect("join");
}

#[test]
fn markup_authoring_still_soft_refuses_nudge() {
    let authoring = include_str!("../../../examples/markup_ja.rpx");
    let expanded = reciplexa_macro::expand_source(authoring).expect("expand markup");
    assert!(!reciplexa_gui::canvas_sync::authoring_layers_align(authoring, &expanded, 0).unwrap());
    assert!(reciplexa_gui::canvas_sync::nudge_authoring_layers(
        authoring,
        &expanded,
        0,
        &[0],
        1.0,
        0.0
    )
    .is_err());
    let (doc, _) = document_for_export(&mut TestHandler::default(), authoring).expect("export ok");
    assert!(!doc.pages.is_empty());
}
