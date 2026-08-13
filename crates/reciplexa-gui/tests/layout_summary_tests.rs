//! HC10 follow-on: tooltip-like layout summary without GUI chrome changes.

use std::path::PathBuf;

use reciplexa_package::{debug_layout_summary, LocalPackageIndex};

fn index() -> LocalPackageIndex {
    let packages = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    LocalPackageIndex::discover(&[packages]).expect("discover packages/")
}

#[test]
fn debug_layout_summary_tooltip_string_for_package_doc() {
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
    let tip = debug_layout_summary(&source, &index()).expect("tooltip summary");
    // Suitable for a hover/tooltip: short, no newlines, includes line count.
    assert!(!tip.contains('\n'), "tooltip should be one line: {tip}");
    assert!(tip.len() < 120, "tooltip too long: {tip}");
    assert!(
        tip.contains("line(s)") && tip.contains("text shape(s)"),
        "unexpected tip: {tip}"
    );
}
