use std::path::PathBuf;

use reciplexa_core::elaborate::elaborate_source;
use reciplexa_core::expr::CoreExpr;
use reciplexa_eval::RuntimeValue;
use reciplexa_package::{
    differential_eval_both_fail, differential_eval_package_modules, eval_package_entry_main,
    register_test_ping_module, std_domain_natives, DomainNativeRegistry, LocalPackageIndex,
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
fn dn2_graphics_modules_hybrid_and_direct_native_agree_on_pkg_graphics_shapes() {
    assert_example_differential(
        "pkg_graphics_shapes.rpx",
        &["graphics/color", "graphics/page", "graphics/shapes"],
    );
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
fn dn2_japanese_linebreak_predicates_and_classify_sample_agree() {
    let idx = LocalPackageIndex::default().with_native(reciplexa_package::std_domain_natives());
    let src = r#"(import japanese/linebreak only
  line-head-prohibited-class? line-end-prohibited-class? inseparable-pair?
  hangable-class? numeric-before-close-prohibited? classify-sample
  opportunity pair-rule)
(val main
  (record
    (head2 (line-head-prohibited-class? 2))
    (head6 (line-head-prohibited-class? 6))
    (head15 (line-head-prohibited-class? 15))
    (end1 (line-end-prohibited-class? 1))
    (end12 (line-end-prohibited-class? 12))
    (end15 (line-end-prohibited-class? 15))
    (insep88 (inseparable-pair? 8 8))
    (insep819 (inseparable-pair? 8 19))
    (insep2424 (inseparable-pair? 24 24))
    (insep1515 (inseparable-pair? 15 15))
    (hang6 (hangable-class? 6))
    (hang15 (hangable-class? 15))
    (num202 (numeric-before-close-prohibited? 20 2))
    (num152 (numeric-before-close-prohibited? 15 2))
    (cl-open (classify-sample "「"))
    (cl-close (classify-sample "」"))
    (cl-ellipsis (classify-sample "…"))
    (cl-hi (classify-sample "あ"))
    (cl-A (classify-sample "A"))
    (opp (opportunity "allowed" 19 19 "ideograph run"))
    (rule (pair-rule 15 2 "prohibited" "letter + close"))))
"#;
    let entry = write_temp_rpx("reciplexa-dn2-ja-lb-dyn", src);
    differential_eval_package_modules(&entry, &idx, &["japanese/linebreak"])
        .expect("japanese/linebreak predicates and classify-sample hybrid and DN2 agree");
}

#[test]
fn dn2_japanese_linebreak_break_between_kind_agrees_on_hybrid_covered_pairs() {
    let idx = LocalPackageIndex::default().with_native(reciplexa_package::std_domain_natives());
    let src = r#"(import japanese/linebreak only break-between)
(val main
  (record
    (p88 (break-between 8 8))
    (p819 (break-between 8 19))
    (p115 (break-between 1 15))
    (p152 (break-between 15 2))
    (p156 (break-between 15 6))
    (p157 (break-between 15 7))
    (p1610 (break-between 16 10))
    (p1220 (break-between 12 20))
    (p201 (break-between 20 1))
    (p202 (break-between 20 2))
    (p1919 (break-between 19 19))))
"#;
    let entry = write_temp_rpx("reciplexa-dn2-ja-lb-kind", src);
    let hybrid_idx = idx
        .with_hybrid_reference_bodies(&["japanese/linebreak"])
        .expect("hybrid linebreak body");
    let hybrid = eval_package_entry_main(&entry, &hybrid_idx).expect("hybrid eval");
    let dn2 = eval_package_entry_main(&entry, &idx).expect("dn2 eval");
    let keys = [
        "p88", "p819", "p115", "p152", "p156", "p157", "p1610", "p1220", "p201", "p202", "p1919",
    ];
    for key in keys {
        let hk = opportunity_kind(record_field(&hybrid, key), key);
        let dk = opportunity_kind(record_field(&dn2, key), key);
        assert_eq!(
            hk, dk,
            "break-between kind mismatch on `{key}`: hybrid={hk} dn2={dk}"
        );
    }
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
fn dn2_document_page_hybrid_and_direct_native_agree_on_pkg_columns() {
    assert_example_differential("pkg_columns.rpx", &["document/page"]);
}

#[test]
fn dn2_math_modules_hybrid_and_direct_native_agree_on_pkg_math() {
    assert_example_differential(
        "pkg_math.rpx",
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
    );
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
        let source = module.effective_source();
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

#[test]
fn dn2_bind_stubs_assigns_compilation_binding_ids_for_all_std_exports() {
    use reciplexa_package::DomainNativeBindMap;
    let reg = std_domain_natives();
    let map = DomainNativeBindMap::bind_stubs(&reg).expect("std stubs bind");
    for module in reg.modules() {
        for name in module.typed_exports.keys() {
            let id = map
                .binding_for(&module.module_path, name)
                .unwrap_or_else(|| panic!("unbound {}/{name}", module.module_path));
            assert!(id.is_valid());
            assert_eq!(
                map.op_for(id),
                Some(module.typed_exports[name].op),
                "{}/{}",
                module.module_path,
                name
            );
        }
    }
}

#[test]
fn dn2_length_units_package_typechecks_and_exports_are_pure() {
    use reciplexa_package::{length_units_module, typecheck_with_packages};
    let packages = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_length.rpx");
    if !entry.is_file() {
        return;
    }
    let idx = LocalPackageIndex::discover(&[packages.as_path()]).unwrap();
    typecheck_with_packages(&entry, &idx).expect("DN2 typechecks pkg_length");
    for export in length_units_module().typed_exports.values() {
        match &export.ty {
            reciplexa_core::ty::CoreType::Fun { effects, .. } => {
                assert!(
                    effects.ops.is_empty(),
                    "`{}` should be pure, got {effects:?}",
                    export.name
                );
            }
            other => panic!("`{}` should be Fun, got {other:?}", export.name),
        }
    }
}

#[test]
fn dn2_length_units_arity_failure_agrees_on_hybrid_and_direct_native() {
    let idx = LocalPackageIndex::default().with_native(std_domain_natives());
    let src = "(import length/units only mm)\n(val main (mm))\n";
    let entry = write_temp_rpx("reciplexa-dn2-mm-arity", src);
    differential_eval_both_fail(&entry, &idx, &["length/units"])
        .expect("mm arity Failure on both hybrid and DN2");
    let err = eval_package_entry_main(&entry, &idx).expect_err("DN2 mm arity");
    let report = reciplexa_eval::parse_package_failure(&err.to_string()).expect("structured");
    assert_eq!(report.code.as_path(), "package/arity");
    assert!(report.message.contains("length/units/mm"));
}

#[test]
fn dn2_length_units_type_failure_is_structured_on_direct_native() {
    let idx = LocalPackageIndex::default().with_native(std_domain_natives());
    let src = "(import length/units only to-mm)\n(val main (to-mm 1))\n";
    let entry = write_temp_rpx("reciplexa-dn2-tomm-type", src);
    differential_eval_both_fail(&entry, &idx, &["length/units"])
        .expect("to-mm type Failure on both hybrid and DN2");
    let err = eval_package_entry_main(&entry, &idx).expect_err("DN2 to-mm type");
    let report = reciplexa_eval::parse_package_failure(&err.to_string()).expect("structured");
    assert_eq!(report.code.as_path(), "package/type");
    assert!(report.message.contains("length/units/to-mm"));
}

#[test]
fn dn2_color_and_japanese_type_failures_are_structured() {
    let idx = LocalPackageIndex::default().with_native(std_domain_natives());
    let from_byte = write_temp_rpx(
        "reciplexa-dn2-from-byte-type",
        "(import color/srgb only from-byte)\n(val main (from-byte \"a\" \"b\" \"c\"))\n",
    );
    let err = eval_package_entry_main(&from_byte, &idx).expect_err("DN2 from-byte type");
    let report = reciplexa_eval::parse_package_failure(&err.to_string()).expect("structured");
    assert_eq!(report.code.as_path(), "package/type");
    assert!(report.message.contains("color/srgb from-byte"));

    let classify = write_temp_rpx(
        "reciplexa-dn2-classify-type",
        "(import japanese/linebreak only classify-sample)\n(val main (classify-sample 1))\n",
    );
    let err = eval_package_entry_main(&classify, &idx).expect_err("DN2 classify-sample type");
    let report = reciplexa_eval::parse_package_failure(&err.to_string()).expect("structured");
    assert_eq!(report.code.as_path(), "package/type");
    assert!(report
        .message
        .contains("japanese/linebreak classify-sample"));
}

fn repo_example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .join(name)
}

fn assert_example_differential(example: &'static str, modules: &'static [&'static str]) {
    std::thread::Builder::new()
        .name(format!("dn2-{example}"))
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            let packages = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages");
            let entry = repo_example(example);
            assert!(
                entry.is_file(),
                "missing example {example} at {}",
                entry.display()
            );
            let idx = LocalPackageIndex::discover(&[packages.as_path()]).unwrap();
            differential_eval_package_modules(&entry, &idx, modules)
                .unwrap_or_else(|e| panic!("{example}: {e}"));
        })
        .expect("spawn dn2 differential thread")
        .join()
        .unwrap_or_else(|payload| std::panic::resume_unwind(payload));
}

fn write_temp_rpx(prefix: &str, src: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{prefix}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let entry = dir.join("main.rpx");
    std::fs::write(&entry, src).unwrap();
    entry
}

fn record_field<'a>(value: &'a RuntimeValue, key: &str) -> &'a RuntimeValue {
    let RuntimeValue::Record(fields) = value else {
        panic!("expected record, got {value:?}");
    };
    fields
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .unwrap_or_else(|| panic!("missing field `{key}` in {value:?}"))
}

fn opportunity_kind<'a>(value: &'a RuntimeValue, ctx: &str) -> &'a str {
    match record_field(value, "kind") {
        RuntimeValue::String(s) => s.as_str(),
        other => panic!("`{ctx}` kind should be string, got {other:?}"),
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
