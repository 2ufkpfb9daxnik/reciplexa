//! Integration tests moved from src/unify.rs for region coverage.

use reciplexa_core::ty::*;
use reciplexa_core::unify::*;

#[test]
fn unifies_function_types() {
    let mut s = Subst::new();
    let a = CoreType::Fun {
        args: vec![CoreType::Number],
        ret: Box::new(CoreType::Number),
        effects: EffectRow::default(),
    };
    let b = CoreType::Fun {
        args: vec![CoreType::Number],
        ret: Box::new(CoreType::Number),
        effects: EffectRow::default(),
    };
    assert!(unify(&a, &b, &mut s).is_ok());
}

#[test]
fn fresh_var_unifies_with_number() {
    let mut s = Subst::new();
    let v = s.fresh_var();
    assert!(unify(&CoreType::Var(v), &CoreType::Number, &mut s).is_ok());
    assert_eq!(s.apply(&CoreType::Var(v)), CoreType::Number);
}

#[test]
fn unifies_primitives() {
    let mut s = Subst::new();
    for ty in [
        CoreType::Number,
        CoreType::String,
        CoreType::Color,
        CoreType::Shape,
        CoreType::Unit,
    ] {
        assert!(unify(&ty, &ty, &mut s).is_ok());
    }
}

#[test]
fn primitive_mismatch() {
    let mut s = Subst::new();
    assert!(unify(&CoreType::Number, &CoreType::String, &mut s).is_err());
}

#[test]
fn occurs_check_rejects_infinite_type() {
    let mut s = Subst::new();
    let v = s.fresh_var();
    let fun = CoreType::Fun {
        args: vec![CoreType::Var(v)],
        ret: Box::new(CoreType::Var(v)),
        effects: EffectRow::default(),
    };
    assert!(unify(&CoreType::Var(v), &fun, &mut s).is_err());
}

#[test]
fn self_bind_var_is_ok() {
    let mut s = Subst::new();
    let v = s.fresh_var();
    assert!(s.bind(v, CoreType::Var(v)).is_ok());
}

#[test]
fn function_arg_count_mismatch() {
    let mut s = Subst::new();
    let a = CoreType::Fun {
        args: vec![CoreType::Number],
        ret: Box::new(CoreType::Unit),
        effects: EffectRow::default(),
    };
    let b = CoreType::Fun {
        args: vec![CoreType::Number, CoreType::Number],
        ret: Box::new(CoreType::Unit),
        effects: EffectRow::default(),
    };
    assert!(unify(&a, &b, &mut s).is_err());
}

#[test]
fn function_effect_mismatch() {
    let mut s = Subst::new();
    let a = CoreType::Fun {
        args: vec![],
        ret: Box::new(CoreType::Unit),
        effects: EffectRow {
            ops: vec!["log".into()],
        },
    };
    let b = CoreType::Fun {
        args: vec![],
        ret: Box::new(CoreType::Unit),
        effects: EffectRow::default(),
    };
    assert!(unify(&a, &b, &mut s).is_err());
}

#[test]
fn record_field_name_mismatch() {
    let mut s = Subst::new();
    let a = CoreType::Record {
        fields: vec![("x".into(), CoreType::Number)],
    };
    let b = CoreType::Record {
        fields: vec![("y".into(), CoreType::Number)],
    };
    assert!(unify(&a, &b, &mut s).is_err());
}

#[test]
fn record_unifies_fields() {
    let mut s = Subst::new();
    let a = CoreType::Record {
        fields: vec![("x".into(), CoreType::Number)],
    };
    let b = CoreType::Record {
        fields: vec![("x".into(), CoreType::Number)],
    };
    assert!(unify(&a, &b, &mut s).is_ok());
}

#[test]
fn variant_payload_mismatch() {
    let mut s = Subst::new();
    let a = CoreType::Variant {
        variants: vec![("Some".into(), Some(CoreType::Number))],
    };
    let b = CoreType::Variant {
        variants: vec![("Some".into(), None)],
    };
    assert!(unify(&a, &b, &mut s).is_err());
}

#[test]
fn variant_unifies_nullary() {
    let mut s = Subst::new();
    let a = CoreType::Variant {
        variants: vec![("Done".into(), None)],
    };
    let b = CoreType::Variant {
        variants: vec![("Done".into(), None)],
    };
    assert!(unify(&a, &b, &mut s).is_ok());
}

