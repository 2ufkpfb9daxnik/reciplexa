//! DD-TYP-DYN: cast evidence, try-cast, check-cast, and `any`.

use reciplexa_core::{
    cast::{plan_cast_evidence, CastEvidence},
    typecheck_language_source, CoreType,
};

#[test]
fn any_type_parses_and_unifies_with_primitives() {
    use reciplexa_core::unify::{unify, Subst};
    let mut s = Subst::new();
    assert!(unify(&CoreType::Int, &CoreType::Any, &mut s).is_ok());
    assert!(unify(&CoreType::String, &CoreType::Any, &mut s).is_ok());
    assert!(unify(&CoreType::Any, &CoreType::Int, &mut s).is_err());
}

#[test]
fn plan_cast_dynamic_to_int_is_tag_check() {
    assert_eq!(
        plan_cast_evidence(&CoreType::Dynamic, &CoreType::Int),
        Some(CastEvidence::TagCheck {
            tag: "int".to_string()
        })
    );
}

#[test]
fn plan_cast_widen_to_dynamic() {
    assert_eq!(
        plan_cast_evidence(&CoreType::Int, &CoreType::Dynamic),
        Some(CastEvidence::Widen)
    );
}

#[test]
fn try_cast_literal_int_types_as_option_int() {
    let ty = typecheck_language_source("(val main (try-cast 1 int))").unwrap();
    assert_eq!(
        ty,
        CoreType::App {
            ctor: "option".into(),
            args: vec![CoreType::Int],
        }
    );
}

#[test]
fn check_cast_literal_string_types_as_result_string() {
    let ty = typecheck_language_source("(val main (check-cast \"a\" string))").unwrap();
    assert_eq!(
        ty,
        CoreType::App {
            ctor: "result".into(),
            args: vec![CoreType::String, CoreType::String],
        }
    );
}

#[test]
fn try_cast_to_never_is_rejected() {
    let err = typecheck_language_source("(val main (try-cast 1 never))").unwrap_err();
    assert!(
        err.message.contains("incompatible") || err.message.contains("impossible"),
        "unexpected: {}",
        err.message
    );
}
