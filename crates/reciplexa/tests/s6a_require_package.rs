//! S6b: production pipeline always refuses interim top-level `(page …)`.

use reciplexa::{document_from_source, refuse_interim_if_required};

#[test]
fn production_pipeline_rejects_interim_page() {
    let interim = "(page a4 (circle 1 2 3))";
    let err = refuse_interim_if_required(interim).expect_err("always refuse");
    assert_eq!(err.stage, "package");
    assert!(err.message.contains("retired") || err.message.contains("import graphics"));
    let doc_err = document_from_source(interim).expect_err("ingest blocked");
    assert_eq!(doc_err.stage, "package");
    let pkg = include_str!("../../../examples/black_circle.rpx");
    assert!(document_from_source(pkg).is_ok());
}
