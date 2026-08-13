//! N5.3 tip: document import + markup package emit + export path.

use reciplexa::{
    document_for_export, document_from_source, document_from_source_with_snapshot, expand,
    wants_package_graphics_path,
};
use reciplexa_effect::TestHandler;

#[test]
fn n5_3_document_import_and_markup_package_paths() {
    let pkg_doc = include_str!("../../../examples/pkg_document.rpx");
    let expanded = expand(pkg_doc).unwrap();
    assert!(wants_package_graphics_path(&expanded));
    let doc = document_from_source(pkg_doc).expect("pkg_document");
    assert_eq!(doc.pages.len(), 1);
    let snap = document_from_source_with_snapshot(pkg_doc, false).expect("snapshot skip");
    assert!(snap.editable.is_none());
    let mut h = TestHandler::default();
    let (exported, _) = document_for_export(&mut h, pkg_doc).expect("export");
    assert_eq!(exported.pages.len(), 1);

    let markup = "(markup @title{N5 tip}\n@p{Body}\n@hr{}\n)";
    let mexp = expand(markup).unwrap();
    assert!(wants_package_graphics_path(&mexp));
    assert!(mexp.contains("(import graphics"));
    assert!(mexp.contains("(stroke (line"));
    let mdoc = document_from_source(markup).expect("markup package bridge");
    assert!(!mdoc.pages[0].shapes.is_empty());

    // Interim golden stays off package path.
    let interim = include_str!("../../../examples/black_circle.rpx");
    assert!(!wants_package_graphics_path(&expand(interim).unwrap()));
}
