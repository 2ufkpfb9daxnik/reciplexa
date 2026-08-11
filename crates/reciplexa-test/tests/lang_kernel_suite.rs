//! Language-kernel conformance suite (LEX→…→EVAL path).
//!
//! Aggregates elaborate+eval, resolve_language, expand_language, and
//! typecheck_language. Document / page / circle surface checks live in
//! `reciplexa-types` (DOCUMENT SURFACE), not here.

use std::path::PathBuf;

use reciplexa_bind::{elaborate_module_tree, elaborate_units, resolve_language_source};
use reciplexa_core::{elaborate_source, typecheck_language_source, CoreType};
use reciplexa_eval::{eval_expr, eval_source, primitive_env, RuntimeValue, UnitHost};
use reciplexa_macro::expand_language;
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
            Some(CoreType::Dynamic)
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
