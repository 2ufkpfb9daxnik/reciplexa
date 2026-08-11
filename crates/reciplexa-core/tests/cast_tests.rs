//! DD-TYP-DYN: cast evidence, try-cast, check-cast, and `any`.

use reciplexa_core::{
    cast::{
        cast_success_type, compose_evidence, is_runtime_checkable, judge_dynamic_use,
        plan_cast_evidence, CastEvidence, DynamicUseJudgment,
    },
    coerce_to_static, typecheck_language_source, CoreExpr, CoreLiteral, CoreType,
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
        plan_cast_evidence(&CoreType::dyn_any(), &CoreType::Int),
        Some(CastEvidence::TagCheck {
            tag: "int".to_string()
        })
    );
}

#[test]
fn plan_cast_widen_to_dynamic() {
    assert_eq!(
        plan_cast_evidence(&CoreType::Int, &CoreType::dyn_any()),
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

#[test]
fn three_way_judgment_fully_included_and_disjoint() {
    assert_eq!(
        judge_dynamic_use(&CoreType::Int, &CoreType::Number),
        DynamicUseJudgment::FullyIncluded
    );
    assert_eq!(
        judge_dynamic_use(&CoreType::String, &CoreType::Number),
        DynamicUseJudgment::Disjoint
    );
}

#[test]
fn cast_success_intersect_union_number() {
    let success = cast_success_type(
        &CoreType::Union(vec![CoreType::Int, CoreType::String]),
        &CoreType::Number,
    );
    assert_eq!(success, CoreType::Int);
}

#[test]
fn coerce_inserts_cast_on_partial_overlap() {
    let expr = CoreExpr::Var("x".into());
    let found = CoreType::dynamic_bound(CoreType::Union(vec![CoreType::Int, CoreType::String]));
    let out = coerce_to_static(expr, &found, &CoreType::Number, 7).unwrap();
    assert!(matches!(
        out,
        CoreExpr::Cast {
            evidence: CastEvidence::TagCheck { tag },
            target: CoreType::Number,
            cast_id: 7,
            ..
        } if tag == "number"
    ));
}

#[test]
fn coerce_rejects_disjoint_dynamic_use() {
    let err = coerce_to_static(
        CoreExpr::Lit(CoreLiteral::Int(1)),
        &CoreType::dynamic_bound(CoreType::String),
        &CoreType::Number,
        1,
    )
    .unwrap_err();
    assert!(err.message.contains("disjoint") || err.message.contains("never"));
}

#[test]
fn evidence_compose_is_pure_rewrite() {
    let e = compose_evidence(vec![
        CastEvidence::Identity,
        CastEvidence::Widen,
        CastEvidence::Identity,
    ]);
    assert_eq!(e, CastEvidence::Widen);
}

#[test]
fn runtime_checkable_primitives_and_rejects_not() {
    assert!(is_runtime_checkable(&CoreType::Bool));
    assert!(is_runtime_checkable(&CoreType::Record {
        fields: vec![("a".into(), CoreType::Int)]
    }));
    assert!(!is_runtime_checkable(&CoreType::Not(Box::new(
        CoreType::Int
    ))));
}

#[test]
fn bounded_dynamic_elaborates() {
    use reciplexa_core::elaborate_with_data;
    let (_, data) = elaborate_with_data(
        r#"
(type-alias dnum (dynamic number))
(val main 1)
"#,
    )
    .unwrap();
    assert!(matches!(
        data.type_aliases.get("dnum"),
        Some(CoreType::Dynamic(b)) if matches!(b.as_ref(), CoreType::Number)
    ));
}
