//! HC4: `estimate_package_math_main` over `examples/pkg_math.rpx`.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use reciplexa_eval::estimate_math_box_from_value_with_style;
use reciplexa_package::{
    elaborate_with_packages, estimate_package_math_main, estimate_package_math_main_with_style,
    LocalPackageIndex,
};
use reciplexa_std::math::EstimateStyle;

fn workspace_packages() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages")
}

#[test]
fn estimate_package_math_main_pkg_math_demo_tree() {
    std::thread::Builder::new()
        .name("math-main-box".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
            let entry =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_math.rpx");
            let box_ = estimate_package_math_main(&entry, &idx).expect("math main box");
            assert!(
                box_.width > 0.0 && box_.total_height() > 0.0,
                "expected non-empty MathBox, got {box_:?}"
            );
        })
        .expect("spawn")
        .join()
        .expect("join");
}

/// Host package main always estimates with Display, even if the tree says style text.
#[test]
fn estimate_package_math_main_forces_display_style() {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-math-display-{}-{seq}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let entry = dir.join("math_display_main.rpx");
    // Nested style "text" would shrink scripts; host main must still use Display.
    std::fs::write(
        &entry,
        r#"(val main
  (record
    (tag "math-scripts")
    (style "text")
    (base (record (tag "math-symbol") (glyph "x") (class "ord")))
    (subscript (record (tag "math-symbol") (glyph "i") (class "ord")))
    (superscript (record (tag "math-symbol") (glyph "n") (class "ord")))))
"#,
    )
    .unwrap();
    let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
    let host = estimate_package_math_main(&entry, &idx).expect("host display box");
    let units = elaborate_with_packages(&entry, &idx).unwrap();
    let demo = units
        .iter()
        .find(|u| u.name == "math_display_main")
        .unwrap();
    let v = reciplexa_eval::eval_expr(
        &demo.expr,
        &reciplexa_eval::primitive_env(),
        &mut reciplexa_eval::UnitHost,
    )
    .unwrap();
    let text = estimate_math_box_from_value_with_style(&v, EstimateStyle::Text).unwrap();
    let display = estimate_math_box_from_value_with_style(&v, EstimateStyle::Display).unwrap();
    assert!(
        (host.width - display.width).abs() < 1e-9,
        "host should match Display: host={host:?} display={display:?}"
    );
    assert!(
        (display.width - text.width).abs() > 1e-9
            || (display.total_height() - text.total_height()).abs() > 1e-9,
        "Display and Text should differ for scripts: display={display:?} text={text:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// Hosts that want inline estimates pass Text; default helper stays Display.
#[test]
fn estimate_package_math_main_with_style_text_shrinks_scripts() {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir =
        std::env::temp_dir().join(format!("reciplexa-math-style-{}-{seq}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let entry = dir.join("math_style_main.rpx");
    std::fs::write(
        &entry,
        r#"(val main
  (record
    (tag "math-scripts")
    (style "display")
    (base (record (tag "math-symbol") (glyph "x") (class "ord")))
    (subscript (record (tag "math-symbol") (glyph "i") (class "ord")))
    (superscript (record (tag "math-symbol") (glyph "n") (class "ord")))))
"#,
    )
    .unwrap();
    let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
    let display = estimate_package_math_main(&entry, &idx).expect("display default");
    let text =
        estimate_package_math_main_with_style(&entry, &idx, EstimateStyle::Text).expect("text");
    let forced_display =
        estimate_package_math_main_with_style(&entry, &idx, EstimateStyle::Display)
            .expect("forced display");
    assert!(
        (display.width - forced_display.width).abs() < 1e-9,
        "with_style(Display) matches default: {display:?} vs {forced_display:?}"
    );
    assert!(
        text.width < display.width,
        "Text style scripts should be narrower: text={text:?} display={display:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
