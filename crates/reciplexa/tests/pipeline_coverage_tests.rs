//! Pipeline / document_pipeline residual region coverage.

use std::fs;
use std::path::PathBuf;

use reciplexa::document_pipeline::document_snapshot_from_source;
use reciplexa::{
    document_from_source, document_from_source_with_snapshot, expand,
    is_package_shaped_graphics_source, lower, wants_package_graphics_path,
};
use reciplexa_identity::document::DocumentIdentity;

const PKG_CIRCLE: &str = r#"
(import graphics/shapes only circle fill)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main (page a4 (fill (circle 1 2 3) black)))
"#;

fn tmp_file(name: &str, contents: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("rpx_pipeline_cov");
    let _ = fs::create_dir_all(&dir);
    let path = dir.join(name);
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn document_snapshot_ok_and_parse_err() {
    let snap =
        document_snapshot_from_source(PKG_CIRCLE, DocumentIdentity::new(1)).expect("snapshot");
    assert!(snap.nodes.iter().count() > 0);

    let err = document_snapshot_from_source("(unclosed", DocumentIdentity::new(2));
    assert!(err.is_err());

    let refused =
        document_snapshot_from_source("(page a4 (circle 1 2 3))", DocumentIdentity::new(3));
    assert!(refused.is_err());
}

#[test]
fn pipeline_expand_lower_and_document() {
    // Production lower refuses keyword pages (S6b).
    let interim = "(page a4 (circle 1 2 3))";
    let expanded = expand(interim).expect("expand");
    assert!(lower(&expanded).is_err());
    assert!(document_from_source(interim).is_err());

    let via = document_from_source(PKG_CIRCLE).expect("document");
    assert_eq!(via.pages.len(), 1);

    let pipe_doc = document_from_source_with_snapshot(PKG_CIRCLE, true).expect("with snap");
    assert_eq!(pipe_doc.scene.pages.len(), 1);
    assert!(pipe_doc.editable.is_some());
    assert!(pipe_doc.editable.unwrap().nodes.iter().count() > 0);
}

#[test]
fn pipeline_package_graphics_env_and_bad_source() {
    let src = "(import graphics/shapes)\n(val main 1)";
    let expanded = expand(src).expect("expand");
    assert!(!wants_package_graphics_path("(page a4 (circle 1 2 3))"));
    assert!(is_package_shaped_graphics_source(&expanded) || expanded.contains("import"));

    let bad = tmp_file("bad.rpx", "(page a4");
    let bytes = fs::read_to_string(&bad).unwrap();
    assert!(document_from_source(&bytes).is_err());
}
