//! Language-kernel conformance suite (LEX→…→EVAL path).
//!
//! Aggregates elaborate+eval, resolve_language, expand_language, and
//! typecheck_language. Document / page / circle surface checks live in
//! `reciplexa-types` (DOCUMENT SURFACE), not here.

use std::path::PathBuf;

use reciplexa_bind::{elaborate_module_tree, elaborate_units, resolve_language_source};
use reciplexa_core::{
    elaborate_source, elaborate_with_data, typecheck_language_source, CoreType, Variance,
};
use reciplexa_eval::{eval_expr, eval_source, primitive_env, RuntimeValue, UnitHost};
use reciplexa_macro::{expand_language, expand_language_with_map};
use reciplexa_package::{elaborate_with_packages, LocalPackageIndex};
use reciplexa_test::{run_conformance, ConformanceCase};

fn workspace_packages() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages")
}

fn package_index() -> LocalPackageIndex {
    LocalPackageIndex::discover(&[workspace_packages()]).expect("discover packages/")
}

#[test]
fn lang_elaborate_and_eval_identity() {
    let case = ConformanceCase::new(
        "TEST-LANG-EVAL-001",
        "EVAL-001",
        "elaborate + eval identity application",
    );
    run_conformance(&case, || {
        let expr = elaborate_source("(val main ((fn (x) x) 42))").unwrap();
        let _ = expr;
        let v = eval_source("(val main ((fn (x) x) 42))").unwrap();
        assert_eq!(v, RuntimeValue::Int(42));
    });
}

#[test]
fn lang_resolve_language_shadowing() {
    let case = ConformanceCase::new(
        "TEST-LANG-RES-001",
        "RES-001",
        "resolve_language_source shadowing",
    );
    run_conformance(&case, || {
        let r = resolve_language_source("(val x 1) (val main (let ((x 2)) x))");
        assert!(r.is_ok(), "{:?}", r.errors);
    });
}

#[test]
fn lang_expand_language_call1() {
    let case = ConformanceCase::new(
        "TEST-LANG-MAC-001",
        "MAC-001",
        "expand_language user macro before eval",
    );
    run_conformance(&case, || {
        let src = r#"
(macro call1 ($f $x) -> ($f $x))
(val main (call1 (fn (n) n) 9))
"#;
        let expanded = expand_language(src).unwrap();
        assert!(!expanded.contains("call1") || expanded.contains("fn"));
        let v = eval_source(src).unwrap();
        assert_eq!(v, RuntimeValue::Int(9));
    });
}

#[test]
fn lang_typecheck_language_identity_number() {
    let case = ConformanceCase::new(
        "TEST-LANG-TYP-001",
        "TYP-001",
        "typecheck_language_source identity app is Int",
    );
    run_conformance(&case, || {
        let ty = typecheck_language_source("(val main ((fn (x) x) 1))").unwrap();
        assert_eq!(ty, CoreType::Int);
    });
}