#[test]
fn apply_traverses_nested_types() {
    let mut s = Subst::new();
    let v = s.fresh_var();
    s.bind(v, CoreType::String).unwrap();
    let ty = CoreType::Fun {
        args: vec![CoreType::Var(v)],
        ret: Box::new(CoreType::Record {
            fields: vec![("f".into(), CoreType::Var(v))],
        }),
        effects: EffectRow::default(),
    };
    let applied = s.apply(&ty);
    assert_eq!(
        applied,
        CoreType::Fun {
            args: vec![CoreType::String],
            ret: Box::new(CoreType::Record {
                fields: vec![("f".into(), CoreType::String)],
            }),
            effects: EffectRow::default(),
        }
    );
}

#[test]
fn fresh_var_ids_increment() {
    let mut s = Subst::new();
    let a = s.fresh_var();
    let b = s.fresh_var();
    assert_ne!(a, b);
}

#[test]
fn record_length_mismatch() {
    let mut s = Subst::new();
    let a = CoreType::Record {
        fields: vec![("x".into(), CoreType::Number)],
    };
    let b = CoreType::Record {
        fields: vec![
            ("x".into(), CoreType::Number),
            ("y".into(), CoreType::Number),
        ],
    };
    assert!(matches!(
        unify(&a, &b, &mut s),
        Err(UnifyError::Mismatch { .. })
    ));
}

#[test]
fn variant_length_and_tag_mismatch() {
    let mut s = Subst::new();
    let a = CoreType::Variant {
        variants: vec![("A".into(), None)],
    };
    let b = CoreType::Variant {
        variants: vec![("A".into(), None), ("B".into(), None)],
    };
    assert!(unify(&a, &b, &mut s).is_err());
    let c = CoreType::Variant {
        variants: vec![("B".into(), None)],
    };
    assert!(unify(&a, &c, &mut s).is_err());
}

#[test]
fn occurs_check_in_record_and_variant() {
    let mut s = Subst::new();
    let v = s.fresh_var();
    let rec = CoreType::Record {
        fields: vec![("f".into(), CoreType::Var(v))],
    };
    assert!(unify(&CoreType::Var(v), &rec, &mut s).is_err());
    let mut s2 = Subst::new();
    let v2 = s2.fresh_var();
    let var = CoreType::Variant {
        variants: vec![("Some".into(), Some(CoreType::Var(v2)))],
    };
    assert!(unify(&CoreType::Var(v2), &var, &mut s2).is_err());
}

#[test]
fn unifies_variant_with_payload() {
    let mut s = Subst::new();
    let a = CoreType::Variant {
        variants: vec![("Some".into(), Some(CoreType::Number))],
    };
    let b = CoreType::Variant {
        variants: vec![("Some".into(), Some(CoreType::Number))],
    };
    assert!(unify(&a, &b, &mut s).is_ok());
}

#[test]
fn bind_var_to_itself_is_ok() {
    let mut s = Subst::new();
    let v = TypeVarId::new(7);
    assert!(s.bind(v, CoreType::Var(v)).is_ok());
}

#[test]
fn occurs_in_fun_args_and_record_fields() {
    let mut s = Subst::new();
    let v = TypeVarId::new(0);
    let fun = CoreType::Fun {
        args: vec![CoreType::Var(v)],
        ret: Box::new(CoreType::Unit),
        effects: Default::default(),
    };
    assert!(s.bind(v, fun).is_err());
    let mut s = Subst::new();
    let v = TypeVarId::new(1);
    let rec = CoreType::Record {
        fields: vec![("x".into(), CoreType::Var(v))],
    };
    assert!(s.bind(v, rec).is_err());
}

#[test]
fn function_arg_and_return_mismatch() {
    let mut s = Subst::new();
    let arg_mismatch = (
        CoreType::Fun {
            args: vec![CoreType::Number],
            ret: Box::new(CoreType::Unit),
            effects: EffectRow::default(),
        },
        CoreType::Fun {
            args: vec![CoreType::String],
            ret: Box::new(CoreType::Unit),
            effects: EffectRow::default(),
        },
    );
    assert!(unify(&arg_mismatch.0, &arg_mismatch.1, &mut s).is_err());

    let mut s = Subst::new();
    let ret_mismatch = (
        CoreType::Fun {
            args: vec![CoreType::Number],
            ret: Box::new(CoreType::Number),
            effects: EffectRow::default(),
        },
        CoreType::Fun {
            args: vec![CoreType::Number],
            ret: Box::new(CoreType::String),
            effects: EffectRow::default(),
        },
    );
    assert!(unify(&ret_mismatch.0, &ret_mismatch.1, &mut s).is_err());
}

