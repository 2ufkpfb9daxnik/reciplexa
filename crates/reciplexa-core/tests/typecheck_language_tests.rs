//! TYP-001: language-kernel typecheck via Core infer (not document surface).

use reciplexa_core::{typecheck_language_source, CoreType};

#[test]
fn identity_app_types_as_number() {
    let ty = typecheck_language_source("(val main ((fn (x) x) 1))").unwrap();
    assert_eq!(ty, CoreType::Number);
}

#[test]
fn dynamic_unifies_as_gradual_stub() {
    use reciplexa_core::unify::{unify, Subst};
    let mut s = Subst::new();
    assert!(unify(&CoreType::Dynamic, &CoreType::Number, &mut s).is_ok());
    assert!(unify(&CoreType::String, &CoreType::Dynamic, &mut s).is_ok());
}

#[test]
fn match_non_exhaustive_errors() {
    let err = typecheck_language_source(
        r#"
(data Option (None) (Some x))
(val main (match (Some 1) ((Some x) x)))
"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("non-exhaustive"),
        "unexpected: {}",
        err.message
    );
}

#[test]
fn match_exhaustive_option_ok() {
    let ty = typecheck_language_source(
        r#"
(data Option (None) (Some x))
(val main (match (Some 1) (None 0) ((Some x) x)))
"#,
    )
    .unwrap();
    assert_eq!(ty, CoreType::Number);
}

