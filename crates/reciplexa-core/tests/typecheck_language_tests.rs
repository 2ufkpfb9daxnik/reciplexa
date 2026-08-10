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
