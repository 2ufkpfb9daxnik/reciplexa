use std::path::PathBuf;

use reciplexa_core::elaborate::elaborate_source;
use reciplexa_core::expr::CoreExpr;
use reciplexa_package::{
    differential_eval_package_modules, register_test_ping_module, std_domain_natives,
    DomainNativeRegistry, LocalPackageIndex,
};

#[test]
fn dn2_stub_elaborates_to_var_ref() {
    let mut reg = DomainNativeRegistry::empty();
    register_test_ping_module(&mut reg);
    let stub = reg.get("native/test").unwrap().effective_source();
    let expr = elaborate_source(&stub).expect("stub elaborates");
    let ping = collect_bindings(&expr).get("ping").cloned().expect("ping");
    assert!(!matches!(ping, CoreExpr::Error), "ping={ping:?}");
}

#[test]
fn dn2_single_unit_via_elaborate_units() {
    let mut reg = DomainNativeRegistry::empty();
    register_test_ping_module(&mut reg);
    let stub = reg.get("native/test").unwrap().effective_source();
    let units = reciplexa_bind::elaborate_units(&[("native/test", stub.as_str())]).unwrap();
    let ping = collect_bindings(&units[0].expr)
        .get("ping")
        .cloned()
        .expect("ping");
    assert!(
        !matches!(ping, CoreExpr::Error),
        "stub={stub:?} ping={ping:?}"
    );
}

#[test]
fn dn2_test_ping_module_hybrid_and_direct_native_agree() {
    let mut reg = DomainNativeRegistry::empty();
    register_test_ping_module(&mut reg);
    let idx = LocalPackageIndex::default().with_native(reg);
    let src = "(import native/test only ping)\n(val main ping)\n";
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-dn2-ping-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let entry = dir.join("main.rpx");
    std::fs::write(&entry, src).unwrap();
    differential_eval_package_modules(&entry, &idx, &["native/test"])
        .expect("hybrid and DN2 agree");
}

#[test]
fn dn2_length_units_hybrid_and_direct_native_agree_on_pkg_length() {
    let packages = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_length.rpx");
    if !entry.is_file() {
        return;
    }
    let idx = LocalPackageIndex::discover(&[packages.as_path()]).unwrap();
    differential_eval_package_modules(&entry, &idx, &["length/units"])
        .expect("length/units hybrid and DN2 agree on pkg_length");
}

#[test]
fn dn2_length_units_module_has_typed_exports_for_all_rpi_names() {
    use reciplexa_package::length_units_module;
    let m = length_units_module();
    assert_eq!(m.typed_exports.len(), m.exports.len());
    for name in &m.exports {
        assert!(
            m.typed_exports.contains_key(name),
            "missing typed export `{name}`"
        );
    }
}

#[test]
fn dn2_color_srgb_hybrid_and_direct_native_agree_on_pkg_color() {
    let packages = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_color.rpx");
    if !entry.is_file() {
        return;
    }
    let idx = LocalPackageIndex::discover(&[packages.as_path()]).unwrap();
    differential_eval_package_modules(&entry, &idx, &["color/srgb"])
        .expect("color/srgb hybrid and DN2 agree on pkg_color");
}

#[test]
fn dn2_graphics_shapes_hybrid_and_direct_native_agree_on_pkg_circle() {
    let packages = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_circle.rpx");
    if !entry.is_file() {
        return;
    }
    let idx = LocalPackageIndex::discover(&[packages.as_path()]).unwrap();
    differential_eval_package_modules(&entry, &idx, &["graphics/shapes"])
        .expect("graphics/shapes hybrid and DN2 agree on pkg_circle");
}

#[test]
fn dn2_graphics_modules_hybrid_and_direct_native_agree_on_pkg_graphics_static() {
    let packages = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    let entry =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_graphics_static.rpx");
    if !entry.is_file() {
        return;
    }
    let idx = LocalPackageIndex::discover(&[packages.as_path()]).unwrap();
    differential_eval_package_modules(
        &entry,
        &idx,
        &["graphics/color", "graphics/page", "graphics/shapes"],
    )
    .expect("graphics modules hybrid and DN2 agree on pkg_graphics_static");
}

#[test]
fn dn2_pure_constructor_modules_have_typed_exports_for_all_rpi_names() {
    use reciplexa_package::{
        color_srgb_module, graphics_color_module, graphics_page_module, graphics_shapes_module,
    };
    for m in [
        color_srgb_module(),
        graphics_color_module(),
        graphics_page_module(),
        graphics_shapes_module(),
    ] {
        assert_eq!(
            m.typed_exports.len(),
            m.exports.len(),
            "module {}",
            m.module_path
        );
        for name in &m.exports {
            assert!(
                m.typed_exports.contains_key(name),
                "module {} missing typed export `{name}`",
                m.module_path
            );
        }
    }
}

