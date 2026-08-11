//! Round-6 bind: remaining resolve pattern/module load edges.

use reciplexa_bind::module::*;
use reciplexa_bind::resolve_language_source;

#[test]
fn resolve_pattern_and_let_malformed_dense() {
    for src in [
        "(val main (match v ((tuple a (bind b)) -> a) (_ -> 0)))",
        "(val main (match v ((record (a (bind b))) -> b) (_ -> 0)))",
        "(val main (match v ((record (a _)) -> 0) (_ -> 1)))",
        "(val main (match v ((some (tuple a b)) -> a) (_ -> 0)))",
        "(val main (match v ((mk a b c) -> a) (_ -> 0)))",
        "(val main (let (not-list) 1))",
        "(val main (letrec (not-list) 1))",
        "(val main (let ((x 1) not-pair) x))",
        "(val main (letrec ((f (fn (x) x)) not-pair) f))",
        "(val main (fn (x 1) x))",
        "(val main (fn [x] x))",
        "(fn f [x] x)\n(val main (f 1))",
        "(val (f [x]) x)\n(val main (f 1))",
        "(data t ((a type)(b type)) (mk a b))\n(val main (mk 1 2))",
        "(type)\n(fn)\n(val)",
        "((val x 1))\n(val main 1)",
    ] {
        let _ = resolve_language_source(src);
    }
}

#[test]
fn module_elaborate_tree_and_empty_dir() {
    let dir = std::env::temp_dir().join(format!("reciplexa-b6-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // Empty dir → no .rpx
    let err = load_module_tree(&dir);
    assert!(err.is_err());

    std::fs::write(dir.join("lib.rpx"), "(val x 1)\n").unwrap();
    std::fs::write(
        dir.join("main.rpx"),
        "(import lib only x as y)\n(val main y)\n",
    )
    .unwrap();
    let units = elaborate_module_tree(dir.join("main.rpx")).unwrap();
    assert!(units.iter().any(|u| u.name == "main"));

    let units = elaborate_module_tree(&dir).unwrap();
    assert!(units.len() >= 2);

    // Structured comment + token body path in split_imports
    let imports = parse_imports("(// note)\n(import lib)\n1").unwrap();
    assert_eq!(imports.len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn resolve_handle_perform_raise_nested_deep() {
    let r = resolve_language_source(
        r#"
(val main
  (handle outer (fn (m k) (k m))
    (handle inner (fn (m k) (forward k))
      (handle failure (fn (e) e)
        (seq
          (perform outer "a")
          (perform inner "b")
          (raise "c")
          (or-raise (as-result (fn () 1))))))))
"#,
    );
    assert!(r.is_ok(), "{:?}", r.errors);
}
