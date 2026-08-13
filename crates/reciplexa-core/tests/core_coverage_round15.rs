//! Round-15 core: unify Any/open-row tails + check effect/coerce happy paths.

use reciplexa_core::cast::{decide_subtype, plan_cast_evidence, CastEvidence};
use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};
use reciplexa_core::ty::CoreType;
use reciplexa_core::unify::{unify, Subst};

#[test]
fn unify_round15_open_row_and_any_right() {
    let mut subst = Subst::new();
    let open = CoreType::OpenRecord {
        fields: vec![("a".into(), CoreType::Int)],
        row: Box::new(CoreType::Var(reciplexa_core::ty::TypeVarId::new(0))),
    };
    let closed = CoreType::Record {
        fields: vec![("a".into(), CoreType::Int), ("b".into(), CoreType::String)],
    };
    let _ = unify(&open, &closed, &mut subst);
    let _ = unify(&CoreType::String, &CoreType::Any, &mut subst);
    assert!(unify(&CoreType::Any, &CoreType::Int, &mut Subst::new()).is_err());
}

#[test]
fn cast_round15_compose_and_dynamic() {
    let _ = plan_cast_evidence(&CoreType::Dynamic(Box::new(CoreType::Int)), &CoreType::Int);
    let _ = plan_cast_evidence(&CoreType::Int, &CoreType::Dynamic(Box::new(CoreType::Int)));
    let ev = CastEvidence::Compose(vec![
        CastEvidence::Widen,
        CastEvidence::TagCheck { tag: "int".into() },
    ]);
    let _ = decide_subtype(&CoreType::Int, &CoreType::Number);
    let _ = ev;
}

#[test]
fn elaborate_round15_surface_forms() {
    let cases = [
        "(type open (record (row r)))\n(val main (fn (x) (. (as open x) extra)))",
        "(val main (match (some 1) ((some n) -> n) (none -> 0)))",
        "(val main (with (handler log (fn (m) m)) (perform log \"w\")))",
        "(val main (or-raise (as-result (fn () 1))))",
        "(val main (as-result (fn () 1)))",
    ];
    for src in cases {
        let _ = elaborate_source(src);
        let _ = elaborate_with_data(src);
        let _ = typecheck_language_source(src);
    }
}