#[test]
fn dn2_math_atoms_hybrid_and_direct_native_agree_on_pkg_math_spacing() {
    let packages = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    let entry =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_math_spacing.rpx");
    if !entry.is_file() {
        return;
    }
    let idx = LocalPackageIndex::discover(&[packages.as_path()]).unwrap();
    differential_eval_package_modules(&entry, &idx, &["math/atoms"])
        .expect("math/atoms hybrid and DN2 agree on pkg_math_spacing");
}

#[test]
fn dn2_math_modules_hybrid_and_direct_native_agree_on_smoke_consumer() {
    let idx = LocalPackageIndex::default().with_native(reciplexa_package::std_domain_natives());
    let src = r#"(import math/atoms only ord)
(import math/scripts only superscript)
(import math/frac only fraction)
(import math/sqrt only sqrt)
(import math/delimiters only paren)
(import math/matrix only matrix-row)
(import math/accents only hat)
(import math/bigops only sum)
(import math/cases only case-arm)
(import math/align only align-row)
(import math/stack only stack)
(val x (ord "x"))
(val main
  (record (tag "dn2-math-smoke")
    (scripts (superscript x x))
    (frac (fraction x x))
    (rad (sqrt x))
    (delim (paren x))
    (row (matrix-row (list x x)))
    (accent (hat x))
    (op (sum x x x))
    (arm (case-arm x x))
    (align (align-row (list x)))
    (stk (stack (list x)))))
"#;
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-dn2-math-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let entry = dir.join("main.rpx");
    std::fs::write(&entry, src).unwrap();
    differential_eval_package_modules(
        &entry,
        &idx,
        &[
            "math/atoms",
            "math/scripts",
            "math/frac",
            "math/sqrt",
            "math/delimiters",
            "math/matrix",
            "math/accents",
            "math/bigops",
            "math/cases",
            "math/align",
            "math/stack",
        ],
    )
    .expect("math modules hybrid and DN2 agree on smoke consumer");
}

#[test]
fn dn2_math_modules_have_typed_exports_for_all_rpi_names() {
    use reciplexa_package::{
        math_accents_module, math_align_module, math_atoms_module, math_bigops_module,
        math_cases_module, math_delimiters_module, math_frac_module, math_matrix_module,
        math_scripts_module, math_sqrt_module, math_stack_module,
    };
    for m in [
        math_atoms_module(),
        math_scripts_module(),
        math_frac_module(),
        math_sqrt_module(),
        math_delimiters_module(),
        math_matrix_module(),
        math_accents_module(),
        math_bigops_module(),
        math_cases_module(),
        math_align_module(),
        math_stack_module(),
    ] {
        assert_eq!(
            m.typed_exports.len(),
            m.exports.len(),
            "module {}",
            m.module_path
        );
        for name in &m.exports {
            assert!(
                m.typed_exports.contains_key(name),
                "module {} missing typed export `{name}`",
                m.module_path
            );
        }
    }
}

#[test]
fn dn2_japanese_classes_kihon_markup_agree_on_smoke_consumer() {
    let idx = LocalPackageIndex::default().with_native(reciplexa_package::std_domain_natives());
    let src = r#"(import japanese/classes only cl-19 class-name)
(import japanese/kihon only a5-trim default-horizontal-kihon)
(import japanese/markup only heading paragraph doc)
(val main
  (record (tag "ja-smoke")
    (cl cl-19)
    (name (class-name 19))
    (trim a5-trim)
    (kihon default-horizontal-kihon)
    (doc (doc "title" (list (heading "h") (paragraph "p"))))))
"#;
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-dn2-ja-smoke-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let entry = dir.join("main.rpx");
    std::fs::write(&entry, src).unwrap();
    differential_eval_package_modules(
        &entry,
        &idx,
        &["japanese/classes", "japanese/kihon", "japanese/markup"],
    )
    .expect("japanese classes/kihon/markup hybrid and DN2 agree on smoke consumer");
}

#[test]
fn dn2_japanese_linebreak_static_exports_hybrid_and_direct_native_agree() {
    let idx = LocalPackageIndex::default().with_native(reciplexa_package::std_domain_natives());
    let src = r#"(import japanese/linebreak only kinsoku-profile sample-line-head-prohibited)
(val main (record (profile kinsoku-profile) (head sample-line-head-prohibited)))
"#;
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-dn2-ja-lb-static-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let entry = dir.join("main.rpx");
    std::fs::write(&entry, src).unwrap();
    differential_eval_package_modules(&entry, &idx, &["japanese/linebreak"])
        .expect("japanese/linebreak static exports hybrid and DN2 agree");
}