#[test]
fn record_and_variant_value_mismatch() {
    let mut s = Subst::new();
    assert!(unify(
        &CoreType::Record {
            fields: vec![("x".into(), CoreType::Number)],
        },
        &CoreType::Record {
            fields: vec![("x".into(), CoreType::String)],
        },
        &mut s
    )
    .is_err());

    let mut s = Subst::new();
    assert!(unify(
        &CoreType::Variant {
            variants: vec![("Some".into(), Some(CoreType::Number))],
        },
        &CoreType::Variant {
            variants: vec![("Some".into(), Some(CoreType::String))],
        },
        &mut s
    )
    .is_err());
}

#[test]
fn occurs_in_function_argument_position() {
    let mut s = Subst::new();
    let v = s.fresh_var();
    let fun = CoreType::Fun {
        args: vec![CoreType::Var(v)],
        ret: Box::new(CoreType::Unit),
        effects: EffectRow::default(),
    };
    assert!(unify(&CoreType::Var(v), &fun, &mut s).is_err());
}

#[test]
fn apply_chains_substitutions_and_nullary_variant() {
    let mut s = Subst::new();
    let v1 = s.fresh_var();
    let v2 = s.fresh_var();
    s.bind(v1, CoreType::Var(v2)).unwrap();
    s.bind(v2, CoreType::Number).unwrap();
    assert_eq!(s.apply(&CoreType::Var(v1)), CoreType::Number);

    let nullary = CoreType::Variant {
        variants: vec![("Done".into(), None)],
    };
    assert_eq!(s.apply(&nullary), nullary);
}

#[test]
fn bind_occurs_in_function_return() {
    let mut s = Subst::new();
    let v = s.fresh_var();
    let fun = CoreType::Fun {
        args: vec![CoreType::Unit],
        ret: Box::new(CoreType::Var(v)),
        effects: EffectRow::default(),
    };
    assert!(s.bind(v, fun).is_err());
}

#[test]
fn bind_different_vars_noop_path() {
    let mut s = Subst::new();
    let a = s.fresh_var();
    let b = s.fresh_var();
    assert!(s.bind(a, CoreType::Var(b)).is_ok());
    assert_eq!(s.apply(&CoreType::Var(a)), CoreType::Var(b));
}

#[test]
fn closed_records_unify_same_fields() {
    let mut s = Subst::new();
    let a = CoreType::Record {
        fields: vec![
            ("x".into(), CoreType::Number),
            ("y".into(), CoreType::String),
        ],
    };
    let b = CoreType::Record {
        fields: vec![
            ("x".into(), CoreType::Number),
            ("y".into(), CoreType::String),
        ],
    };
    assert!(unify(&a, &b, &mut s).is_ok());
}

#[test]
fn closed_record_missing_field_errors() {
    let mut s = Subst::new();
    let a = CoreType::Record {
        fields: vec![
            ("x".into(), CoreType::Number),
            ("y".into(), CoreType::String),
        ],
    };
    let b = CoreType::Record {
        fields: vec![("x".into(), CoreType::Number)],
    };
    assert!(matches!(
        unify(&a, &b, &mut s),
        Err(UnifyError::Mismatch { .. })
    ));
}

#[test]
fn lacks_stub_unifies_same_label() {
    let mut s = Subst::new();
    let a = CoreType::Lacks {
        label: "z".into(),
        row: Box::new(CoreType::Record {
            fields: vec![("x".into(), CoreType::Number)],
        }),
    };
    let b = CoreType::Lacks {
        label: "z".into(),
        row: Box::new(CoreType::Record {
            fields: vec![("x".into(), CoreType::Number)],
        }),
    };
    assert!(unify(&a, &b, &mut s).is_ok());
}

#[test]
fn lacks_stub_label_mismatch_errors() {
    let mut s = Subst::new();
    let a = CoreType::Lacks {
        label: "z".into(),
        row: Box::new(CoreType::Unit),
    };
    let b = CoreType::Lacks {
        label: "w".into(),
        row: Box::new(CoreType::Unit),
    };
    assert!(matches!(
        unify(&a, &b, &mut s),
        Err(UnifyError::Mismatch { .. })
    ));
}