#[test]
fn lang_handle_shallow_v0() {
    let case = ConformanceCase::new(
        "TEST-LANG-EFF-001",
        "EFF-001",
        "shallow handle catches perform",
    );
    run_conformance(&case, || {
        let v =
            eval_source(r#"(val main (handle log (fn (msg) msg) (perform log "ok")))"#).unwrap();
        assert_eq!(v, RuntimeValue::String("ok".into()));
    });
}

#[test]
fn lang_data_match() {
    let case = ConformanceCase::new("TEST-LANG-DAT-001", "DAT-001", "data/match elaborates to 1");
    run_conformance(&case, || {
        let src = include_str!("../../../examples/lang_match.rpx");
        let v = eval_source(src).unwrap();
        assert_eq!(v, RuntimeValue::Int(1));
        let ty = typecheck_language_source(src).unwrap();
        assert_eq!(ty, CoreType::Int);
    });
}

#[test]
fn lang_data_match_non_exhaustive() {
    let case = ConformanceCase::new(
        "TEST-LANG-DAT-002",
        "DAT-001",
        "non-exhaustive match is a static error",
    );
    run_conformance(&case, || {
        let src = r#"
(data option (none) (some x))
(val main (match (some 1) (some x -> x)))
"#;
        let err = elaborate_source(src).unwrap_err();
        assert!(err.message.contains("non-exhaustive"), "{}", err.message);
        let err = typecheck_language_source(src).unwrap_err();
        assert!(err.message.contains("non-exhaustive"), "{}", err.message);
    });
}

#[test]
fn lang_literal_tuple_multipayload_patterns() {
    let case = ConformanceCase::new(
        "TEST-LANG-DAT-003",
        "DAT-001",
        "literal, tuple, and multi-payload patterns",
    );
    run_conformance(&case, || {
        let v = eval_source(
            r#"
(val main (match 0 (0 -> "zero") (_ -> "other")))
"#,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::String("zero".into()));

        let v = eval_source(
            r#"
(val main
  (match (tuple 1 2)
    (tuple a b -> (+ a b))))
"#,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::Int(3));

        let v = eval_source(
            r#"
(data pair (pair x y))
(val main (match (pair 4 5) (pair a b -> (* a b))))
"#,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::Int(20));
    });
}

#[test]
fn lang_letrec() {
    let case = ConformanceCase::new("TEST-LANG-BND-001", "BND-001", "letrec self-call");
    run_conformance(&case, || {
        let src = include_str!("../../../examples/lang_letrec.rpx");
        let v = eval_source(src).unwrap();
        assert_eq!(v, RuntimeValue::Int(7));
    });
}

#[test]
fn lang_toplevel_rec_and_local_var() {
    let case = ConformanceCase::new(
        "TEST-LANG-BND-rec-local",
        "BND-001",
        "top-level rec and local var decls",
    );
    run_conformance(&case, || {
        let v = eval_source(
            r#"
(rec
  (val even?
    (fn (n)
      (if (= n 0) true (odd? (- n 1)))))
  (val odd?
    (fn (n)
      (if (= n 0) false (even? (- n 1))))))
(val main (even? 3))
"#,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::Bool(false));

        let v = eval_source(
            r#"
(val main
  (local
    (var n 1)
    (rec
      (val bump (fn () (set n (+ n 1)))))
    (seq (bump) n)))
"#,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::Int(2));
    });
}

#[test]
fn lang_primitives() {
    let case = ConformanceCase::new("TEST-LANG-KER-001", "KER-001", "numeric primitives");
    run_conformance(&case, || {
        let src = include_str!("../../../examples/lang_prim.rpx");
        let v = eval_source(src).unwrap();
        assert_eq!(v, RuntimeValue::Int(10));
    });
}

#[test]
fn lang_deep_resume() {
    let case = ConformanceCase::new(
        "TEST-LANG-EFF-001-deep",
        "EFF-001",
        "deep one-shot resume continues body",
    );
    run_conformance(&case, || {
        let v =
            eval_source(r#"(val main (handle ask (fn (_ k) (k 41)) (seq (perform ask 0) 99)))"#)
                .unwrap();
        assert_eq!(v, RuntimeValue::Int(99));
    });
}

#[test]
fn lang_with_and_handler_value() {
    let case = ConformanceCase::new(
        "TEST-LANG-EFF-with",
        "EFF-001",
        "first-class handler + with sugar",
    );
    run_conformance(&case, || {
        let v = eval_source(
            r#"
(val h (handler log (fn (msg) msg)))
(val main (with h (perform log "via-with")))
"#,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::String("via-with".into()));

        let v2 = eval_source(
            r#"
(val main
  (with (handler ask (fn (_ k) (k 7)))
    (seq (perform ask 0) 42)))
"#,
        )
        .unwrap();
        assert_eq!(v2, RuntimeValue::Int(42));
    });
}

#[test]
fn lang_var_set() {
    let case = ConformanceCase::new("TEST-LANG-BND-var", "BND-001", "var/set cell");
    run_conformance(&case, || {
        let v = eval_source(
            r#"
(val main
  (var count 0
    (set count 3)
    count))
"#,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::Int(3));
    });
}

#[test]
fn lang_surface_type_decls_and_dynamic() {
    let case = ConformanceCase::new(
        "TEST-LANG-TYP-type",
        "TYP-001",
        "surface (type name Ty) and (dynamic)",
    );
    run_conformance(&case, || {
        let (expr, data) = reciplexa_core::elaborate_with_data(
            r#"
(type title str)
(type blob (dynamic))
(type either (union int str))
(val title "ok")
(val main title)
"#,
        )
        .unwrap();
        assert!(matches!(
            data.type_aliases.get("title"),
            Some(CoreType::String)
        ));
        assert!(matches!(
            data.type_aliases.get("blob"),
            Some(CoreType::Dynamic(b)) if matches!(b.as_ref(), CoreType::Any)
        ));
        assert!(matches!(
            data.type_aliases.get("either"),
            Some(CoreType::Union(_))
        ));
        let _ = expr;
        let v = eval_source(
            r#"
(type title str)
(val title "ok")
(val main title)
"#,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::String("ok".into()));
        let ty = typecheck_language_source(
            r#"
(type title str)
(val title "ok")
(val main title)
"#,
        )
        .unwrap();
        assert_eq!(ty, CoreType::String);
    });
}

#[test]
fn lang_match_unreachable_case() {
    let case = ConformanceCase::new(
        "TEST-LANG-DAT-unreach",
        "DAT-001",
        "unreachable match arm is a static error",
    );
    run_conformance(&case, || {
        let src = r#"
(data option (none) (some x))
(val main (match (some 1) (_ -> 0) (some x -> x)))
"#;
        let err = elaborate_source(src).unwrap_err();
        assert!(err.message.contains("unreachable"), "{}", err.message);
        let err = typecheck_language_source(src).unwrap_err();
        assert!(err.message.contains("unreachable"), "{}", err.message);
    });
}

#[test]
fn lang_occurrence_typing_predicates() {
    let case = ConformanceCase::new(
        "TEST-LANG-TYP-occ",
        "TYP-001",
        "DD-TYP-IF-001 occurrence typing on builtins",
    );
    run_conformance(&case, || {
        let ty = typecheck_language_source(
            r#"
(val x (if true 1 "a"))
(val main
  (if (number? x)
    (+ x 1)
    0))
"#,
        )
        .unwrap();
        assert_eq!(ty, CoreType::Int);
    });
}

#[test]
fn lang_data_rec_group_positivity() {
    let case = ConformanceCase::new(
        "TEST-LANG-DAT-rec-pos",
        "DAT-001",
        "mutual data rec group positivity",
    );
    run_conformance(&case, || {
        let err = elaborate_source(
            r#"
(rec
  (data a (a-con (fn b unit)))
  (data b (b-con (fn a unit))))
(val main unit)
"#,
        )
        .unwrap_err();
        assert!(
            err.message.contains("positivity") || err.message.contains("negative"),
            "{}",
            err.message
        );
    });
}

#[test]
fn lang_lit_bytes_utf8_roundtrip() {
    let case = ConformanceCase::new(
        "TEST-LANG-LIT-bytes",
        "SYN-001",
        "encode-utf8 / decode-utf8 builtins (SYN §11)",
    );
    run_conformance(&case, || {
        let v = eval_source(r#"(val main (encode-utf8 "hi"))"#).unwrap();
        assert_eq!(v, RuntimeValue::Bytes(vec![104, 105]));
        let v = eval_source(r#"(val main (decode-utf8 (encode-utf8 "ok")))"#).unwrap();
        let s = format!("{v}");
        assert!(s.contains("ok"), "got {s}");
    });
}

#[test]
fn lang_pkg_length_and_color_imports() {
    let case = ConformanceCase::new(
        "TEST-LANG-PKG-length-color",
        "PKG-001",
        "length/units and color/srgb package imports",
    );
    run_conformance(&case, || {
        let dir =
            std::env::temp_dir().join(format!("reciplexa-lang-pkg-lc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let entry = dir.join("demo.rpx");
        std::fs::write(
            &entry,
            r#"(import length/units only cm)
(import color/srgb only blue)
(val main (record (len (cm 10)) (color blue)))
"#,
        )
        .unwrap();
        let units = elaborate_with_packages(&entry, &package_index()).unwrap();
        let demo = units.iter().find(|u| u.name == "demo").unwrap();
        let v = eval_expr(&demo.expr, &primitive_env(), &mut UnitHost).unwrap();
        let s = format!("{v}");
        assert!(s.contains("cm") || s.contains("blue"), "got {s}");
        let _ = std::fs::remove_dir_all(&dir);
    });
}

#[test]
fn lang_mod_import_link_and_qualified_ref() {
    let case = ConformanceCase::new(
        "TEST-LANG-MOD-01",
        "MOD-001",
        "import only + qualified ref linking (MOD-01)",
    );
    run_conformance(&case, || {
        let units = elaborate_units(&[
            ("graphics/color", r#"(val black 0) (val white 1)"#),
            (
                "main",
                r#"(import graphics/color as color only black) (val main color/black)"#,
            ),
        ])
        .unwrap();
        let main = units.iter().find(|u| u.name == "main").unwrap();
        let v = eval_expr(&main.expr, &primitive_env(), &mut UnitHost).unwrap();
        assert_eq!(v, RuntimeValue::Int(0));
    });
}

#[test]
fn lang_mod_duplicate_identity_import() {
    let case = ConformanceCase::new(
        "TEST-LANG-MOD-02",
        "MOD-001",
        "same module identity imported twice (MOD-02 / §7.4)",
    );
    run_conformance(&case, || {
        let units = elaborate_units(&[
            ("lib", r#"(val id (fn (x) x))"#),
            (
                "main",
                r#"(import lib only id) (import lib only id) (val main (id 9))"#,
            ),
        ])
        .unwrap();
        let main = units.iter().find(|u| u.name == "main").unwrap();
        let v = eval_expr(&main.expr, &primitive_env(), &mut UnitHost).unwrap();
        assert_eq!(v, RuntimeValue::Int(9));
    });
}

#[test]
fn lang_mod_rejects_colliding_locals_from_distinct_modules() {
    let case = ConformanceCase::new(
        "TEST-LANG-MOD-03",
        "MOD-001",
        "distinct module identities cannot share only-import local",
    );
    run_conformance(&case, || {
        let err = elaborate_units(&[
            ("a", r#"(val id 1)"#),
            ("b", r#"(val id 2)"#),
            (
                "main",
                r#"(import a only id) (import b only id) (val main id)"#,
            ),
        ])
        .unwrap_err();
        assert!(err.message.contains("distinct modules"), "{}", err.message);
    });
}

#[test]
fn lang_mod_filesystem_nested_module_path() {
    let case = ConformanceCase::new(
        "TEST-LANG-MOD-fs",
        "MOD-001",
        "load_module_tree resolves nested sibling paths",
    );
    run_conformance(&case, || {
        let dir =
            std::env::temp_dir().join(format!("reciplexa-lang-mod-fs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::write(dir.join("lib/util.rpx"), "(val twice (fn (x) (* x 2)))\n").unwrap();
        std::fs::write(
            dir.join("main.rpx"),
            "(import lib/util only twice)\n(val main (twice 3))\n",
        )
        .unwrap();
        let units = elaborate_module_tree(dir.join("main.rpx")).unwrap();
        let main = units.iter().find(|u| u.name == "main").unwrap();
        let v = eval_expr(&main.expr, &primitive_env(), &mut UnitHost).unwrap();
        assert_eq!(v, RuntimeValue::Int(6));
        let _ = std::fs::remove_dir_all(&dir);
    });
}

#[test]
fn lang_pkg_graphics_shapes_import() {
    let case = ConformanceCase::new(
        "TEST-LANG-PKG-01",
        "PKG-001",
        "script imports graphics/shapes circle from packages/",
    );
    run_conformance(&case, || {
        let src = include_str!("../../../examples/pkg_circle.rpx");
        let dir =
            std::env::temp_dir().join(format!("reciplexa-lang-pkg-01-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let entry = dir.join("demo.rpx");
        std::fs::write(&entry, src).unwrap();
        let units = elaborate_with_packages(&entry, &package_index()).unwrap();
        let demo = units.iter().find(|u| u.name == "demo").unwrap();
        let v = eval_expr(&demo.expr, &primitive_env(), &mut UnitHost).unwrap();
        let s = format!("{v}");
        assert!(
            s.contains("circle") || s.contains("105"),
            "expected circle record, got {s}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    });
}

#[test]
fn lang_pkg_path_dep_alias_import() {
    let case = ConformanceCase::new(
        "TEST-LANG-PKG-02",
        "PKG-001",
        "consumer path dependency alias g/shapes resolves",
    );
    run_conformance(&case, || {
        let consumer =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_consumer");
        let (idx, _) =
            LocalPackageIndex::discover_with_consumer(&[workspace_packages()], &consumer).unwrap();
        let entry = consumer.join("src/main.rpx");
        let units = elaborate_with_packages(&entry, &idx).unwrap();
        let main = units.iter().find(|u| u.name == "main").unwrap();
        let v = eval_expr(&main.expr, &primitive_env(), &mut UnitHost).unwrap();
        let s = format!("{v}");
        assert!(s.contains("circle") || s.contains("10"), "got {s}");
    });
}

#[test]
fn lang_pkg_math_and_japanese_stubs() {
    let case = ConformanceCase::new(
        "TEST-LANG-PKG-stubs",
        "PKG-001",
        "math/atoms and japanese/markup package stubs elaborate",
    );
    run_conformance(&case, || {
        let idx = package_index();
        let dir =
            std::env::temp_dir().join(format!("reciplexa-lang-pkg-stubs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let entry = dir.join("demo.rpx");
        std::fs::write(
            &entry,
            r#"(import math/atoms only fraction)
(import japanese/markup only heading)
(val main (heading "ok"))
"#,
        )
        .unwrap();
        let units = elaborate_with_packages(&entry, &idx).unwrap();
        let demo = units.iter().find(|u| u.name == "demo").unwrap();
        let v = eval_expr(&demo.expr, &primitive_env(), &mut UnitHost).unwrap();
        let s = format!("{v}");
        assert!(s.contains("ja-heading") || s.contains("ok"), "got {s}");
        let _ = std::fs::remove_dir_all(&dir);
    });
}

#[test]
fn lang_pkg_graphics_page_and_ring() {
    let case = ConformanceCase::new(
        "TEST-LANG-PKG-graphics",
        "PKG-001",
        "graphics page + ring constructors from packages",
    );
    run_conformance(&case, || {
        let dir =
            std::env::temp_dir().join(format!("reciplexa-lang-pkg-gfx-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let entry = dir.join("demo.rpx");
        std::fs::write(
            &entry,
            r#"(import graphics/page only page a4)
(import graphics/shapes only ring)
(val main (page a4 (ring 1 2 3 0.5)))
"#,
        )
        .unwrap();
        let units = elaborate_with_packages(&entry, &package_index()).unwrap();
        let demo = units.iter().find(|u| u.name == "demo").unwrap();
        let v = eval_expr(&demo.expr, &primitive_env(), &mut UnitHost).unwrap();
        let s = format!("{v}");
        assert!(
            s.contains("page") || s.contains("ring"),
            "expected page/ring record, got {s}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    });
}

#[test]
fn lang_typ_try_cast_and_check_cast() {
    // Spec: TEST-STA-007 (dynamic boundary) / DD-TYP-DYN try-cast·check-cast.
    let case = ConformanceCase::new(
        "TEST-STA-007",
        "TYP-001",
        "try-cast / check-cast dynamic boundary (STA-007)",
    );
    run_conformance(&case, || {
        let ty = typecheck_language_source("(val main (try-cast 1 int))").unwrap();
        assert_eq!(
            ty,
            CoreType::App {
                ctor: "option".into(),
                args: vec![CoreType::Int],
            }
        );
        let v = eval_source("(val main (try-cast 1 int))").unwrap();
        assert!(matches!(
            v,
            RuntimeValue::Variant {
                tag,
                payload: Some(_),
            } if tag == "some"
        ));

        let ty = typecheck_language_source(r#"(val main (check-cast "a" string))"#).unwrap();
        assert_eq!(
            ty,
            CoreType::App {
                ctor: "result".into(),
                args: vec![CoreType::String, CoreType::String],
            }
        );
        let v = eval_source(r#"(val main (check-cast "a" string))"#).unwrap();
        assert!(matches!(
            v,
            RuntimeValue::Variant {
                tag,
                payload: Some(_),
            } if tag == "ok"
        ));
    });
}

#[test]
fn lang_typ_any_and_dynamic_any() {
    let case = ConformanceCase::new(
        "TEST-LANG-TYP-any",
        "TYP-001",
        "`any` / `(dynamic)` type surface (DD-TYP-DYN)",
    );
    run_conformance(&case, || {
        let (_, data) = elaborate_with_data(
            r#"
(type blob (dynamic))
(type anything any)
(val main unit)
"#,
        )
        .unwrap();
        assert!(matches!(
            data.type_aliases.get("blob"),
            Some(CoreType::Dynamic(b)) if matches!(b.as_ref(), CoreType::Any)
        ));
        assert_eq!(data.type_aliases.get("anything"), Some(&CoreType::Any));
    });
}

#[test]
fn lang_dat_variance_inference() {
    let case = ConformanceCase::new(
        "TEST-LANG-DAT-variance",
        "DAT-001",
        "DAT §8 covariance / phantom inference",
    );
    run_conformance(&case, || {
        let (_, data) = elaborate_with_data(
            r#"
(data tree
  ((a type))
  empty
  (node a (tree a) (tree a)))
(val main empty)
"#,
        )
        .unwrap();
        assert_eq!(
            data.type_variances.get("tree").and_then(|v| v.get("a")),
            Some(&Variance::Covariant)
        );

        let (_, data) = elaborate_with_data(
            r#"
(data identifier
  ((domain type))
  (identifier int))
(val main unit)
"#,
        )
        .unwrap();
        assert_eq!(
            data.type_variances
                .get("identifier")
                .and_then(|v| v.get("domain")),
            Some(&Variance::Phantom)
        );
    });
}

#[test]
fn lang_lit_bytes_literal() {
    let case = ConformanceCase::new(
        "TEST-LANG-LIT-bytes-lit",
        "SYN-001",
        "(bytes …) literal elaborates and evaluates (SYN §11)",
    );
    run_conformance(&case, || {
        let v = eval_source("(val main (bytes 0xff 42))").unwrap();
        assert_eq!(v, RuntimeValue::Bytes(vec![0xff, 42]));
    });
}

#[test]
fn lang_eff_failure_raise_and_forward() {
    // Spec: TEST-DYN-003 nested handlers / resume / forward; failure op.
    let case = ConformanceCase::new(
        "TEST-DYN-003",
        "EFF-001",
        "failure/raise + forward nested handlers (DYN-003)",
    );
    run_conformance(&case, || {
        let v =
            eval_source(r#"(val main (handle failure (fn (err) err) (raise "caught")))"#).unwrap();
        assert_eq!(v, RuntimeValue::String("caught".into()));

        let v = eval_source(
            r#"
(val main
  (handle log (fn (msg) (seq (perform log "inner") msg))
    (handle log (fn (msg k) (forward k))
      (perform log "outer"))))
"#,
        )
        .unwrap();
        assert_eq!(v, RuntimeValue::String("outer".into()));
    });
}

#[test]
fn lang_mac_nested_module_and_nest_expand() {
    // Spec: TEST-STA-008 expanded binding identity / nested macros.
    let case = ConformanceCase::new(
        "TEST-STA-008",
        "MAC-001",
        "nested module macros + expand provenance (STA-008)",
    );
    run_conformance(&case, || {
        let src = r#"
(macro when ($condition $body ...+) -> (if $condition (seq $body ...) unit))
(module rendering
  (val render-if-ready (when ready? 1)))
"#;
        let out = expand_language(src).unwrap();
        assert!(out.contains("(if ready? (seq 1) unit)"), "{out}");
        assert!(!out.contains("(when "), "{out}");

        let nested = r#"
(macro unless ($condition $expression) -> (if $condition unit $expression))
(macro unless-ready ($body ...+) -> (unless ready? $body ...))
(val main (unless-ready 1))
"#;
        assert_eq!(
            expand_language(nested).unwrap(),
            "(val main (if ready? unit 1))"
        );

        let (expanded, map) = expand_language_with_map(
            "(macro when ($c $b) -> (if $c $b unit))\n(val main (when true 1))",
        )
        .unwrap();
        assert_eq!(expanded, "(val main (if true 1 unit))");
        assert!(!map.origins.is_empty());
        let o = &map.origins[0];
        assert!(o.call_id.is_valid(), "call SyntaxNodeId");
        assert!(o.def_id.is_valid(), "def SyntaxNodeId");
    });
}

#[test]
fn lang_edt_binding_map_use_sites() {
    // EDT-001 language-core: BindingId use-sites via resolve BindingMap.
    let case = ConformanceCase::new(
        "TEST-LANG-EDT-binding",
        "EDT-001",
        "BindingMap use-site → declaration BindingId",
    );
    run_conformance(&case, || {
        let src = "(val n 1) (val main (let ((x n)) n))";
        let r = resolve_language_source(src);
        assert!(r.is_ok(), "{:?}", r.errors);
        assert!(
            !r.binding_map.uses.is_empty(),
            "expected use-sites, got defs={:?}",
            r.binding_map.definitions.keys().collect::<Vec<_>>()
        );
        assert!(
            !r.binding_map.definitions.is_empty(),
            "expected definitions"
        );
        for id in r.binding_map.uses.values() {
            let site = r
                .binding_map
                .definition_of(*id)
                .unwrap_or_else(|| panic!("missing def for {id}"));
            assert!(id.is_valid());
            assert!(!site.name.is_empty());
            assert!(
                site.syntax_node_id.is_some(),
                "def `{}` should carry SyntaxNodeId",
                site.name
            );
        }
        let n_def = r
            .binding_map
            .definitions
            .values()
            .find(|s| s.name == "n")
            .expect("n binding");
        let use_hits: Vec<_> = r
            .binding_map
            .uses
            .iter()
            .filter(|(_, id)| **id == n_def.binding_id)
            .collect();
        assert!(
            use_hits.len() >= 2,
            "expected ≥2 uses of n, got {}",
            use_hits.len()
        );
    });
}

#[test]
fn lang_pkg_rpi_export_boundary() {
    // Spec: TEST-INT-002 multi-file module/package; MOD §8 .rpi boundary.
    let case = ConformanceCase::new(
        "TEST-INT-002",
        "PKG-001",
        ".rpi interface export boundary hides internal (INT-002)",
    );
    run_conformance(&case, || {
        let idx = package_index();
        let resolved = idx
            .resolve_import_detailed("graphics/shapes")
            .expect("shapes");
        let exports = resolved.interface_exports.expect("shapes.rpi");
        assert!(exports.contains(&"circle".into()));
        assert!(!exports.iter().any(|e| e == "shapes-internal-tag"));

        let (root, manifest) = idx.get("graphics").expect("graphics");
        let path = manifest
            .module_interface_path(root, "shapes")
            .expect("interface-root");
        assert!(
            path.ends_with("interface\\shapes.rpi") || path.ends_with("interface/shapes.rpi"),
            "{path:?}"
        );
        assert!(path.is_file());

        let dir = std::env::temp_dir().join(format!("reciplexa-lang-rpi-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let entry = dir.join("demo.rpx");
        std::fs::write(
            &entry,
            r#"(import graphics/shapes only shapes-internal-tag)
(val main (shapes-internal-tag "x"))
"#,
        )
        .unwrap();
        let err = elaborate_with_packages(&entry, &idx).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("not exported") && msg.contains("shapes-internal-tag"),
            "unexpected: {msg}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    });
}

#[test]
fn lang_dat_adt_01_parameterized_option() {
    // Spec DAT §21.9 ADT-01 (parameterized option + type alias).
    let case = ConformanceCase::new(
        "TEST-LANG-ADT-01",
        "DAT-001",
        "ADT-01 parameterized option elaborates",
    );
    run_conformance(&case, || {
        let src = r#"
(data option
  ((a type))
  none
  (some a))
(type value (option int))
(val value (some 42))
(val main value)
"#;
        let ty = typecheck_language_source(src).unwrap();
        match ty {
            CoreType::App { ctor, args } => {
                assert_eq!(ctor, "option");
                assert_eq!(args, vec![CoreType::Int]);
            }
            CoreType::Variant { variants } => {
                assert!(variants.iter().any(|(t, _)| t == "none"));
                assert!(variants
                    .iter()
                    .any(|(t, p)| { t == "some" && matches!(p, Some(CoreType::Int)) }));
            }
            other => panic!("expected option App or Variant, got {other:?}"),
        }
        let v = eval_source(src).unwrap();
        assert!(matches!(v, RuntimeValue::Variant { tag, .. } if tag == "some"));
    });
}

#[test]
fn lang_dat_adt_02_exhaustive_match() {
    // Spec DAT §21.10 ADT-02.
    let case = ConformanceCase::new(
        "TEST-LANG-ADT-02",
        "DAT-001",
        "ADT-02 exhaustive option match → int",
    );
    run_conformance(&case, || {
        let src = r#"
(data option (none) (some x))
(val value (some 42))
(val main
  (match value
    (some item -> item)
    (none -> 0)))
"#;
        assert_eq!(typecheck_language_source(src).unwrap(), CoreType::Int);
        assert_eq!(eval_source(src).unwrap(), RuntimeValue::Int(42));
    });
}

#[test]
fn lang_dat_adt_03_non_exhaustive() {
    // Spec DAT §21.11 ADT-03.
    let case = ConformanceCase::new(
        "TEST-LANG-ADT-03",
        "DAT-001",
        "ADT-03 non-exhaustive match static error",
    );
    run_conformance(&case, || {
        let src = r#"
(data option (none) (some x))
(val value (some 42))
(val main (match value (some item -> item)))
"#;
        let err = elaborate_source(src).unwrap_err();
        assert!(err.message.contains("non-exhaustive"), "{}", err.message);
        assert!(err.message.contains("none"), "{}", err.message);
    });
}

#[test]
fn lang_dat_adt_04_unreachable() {
    // Spec DAT §21.12 ADT-04.
    let case = ConformanceCase::new(
        "TEST-LANG-ADT-04",
        "DAT-001",
        "ADT-04 unreachable arm after wildcard",
    );
    run_conformance(&case, || {
        let src = r#"
(data option (none) (some x))
(val value (some 42))
(val main
  (match value
    (_ -> 0)
    (some item -> item)))
"#;
        let err = elaborate_source(src).unwrap_err();
        assert!(err.message.contains("unreachable"), "{}", err.message);
    });
}

#[test]
fn lang_dat_adt_05_strictly_positive_tree() {
    // Spec DAT §21.13 ADT-05.
    let case = ConformanceCase::new(
        "TEST-LANG-ADT-05",
        "DAT-001",
        "ADT-05 recursive tree is strictly positive",
    );
    run_conformance(&case, || {
        let src = r#"
(data tree
  ((a type))
  empty
  (node a (tree a) (tree a)))
(val main empty)
"#;
        let (_, data) = elaborate_with_data(src).unwrap();
        assert!(data.data_ctors.contains_key("tree"));
        assert_eq!(
            data.type_variances.get("tree").and_then(|v| v.get("a")),
            Some(&Variance::Covariant)
        );
    });
}

#[test]
fn lang_dat_adt_06_negative_recursion() {
    // Spec DAT §21.14 ADT-06.
    let case = ConformanceCase::new(
        "TEST-LANG-ADT-06",
        "DAT-001",
        "ADT-06 negative recursion rejected",
    );
    run_conformance(&case, || {
        let err = elaborate_source(
            r#"
(data bad
  (bad (fn bad unit)))
(val main unit)
"#,
        )
        .unwrap_err();
        assert!(
            err.message.contains("positivity") || err.message.contains("negative"),
            "{}",
            err.message
        );
    });
}

#[test]
fn lang_dat_adt_07_result_generalizes_error_param() {
    // Spec DAT §21.15 ADT-07 / §7.3.
    let case = ConformanceCase::new(
        "TEST-LANG-ADT-07",
        "DAT-001",
        "ADT-07 forall e. result<int, e> generalization",
    );
    run_conformance(&case, || {
        let src = r#"
(data result
  ((a type) (e type))
  (ok a)
  (err e))
(val success (ok 42))
(val main success)
"#;
        let ty = typecheck_language_source(src).unwrap();
        match ty {
            CoreType::Forall { params, body } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].0, "e");
                assert_eq!(params[0].1, "type");
                match *body {
                    CoreType::App { ctor, args } => {
                        assert_eq!(ctor, "result");
                        assert_eq!(args.len(), 2);
                        assert_eq!(args[0], CoreType::Int);
                        assert_eq!(args[1], CoreType::Name("e".into()));
                    }
                    other => panic!("expected App body, got {other:?}"),
                }
            }
            other => panic!("expected forall e. result<int, e>, got {other:?}"),
        }
        let v = eval_source(src).unwrap();
        assert!(matches!(v, RuntimeValue::Variant { tag, .. } if tag == "ok"));
    });
}

#[test]
fn lang_dat_adt_08_var_rejects_ungeneralized_error() {
    // Spec DAT §21.16 ADT-08 / §7.4 value restriction.
    let case = ConformanceCase::new(
        "TEST-LANG-ADT-08",
        "DAT-001",
        "ADT-08 var ungeneralized error type annotation-required",
    );
    run_conformance(&case, || {
        let err = typecheck_language_source(
            r#"
(data result
  ((a type) (e type))
  (ok a)
  (err e))
(val main
  (local
    (var success (ok 42))
    success))
"#,
        )
        .unwrap_err();
        assert!(
            err.message.contains("ungeneralized")
                || err.message.contains("annotation")
                || err.message.contains("error type"),
            "{}",
            err.message
        );
    });
}

#[test]
fn lang_dat_adt_09_record_pattern() {
    // Spec DAT §21.17 ADT-09.
    let case = ConformanceCase::new(
        "TEST-LANG-ADT-09",
        "DAT-001",
        "ADT-09 required record field pattern",
    );
    run_conformance(&case, || {
        let src = r#"
(val main
  (match (record (title "ok") (page-count 1))
    (record (title title) -> title)))
"#;
        assert_eq!(typecheck_language_source(src).unwrap(), CoreType::String);
        assert_eq!(eval_source(src).unwrap(), RuntimeValue::String("ok".into()));
    });
}

#[test]
fn lang_dat_adt_10_optional_pattern_rejected() {
    // Spec DAT §21.18 ADT-10.
    let case = ConformanceCase::new(
        "TEST-LANG-ADT-10",
        "DAT-001",
        "ADT-10 optional field record pattern rejected",
    );
    run_conformance(&case, || {
        let err = elaborate_source(
            r#"
(val main
  (match (record (title "t"))
    (record (optional subtitle x) -> x)
    (_ -> unit)))
"#,
        )
        .unwrap_err();
        assert!(
            err.message.contains("optional")
                && (err.message.contains("18.5") || err.message.contains("decompos")),
            "{}",
            err.message
        );
    });
}
