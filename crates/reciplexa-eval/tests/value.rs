use reciplexa_core::expr::CoreExpr;
use reciplexa_core::ty::{CoreType, EffectRow, TypeVarId};
use reciplexa_eval::RuntimeValue;
use std::collections::HashMap;

#[test]
fn display_all_variants() {
    assert_eq!(RuntimeValue::Unit.to_string(), "unit");
    assert_eq!(RuntimeValue::Number(3.5).to_string(), "3.5");
    assert_eq!(RuntimeValue::String("hi".into()).to_string(), "\"hi\"");
    assert_eq!(RuntimeValue::Bool(true).to_string(), "true");
    assert_eq!(RuntimeValue::Bool(false).to_string(), "false");
    assert_eq!(
        RuntimeValue::ShapeTag("circle".into()).to_string(),
        "shape:circle"
    );
    assert_eq!(
        RuntimeValue::Closure {
            params: vec!["x".into()],
            body: CoreExpr::Lit(reciplexa_core::expr::CoreLiteral::Number(0.0)),
            env: std::rc::Rc::new(std::cell::RefCell::new(HashMap::new())),
        }
        .to_string(),
        "closure(x)"
    );
    assert_eq!(
        RuntimeValue::Record(vec![("a".into(), RuntimeValue::Number(1.0))]).to_string(),
        "record{a: 1}"
    );
    assert_eq!(
        RuntimeValue::Variant {
            tag: "Ok".into(),
            payload: Some(Box::new(RuntimeValue::Number(1.0))),
        }
        .to_string(),
        "Ok(1)"
    );
    assert_eq!(
        RuntimeValue::Variant {
            tag: "Done".into(),
            payload: None,
        }
        .to_string(),
        "Done"
    );
}

#[test]
fn debug_format_is_stable() {
    let v = RuntimeValue::Number(42.0);
    assert!(format!("{v:?}").contains("42.0"));
    let rec = RuntimeValue::Record(vec![]);
    assert!(format!("{rec:?}").contains("Record"));
}

#[test]
fn equality_and_clone() {
    let a = RuntimeValue::String("x".into());
    let b = a.clone();
    assert_eq!(a, b);
    assert_ne!(a, RuntimeValue::Unit);
}

#[test]
fn ty_maps_all_variants() {
    assert_eq!(RuntimeValue::Unit.ty(), CoreType::Unit);
    assert_eq!(RuntimeValue::Number(1.0).ty(), CoreType::Number);
    assert_eq!(RuntimeValue::String("s".into()).ty(), CoreType::String);
    assert_eq!(RuntimeValue::Bool(true).ty(), CoreType::Bool);
    assert_eq!(RuntimeValue::ShapeTag("rect".into()).ty(), CoreType::Shape);
    let closure_ty = RuntimeValue::Closure {
        params: vec!["x".into()],
        body: CoreExpr::Lit(reciplexa_core::expr::CoreLiteral::Number(0.0)),
        env: std::rc::Rc::new(std::cell::RefCell::new(HashMap::new())),
    }
    .ty();
    assert_eq!(
        closure_ty,
        CoreType::Fun {
            args: vec![CoreType::Var(TypeVarId::new(0))],
            ret: Box::new(CoreType::Unit),
            effects: EffectRow::default(),
        }
    );
    let rec_ty = RuntimeValue::Record(vec![
        ("x".into(), RuntimeValue::Number(1.0)),
        ("y".into(), RuntimeValue::String("z".into())),
    ])
    .ty();
    assert_eq!(
        rec_ty,
        CoreType::Record {
            fields: vec![
                ("x".into(), CoreType::Number),
                ("y".into(), CoreType::String),
            ],
        }
    );
    let var_ty = RuntimeValue::Variant {
        tag: "Some".into(),
        payload: Some(Box::new(RuntimeValue::Number(2.0))),
    }
    .ty();
    assert_eq!(
        var_ty,
        CoreType::Variant {
            variants: vec![("Some".into(), Some(CoreType::Number))],
        }
    );
    let nullary = RuntimeValue::Variant {
        tag: "None".into(),
        payload: None,
    }
    .ty();
    if let CoreType::Variant { variants } = nullary {
        assert_eq!(variants[0].1, None);
    } else {
        panic!("expected variant type");
    }
}

#[test]
fn ty_nullary_variant_and_unit_record() {
    assert_eq!(
        RuntimeValue::Variant {
            tag: "None".into(),
            payload: None,
        }
        .ty(),
        CoreType::Variant {
            variants: vec![("None".into(), None)],
        }
    );
    assert_eq!(
        RuntimeValue::Record(vec![("u".into(), RuntimeValue::Unit)]).ty(),
        CoreType::Record {
            fields: vec![("u".into(), CoreType::Unit)],
        }
    );
}

#[test]
fn closure_ty_has_fun_shape() {
    let v = RuntimeValue::Closure {
        params: vec!["n".into()],
        body: CoreExpr::Lit(reciplexa_core::expr::CoreLiteral::Number(0.0)),
        env: std::rc::Rc::new(std::cell::RefCell::new(HashMap::new())),
    };
    if let CoreType::Fun { args, ret, effects } = v.ty() {
        assert_eq!(args.len(), 1);
        assert_eq!(*ret, CoreType::Unit);
        assert_eq!(effects, EffectRow::default());
    } else {
        panic!("expected Fun");
    }
}

#[test]
fn record_ty_preserves_field_types() {
    if let CoreType::Record { fields } =
        RuntimeValue::Record(vec![("n".into(), RuntimeValue::Number(1.0))]).ty()
    {
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].0, "n");
        assert_eq!(fields[0].1, CoreType::Number);
    } else {
        panic!("expected Record");
    }
}

#[test]
fn display_formats_multi_field_record_and_variants() {
    let rec = RuntimeValue::Record(vec![
        ("a".into(), RuntimeValue::Number(1.0)),
        ("b".into(), RuntimeValue::String("x".into())),
    ]);
    let s = format!("{rec}");
    assert!(s.contains("a: 1"));
    assert!(s.contains(", "));
    assert!(s.contains("b: \"x\""));
    assert_eq!(
        format!(
            "{}",
            RuntimeValue::Variant {
                tag: "Ok".into(),
                payload: Some(Box::new(RuntimeValue::Number(1.0))),
            }
        ),
        "Ok(1)"
    );
    assert_eq!(
        format!(
            "{}",
            RuntimeValue::Variant {
                tag: "None".into(),
                payload: None,
            }
        ),
        "None"
    );
    assert_eq!(
        format!(
            "{}",
            RuntimeValue::Closure {
                params: vec!["x".into()],
                body: CoreExpr::Lit(reciplexa_core::expr::CoreLiteral::Number(0.0)),
                env: std::rc::Rc::new(std::cell::RefCell::new(HashMap::new())),
            }
        ),
        "closure(x)"
    );
    assert_eq!(
        format!("{}", RuntimeValue::ShapeTag("circle".into())),
        "shape:circle"
    );
    assert_eq!(format!("{}", RuntimeValue::Unit), "unit");
}