#[test]
fn dn2_japanese_linebreak_std_parity_via_domain_native() {
    use reciplexa_eval::{call_domain_native, domain_native::JapaneseLinebreakOp, DomainNativeOp};
    use reciplexa_package::linebreak_parity_samples;
    use reciplexa_std::japanese::{classify_char, BreakOpportunity};

    for sample in linebreak_parity_samples() {
        let prev_id = classify_char(sample.prev).id() as i128;
        let next_id = classify_char(sample.next).id() as i128;
        let got = call_domain_native(
            DomainNativeOp::JapaneseLinebreak(JapaneseLinebreakOp::BreakBetween),
            &[
                reciplexa_eval::RuntimeValue::Int(prev_id),
                reciplexa_eval::RuntimeValue::Int(next_id),
            ],
        )
        .expect("break-between");
        let reciplexa_eval::RuntimeValue::Record(fields) = got else {
            panic!("expected record for {}→{}", sample.prev, sample.next);
        };
        let kind = fields
            .iter()
            .find(|(k, _)| k == "kind")
            .and_then(|(_, v)| match v {
                reciplexa_eval::RuntimeValue::String(s) => Some(s.as_str()),
                _ => None,
            })
            .unwrap_or("");
        assert_eq!(
            kind, sample.expected,
            "break-between parity {}→{} ({})",
            sample.prev, sample.next, sample.note
        );

        let glyph = sample.prev.to_string();
        let class = call_domain_native(
            DomainNativeOp::JapaneseLinebreak(JapaneseLinebreakOp::ClassifySample),
            &[reciplexa_eval::RuntimeValue::String(glyph)],
        )
        .expect("classify-sample");
        let reciplexa_eval::RuntimeValue::Int(id) = class else {
            panic!("classify-sample should return Int");
        };
        assert_eq!(id, prev_id, "classify-sample parity for {}", sample.prev);
    }

    // Sanity: std matrix matches itself for a known inseparable pair.
    assert_eq!(BreakOpportunity::Inseparable.as_str(), "inseparable");
}

#[test]
fn dn2_japanese_modules_have_typed_exports_for_all_rpi_names() {
    use reciplexa_package::{
        japanese_classes_module, japanese_kihon_module, japanese_linebreak_module,
        japanese_markup_module,
    };
    for m in [
        japanese_classes_module(),
        japanese_linebreak_module(),
        japanese_kihon_module(),
        japanese_markup_module(),
    ] {
        assert_eq!(
            m.typed_exports.len(),
            m.exports.len(),
            "module {}",
            m.module_path
        );
        for name in &m.exports {
            assert!(
                m.typed_exports.contains_key(name),
                "module {} missing typed export `{name}`",
                m.module_path
            );
        }
    }
}

#[test]
fn dn2_document_page_hybrid_and_direct_native_agree_on_pkg_document() {
    let packages = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_document.rpx");
    if !entry.is_file() {
        return;
    }
    let idx = LocalPackageIndex::discover(&[packages.as_path()]).unwrap();
    differential_eval_package_modules(&entry, &idx, &["document/page"])
        .expect("document/page hybrid and DN2 agree on pkg_document");
}

#[test]
fn dn2_document_page_hybrid_and_direct_native_agree_on_pkg_document_indent() {
    let packages = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    let entry =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_document_indent.rpx");
    if !entry.is_file() {
        return;
    }
    let idx = LocalPackageIndex::discover(&[packages.as_path()]).unwrap();
    differential_eval_package_modules(&entry, &idx, &["document/page"])
        .expect("document/page hybrid and DN2 agree on pkg_document_indent");
}

#[test]
fn dn2_document_page_module_has_typed_exports_for_all_rpi_names() {
    use reciplexa_package::document_page_module;
    let m = document_page_module();
    assert_eq!(m.typed_exports.len(), m.exports.len());
    for name in &m.exports {
        assert!(
            m.typed_exports.contains_key(name),
            "missing typed export `{name}`"
        );
    }
}

#[test]
fn dn2_std_modules_use_stub_not_synthetic_fallback() {
    let reg = std_domain_natives();
    for module in reg.modules() {
        assert_eq!(
            module.typed_exports.len(),
            module.exports.len(),
            "module {}",
            module.module_path
        );
        let source = reg.module_source(&module.module_path, module);
        assert!(
            source.contains("dn2slot-"),
            "module {} should load DN2 stub, got: {source}",
            module.module_path
        );
        assert!(
            !source.contains("native:"),
            "module {} must not fall back to synthetic RPX",
            module.module_path
        );
    }
}

fn collect_bindings(expr: &CoreExpr) -> std::collections::HashMap<String, CoreExpr> {
    let mut map = std::collections::HashMap::new();
    fn walk(expr: &CoreExpr, map: &mut std::collections::HashMap<String, CoreExpr>) {
        match expr {
            CoreExpr::Let { name, value, body } => {
                map.insert(name.clone(), *value.clone());
                walk(body, map);
            }
            CoreExpr::LetRec { bindings, body } => {
                for (name, value) in bindings {
                    map.insert(name.clone(), value.clone());
                }
                walk(body, map);
            }
            _ => {}
        }
    }
    walk(expr, &mut map);
    map
}
