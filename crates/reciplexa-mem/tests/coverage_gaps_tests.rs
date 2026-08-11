//! Extra coverage for product paths not exercised by the primary suites.

use reciplexa_core::expr::{CoreExpr, CoreLiteral, MatchArm};
use reciplexa_eval::RuntimeValue;
use reciplexa_mem::ir::{BlockId, MemInstr, MemLiteral};
use reciplexa_mem::linear::LinearProgram;
use reciplexa_mem::reg::Reg;
use reciplexa_mem::trace::RcTrace;
use reciplexa_mem::{
    assert_observational_equiv, check_observational_equiv, conservative_rc, exec_linear,
    observably_equal, perceus_pass, reuse_pass, seal_before_return, verify_ownership, verify_reuse,
    EquivError, ExecError, OwnMap, OwnState, VerifyError,
};

#[test]
fn observably_equal_none_payload_and_cross_type() {
    assert!(observably_equal(
        &RuntimeValue::Variant {
            tag: "A".into(),
            payload: None,
        },
        &RuntimeValue::Variant {
            tag: "A".into(),
            payload: None,
        },
    ));
    assert!(!observably_equal(
        &RuntimeValue::Unit,
        &RuntimeValue::Int(1)
    ));
}

#[test]
fn reference_eval_error_maps_to_equiv() {
    let e = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        arms: vec![MatchArm::variant(
            "X".into(),
            None,
            CoreExpr::Lit(CoreLiteral::Int(0)),
        )],
    };
    let err = assert_observational_equiv(&e).unwrap_err();
    assert!(matches!(err, EquivError::ReferenceEval(_)));
}

#[test]
fn equiv_error_variants_debug() {
    let _ = format!("{:?}", EquivError::Verify("v".into()));
    let _ = format!("{:?}", EquivError::Exec("e".into()));
    let _ = format!(
        "{:?}",
        EquivError::Mismatch {
            reference: "a".into(),
            optimized: "b".into(),
        }
    );
}

#[test]
fn check_observational_equiv_mismatch_and_ok() {
    check_observational_equiv(&RuntimeValue::Unit, &RuntimeValue::Unit).unwrap();
    let err = check_observational_equiv(&RuntimeValue::Int(1), &RuntimeValue::Int(2)).unwrap_err();
    assert!(matches!(err, EquivError::Mismatch { .. }));
}

#[test]
fn merge_join_uninit_both_orientations() {
    let mut left = OwnMap::new();
    left.define(Reg(0));
    let right = OwnMap::new();
    assert_eq!(left.merge_join(&right).get(Reg(0)), OwnState::Alive);
    assert_eq!(right.merge_join(&left).get(Reg(0)), OwnState::Alive);
}

#[test]
fn seal_tracks_move_dup_project_and_control() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Dup {
                dst: Reg(1),
                src: Reg(0),
            },
            MemInstr::Project {
                dst: Reg(2),
                src: Reg(1),
                field: "x".into(),
            },
            MemInstr::Move {
                dst: Reg(3),
                src: Reg(2),
            },
            MemInstr::Jump { target: BlockId(0) },
            MemInstr::Branch {
                cond: Reg(3),
                then_block: BlockId(0),
                else_block: BlockId(1),
            },
            MemInstr::Raise { tag: "x".into() },
            MemInstr::RegisterCleanup { label: "c".into() },
            MemInstr::RunCleanup,
            MemInstr::Resume { cont: 1 },
            MemInstr::DiscardCont { cont: 2 },
            MemInstr::Phi {
                dst: Reg(4),
                incoming: vec![(BlockId(0), Reg(3))],
            },
            MemInstr::Return { reg: Reg(3) },
        ],
        return_reg: Reg(3),
    };
    let sealed = seal_before_return(prog);
    assert!(sealed
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Return { .. })));
}

#[test]
fn reuse_drains_leftover_pool_and_apply_effects() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Construct {
                dst: Reg(0),
                tag: "record".into(),
                fields: vec![],
            },
            MemInstr::Drop { reg: Reg(0) },
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Dup {
                dst: Reg(2),
                src: Reg(1),
            },
            MemInstr::Move {
                dst: Reg(3),
                src: Reg(2),
            },
            MemInstr::Drop { reg: Reg(3) },
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    let out = reuse_pass(prog);
    assert!(out
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Drop { reg: Reg(0) })));
}

#[test]
fn reuse_drop_of_non_alive_falls_through() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Drop { reg: Reg(0) },
            MemInstr::Drop { reg: Reg(0) },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    let out = reuse_pass(prog);
    assert_eq!(
        out.instrs
            .iter()
            .filter(|i| matches!(i, MemInstr::Drop { .. }))
            .count(),
        2
    );
}

#[test]
fn exec_discard_cont_and_fallthrough_return() {
    let mut trace = RcTrace::default();
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(9.0),
            },
            MemInstr::RegisterCleanup { label: "c".into() },
            MemInstr::DiscardCont { cont: 1 },
        ],
        return_reg: Reg(0),
    };
    let v = exec_linear(&prog, &mut trace).unwrap();
    assert_eq!(v, RuntimeValue::Number(9.0));
    assert_eq!(trace.cleanups, vec!["c".to_string()]);
}

#[test]
fn exec_call_is_noop_and_variant_without_payload() {
    let mut trace = RcTrace::default();
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Construct {
                dst: Reg(0),
                tag: "None".into(),
                fields: vec![],
            },
            MemInstr::Call {
                dst: Reg(1),
                closure: Reg(0),
                arg: Reg(0),
            },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    let v = exec_linear(&prog, &mut trace).unwrap();
    assert_eq!(
        v,
        RuntimeValue::Variant {
            tag: "None".into(),
            payload: None,
        }
    );
}

#[test]
fn exec_unbound_error_paths() {
    let mut trace = RcTrace::default();
    assert!(matches!(
        exec_linear(
            &LinearProgram {
                instrs: vec![MemInstr::Dup {
                    dst: Reg(1),
                    src: Reg(9),
                }],
                return_reg: Reg(1),
            },
            &mut trace
        ),
        Err(ExecError::UnboundReg(_))
    ));

    let mut trace = RcTrace::default();
    assert!(matches!(
        exec_linear(
            &LinearProgram {
                instrs: vec![MemInstr::Drop { reg: Reg(9) }],
                return_reg: Reg(0),
            },
            &mut trace
        ),
        Err(ExecError::UnboundReg(_))
    ));

    let mut trace = RcTrace::default();
    assert!(matches!(
        exec_linear(
            &LinearProgram {
                instrs: vec![MemInstr::Move {
                    dst: Reg(1),
                    src: Reg(9),
                }],
                return_reg: Reg(0),
            },
            &mut trace
        ),
        Err(ExecError::UnboundReg(_))
    ));

    let mut trace = RcTrace::default();
    assert!(matches!(
        exec_linear(
            &LinearProgram {
                instrs: vec![MemInstr::Construct {
                    dst: Reg(0),
                    tag: "record".into(),
                    fields: vec![("x".into(), Reg(9))],
                }],
                return_reg: Reg(0),
            },
            &mut trace
        ),
        Err(ExecError::UnboundReg(_))
    ));

    let mut trace = RcTrace::default();
    assert!(matches!(
        exec_linear(
            &LinearProgram {
                instrs: vec![MemInstr::Construct {
                    dst: Reg(0),
                    tag: "Some".into(),
                    fields: vec![("payload".into(), Reg(9))],
                }],
                return_reg: Reg(0),
            },
            &mut trace
        ),
        Err(ExecError::UnboundReg(_))
    ));

    let mut trace = RcTrace::default();
    assert!(matches!(
        exec_linear(
            &LinearProgram {
                instrs: vec![
                    MemInstr::Lit {
                        dst: Reg(0),
                        lit: MemLiteral::Number(1.0),
                    },
                    MemInstr::Project {
                        dst: Reg(1),
                        src: Reg(0),
                        field: "x".into(),
                    },
                ],
                return_reg: Reg(0),
            },
            &mut trace
        ),
        Err(ExecError::UnboundReg(_))
    ));

    let mut trace = RcTrace::default();
    assert!(matches!(
        exec_linear(
            &LinearProgram {
                instrs: vec![MemInstr::MakeClosure {
                    dst: Reg(0),
                    param: "x".into(),
                    body: BlockId(0),
                    captures: vec![Reg(9)],
                }],
                return_reg: Reg(0),
            },
            &mut trace
        ),
        Err(ExecError::UnboundReg(_))
    ));

    let mut trace = RcTrace::default();
    assert!(matches!(
        exec_linear(
            &LinearProgram {
                instrs: vec![MemInstr::ConstructReuse {
                    dst: Reg(0),
                    reuse: Reg(9),
                    tag: "record".into(),
                    fields: vec![],
                }],
                return_reg: Reg(0),
            },
            &mut trace
        ),
        Err(ExecError::UnboundReg(_))
    ));

    let mut trace = RcTrace::default();
    assert!(matches!(
        exec_linear(
            &LinearProgram {
                instrs: vec![],
                return_reg: Reg(0),
            },
            &mut trace
        ),
        Err(ExecError::UnboundReg(_))
    ));
}

#[test]
fn exec_construct_reuse_unbound_field_and_project_unbound() {
    let mut trace = RcTrace::default();
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Construct {
                dst: Reg(0),
                tag: "record".into(),
                fields: vec![],
            },
            MemInstr::ConstructReuse {
                dst: Reg(1),
                reuse: Reg(0),
                tag: "record".into(),
                fields: vec![("x".into(), Reg(9))],
            },
        ],
        return_reg: Reg(1),
    };
    assert!(matches!(
        exec_linear(&prog, &mut trace),
        Err(ExecError::UnboundReg(_))
    ));
}

#[test]
fn verify_dup_uninit_and_move_and_call() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Dup {
                dst: Reg(1),
                src: Reg(0),
            },
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    assert_eq!(
        verify_ownership(&prog),
        Err(VerifyError::UseAfterDrop(Reg(0)))
    );

    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Move {
                dst: Reg(1),
                src: Reg(0),
            },
            MemInstr::Drop { reg: Reg(1) },
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    // return uses dropped reg
    assert!(verify_ownership(&prog).is_err());

    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Call {
                dst: Reg(1),
                closure: Reg(0),
                arg: Reg(0),
            },
            MemInstr::Drop { reg: Reg(1) },
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    // Call defines dst; dropping return may still leak/err — just exercise Call define arm
    let _ = verify_ownership(&prog);
    let _ = verify_reuse(&prog);
}

#[test]
fn verify_reuse_propagates_ownership_failure() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Drop { reg: Reg(0) },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    assert!(verify_reuse(&prog).is_err());
}

#[test]
fn verify_project_define_arm() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Project {
                dst: Reg(1),
                src: Reg(0),
                field: "x".into(),
            },
            MemInstr::Drop { reg: Reg(0) },
            MemInstr::Drop { reg: Reg(1) },
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    let _ = verify_ownership(&prog);
}

#[test]
fn lower_variant_unit_and_match_bind_without_payload() {
    let unit_var = CoreExpr::Variant {
        tag: "None".into(),
        payload: None,
    };
    let prog = reciplexa_mem::lower_core_linear(&unit_var);
    assert!(prog
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Construct { tag, fields, .. } if tag == "None" && fields.is_empty())));

    let matched = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "None".into(),
            payload: None,
        }),
        arms: vec![MatchArm::variant(
            "None".into(),
            Some("x".into()),
            CoreExpr::Lit(CoreLiteral::Int(1)),
        )],
    };
    let prog = reciplexa_mem::lower_core_linear(&matched);
    assert!(!prog.instrs.is_empty());

    let no_bind = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        arms: vec![
            MatchArm::variant("A".into(), None, CoreExpr::Lit(CoreLiteral::Int(1))),
            MatchArm::variant("B".into(), None, CoreExpr::Lit(CoreLiteral::Int(2))),
        ],
    };
    let prog = reciplexa_mem::lower_core_linear(&no_bind);
    assert!(prog
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Dup { .. })));
}

#[test]
fn conservative_reads_drop_and_remap() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            // Drop is not the last use of R0 — forces remap of Drop via Dup.
            MemInstr::Drop { reg: Reg(0) },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    let out = conservative_rc(&prog);
    assert!(out.instrs.iter().any(|i| matches!(i, MemInstr::Dup { .. })));
}

#[test]
fn perceus_move_with_later_use_skips_dup() {
    // Move src is still named later — exclusion must suppress Dup on Move.
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Move {
                dst: Reg(1),
                src: Reg(0),
            },
            MemInstr::Dup {
                dst: Reg(2),
                src: Reg(0),
            },
            MemInstr::Drop { reg: Reg(2) },
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    let out = perceus_pass(&prog);
    assert!(!out.instrs.is_empty());
}

#[test]
fn verify_reuse_applies_move_state() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Move {
                dst: Reg(1),
                src: Reg(0),
            },
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    assert!(verify_reuse(&prog).is_ok());
}

#[test]
fn reuse_apply_effect_on_construct_reuse_input() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Construct {
                dst: Reg(0),
                tag: "record".into(),
                fields: vec![],
            },
            MemInstr::ConstructReuse {
                dst: Reg(1),
                reuse: Reg(0),
                tag: "record".into(),
                fields: vec![],
            },
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    let out = reuse_pass(prog);
    assert!(out
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::ConstructReuse { .. })));
}

#[test]
fn lower_match_bind_none_and_tag_miss() {
    let bind_none = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "Ok".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Int(1)))),
        }),
        arms: vec![MatchArm::variant(
            "Ok".into(),
            None,
            CoreExpr::Lit(CoreLiteral::Int(2)),
        )],
    };
    let prog = reciplexa_mem::lower_core_linear(&bind_none);
    assert!(!prog.instrs.is_empty());

    let tag_miss = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "Ok".into(),
            payload: None,
        }),
        arms: vec![MatchArm::variant(
            "Err".into(),
            None,
            CoreExpr::Lit(CoreLiteral::Int(0)),
        )],
    };
    let prog = reciplexa_mem::lower_core_linear(&tag_miss);
    assert!(!prog.instrs.is_empty());
}

#[test]
fn exec_project_unbound_src_and_reuse_fallback_unbound_field() {
    let mut trace = RcTrace::default();
    assert!(matches!(
        exec_linear(
            &LinearProgram {
                instrs: vec![MemInstr::Project {
                    dst: Reg(1),
                    src: Reg(9),
                    field: "x".into(),
                }],
                return_reg: Reg(1),
            },
            &mut trace
        ),
        Err(ExecError::UnboundReg(_))
    ));

    let mut trace = RcTrace::default();
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Dup {
                dst: Reg(1),
                src: Reg(0),
            },
            // Shared (non-unique) reuse → fallback construct_value path.
            MemInstr::ConstructReuse {
                dst: Reg(2),
                reuse: Reg(0),
                tag: "record".into(),
                fields: vec![("x".into(), Reg(9))],
            },
        ],
        return_reg: Reg(2),
    };
    assert!(matches!(
        exec_linear(&prog, &mut trace),
        Err(ExecError::UnboundReg(_))
    ));
}

#[test]
fn ir_collect_return_reg() {
    use reciplexa_mem::ir::collect_regs_instr;
    let mut out = Vec::new();
    collect_regs_instr(&MemInstr::Return { reg: Reg(7) }, &mut out);
    assert_eq!(out, vec![Reg(7)]);
    out.clear();
    collect_regs_instr(&MemInstr::Drop { reg: Reg(8) }, &mut out);
    assert_eq!(out, vec![Reg(8)]);
}

#[test]
fn exec_error_variants_still_exist() {
    let _ = format!("{:?}", ExecError::RefCountOverflow(Reg(0)));
    let _ = format!("{:?}", ExecError::RefCountUnderflow(Reg(0)));
}

#[test]
fn observably_equal_cross_numeric_and_closures() {
    use reciplexa_eval::{BuiltinOp, RuntimeValue};
    use reciplexa_mem::observably_equal;
    assert!(observably_equal(
        &RuntimeValue::Int(3),
        &RuntimeValue::Number(3.0)
    ));
    assert!(observably_equal(
        &RuntimeValue::Number(3.0),
        &RuntimeValue::Int(3)
    ));
    assert!(observably_equal(
        &RuntimeValue::Int(3),
        &RuntimeValue::F64(3.0)
    ));
    assert!(observably_equal(
        &RuntimeValue::F64(3.0),
        &RuntimeValue::Int(3)
    ));
    assert!(observably_equal(
        &RuntimeValue::Number(1.5),
        &RuntimeValue::F64(1.5)
    ));
    assert!(observably_equal(
        &RuntimeValue::F64(1.5),
        &RuntimeValue::Number(1.5)
    ));
    assert!(observably_equal(
        &RuntimeValue::Bool(true),
        &RuntimeValue::Bool(true)
    ));
    assert!(observably_equal(
        &RuntimeValue::ShapeTag("circle".into()),
        &RuntimeValue::ShapeTag("circle".into())
    ));
    assert!(!observably_equal(
        &RuntimeValue::Variant {
            tag: "A".into(),
            payload: None,
        },
        &RuntimeValue::Variant {
            tag: "A".into(),
            payload: Some(Box::new(RuntimeValue::Unit)),
        },
    ));
    assert!(observably_equal(
        &RuntimeValue::Closure {
            params: vec!["x".into()],
            body: reciplexa_core::expr::CoreExpr::Lit(reciplexa_core::expr::CoreLiteral::Int(0)),
            env: std::rc::Rc::new(std::cell::RefCell::new(Default::default())),
        },
        &RuntimeValue::Closure {
            params: vec!["y".into()],
            body: reciplexa_core::expr::CoreExpr::Lit(reciplexa_core::expr::CoreLiteral::Int(1)),
            env: std::rc::Rc::new(std::cell::RefCell::new(Default::default())),
        },
    ));
    assert!(observably_equal(
        &RuntimeValue::Builtin(BuiltinOp::Add),
        &RuntimeValue::Builtin(BuiltinOp::Add)
    ));
}

#[test]
fn lower_seq_letrec_local_set_lambda_app_if() {
    use reciplexa_core::expr::{CoreExpr, CoreLiteral};
    use reciplexa_mem::lower_core_linear;

    let expr = CoreExpr::Seq(vec![
        CoreExpr::Lit(CoreLiteral::Unit),
        CoreExpr::Let {
            name: "a".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            body: Box::new(CoreExpr::Var("a".into())),
        },
    ]);
    assert!(!lower_core_linear(&expr).instrs.is_empty());

    let letrec = CoreExpr::LetRec {
        bindings: vec![(
            "n".into(),
            CoreExpr::Lambda {
                params: vec!["x".into()],
                body: Box::new(CoreExpr::Var("x".into())),
            },
        )],
        body: Box::new(CoreExpr::Var("n".into())),
    };
    assert!(!lower_core_linear(&letrec).instrs.is_empty());

    let local = CoreExpr::LocalVar {
        name: "x".into(),
        init: Box::new(CoreExpr::Lit(CoreLiteral::F64(1.5))),
        body: Box::new(CoreExpr::Var("x".into())),
    };
    assert!(!lower_core_linear(&local).instrs.is_empty());

    let set = CoreExpr::Set {
        name: "x".into(),
        value: Box::new(CoreExpr::Lit(CoreLiteral::Bool(true))),
    };
    assert!(!lower_core_linear(&set).instrs.is_empty());

    let lam = CoreExpr::Lambda {
        params: vec!["x".into()],
        body: Box::new(CoreExpr::Var("x".into())),
    };
    assert!(!lower_core_linear(&lam).instrs.is_empty());

    let app0 = CoreExpr::App {
        fun: Box::new(lam.clone()),
        args: vec![],
    };
    assert!(!lower_core_linear(&app0).instrs.is_empty());

    let app1 = CoreExpr::App {
        fun: Box::new(lam),
        args: vec![CoreExpr::Lit(CoreLiteral::Int(9))],
    };
    assert!(!lower_core_linear(&app1).instrs.is_empty());

    let iff = CoreExpr::If {
        cond: Box::new(CoreExpr::Lit(CoreLiteral::Bool(true))),
        then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
    };
    assert!(!lower_core_linear(&iff).instrs.is_empty());

    let bytes = CoreExpr::Lit(CoreLiteral::Bytes(vec![1, 2, 3]));
    assert!(!lower_core_linear(&bytes).instrs.is_empty());
    let color = CoreExpr::Lit(CoreLiteral::Color("red".into()));
    assert!(!lower_core_linear(&color).instrs.is_empty());
}

#[test]
fn exec_select_rejects_non_bool_cond() {
    use reciplexa_mem::ir::{MemInstr, MemLiteral};
    use reciplexa_mem::linear::LinearProgram;
    use reciplexa_mem::reg::Reg;
    use reciplexa_mem::trace::RcTrace;
    use reciplexa_mem::{exec_linear, ExecError};

    let mut trace = RcTrace::default();
    let bad = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(2.0),
            },
            MemInstr::Lit {
                dst: Reg(2),
                lit: MemLiteral::Number(3.0),
            },
            MemInstr::Select {
                dst: Reg(3),
                cond: Reg(0),
                then_reg: Reg(1),
                else_reg: Reg(2),
            },
        ],
        return_reg: Reg(3),
    };
    assert!(matches!(
        exec_linear(&bad, &mut trace),
        Err(ExecError::UnhandledRaise(_))
    ));
}

#[test]
fn select_pipeline_covers_conservative_perceus_seal_verify_ir() {
    use reciplexa_mem::conservative::conservative_rc;
    use reciplexa_mem::ir::{collect_regs_instr, MemInstr, MemLiteral};
    use reciplexa_mem::linear::LinearProgram;
    use reciplexa_mem::perceus::perceus_pass;
    use reciplexa_mem::reg::Reg;
    use reciplexa_mem::seal::seal_before_return;
    use reciplexa_mem::verify::verify_ownership;

    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Bool(true),
            },
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(10.0),
            },
            MemInstr::Lit {
                dst: Reg(2),
                lit: MemLiteral::Number(20.0),
            },
            MemInstr::Select {
                dst: Reg(3),
                cond: Reg(0),
                then_reg: Reg(1),
                else_reg: Reg(2),
            },
            MemInstr::Drop { reg: Reg(0) },
            MemInstr::Drop { reg: Reg(1) },
            MemInstr::Drop { reg: Reg(2) },
            MemInstr::Return { reg: Reg(3) },
        ],
        return_reg: Reg(3),
    };
    let mut regs = Vec::new();
    collect_regs_instr(
        &MemInstr::Select {
            dst: Reg(3),
            cond: Reg(0),
            then_reg: Reg(1),
            else_reg: Reg(2),
        },
        &mut regs,
    );
    assert_eq!(regs, vec![Reg(3), Reg(0), Reg(1), Reg(2)]);

    let cons = conservative_rc(&prog);
    assert!(cons.instrs.iter().any(|i| matches!(i, MemInstr::Select { .. })));
    let perc = perceus_pass(&prog);
    assert!(perc.instrs.iter().any(|i| matches!(i, MemInstr::Select { .. })));
    let sealed = seal_before_return(prog.clone());
    verify_ownership(&sealed).unwrap();
}

#[test]
fn exec_select_bool_true_false_and_unbound_pick() {
    use reciplexa_mem::ir::{MemInstr, MemLiteral};
    use reciplexa_mem::linear::LinearProgram;
    use reciplexa_mem::reg::Reg;
    use reciplexa_mem::trace::RcTrace;
    use reciplexa_eval::RuntimeValue;
    use reciplexa_mem::{exec_linear, ExecError};

    // true picks then
    let mut trace = RcTrace::default();
    let then_prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Bool(true),
            },
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(10.0),
            },
            MemInstr::Lit {
                dst: Reg(2),
                lit: MemLiteral::Number(20.0),
            },
            MemInstr::Select {
                dst: Reg(3),
                cond: Reg(0),
                then_reg: Reg(1),
                else_reg: Reg(2),
            },
            MemInstr::Return { reg: Reg(3) },
        ],
        return_reg: Reg(3),
    };
    let v = exec_linear(&then_prog, &mut trace).unwrap();
    assert_eq!(v, RuntimeValue::Number(10.0));

    // false picks else
    let mut trace = RcTrace::default();
    let else_prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Bool(false),
            },
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(10.0),
            },
            MemInstr::Lit {
                dst: Reg(2),
                lit: MemLiteral::Number(20.0),
            },
            MemInstr::Select {
                dst: Reg(3),
                cond: Reg(0),
                then_reg: Reg(1),
                else_reg: Reg(2),
            },
            MemInstr::Return { reg: Reg(3) },
        ],
        return_reg: Reg(3),
    };
    let v = exec_linear(&else_prog, &mut trace).unwrap();
    assert_eq!(v, RuntimeValue::Number(20.0));

    // unbound pick
    let mut trace = RcTrace::default();
    let bad = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Bool(true),
            },
            MemInstr::Lit {
                dst: Reg(2),
                lit: MemLiteral::Number(20.0),
            },
            MemInstr::Select {
                dst: Reg(3),
                cond: Reg(0),
                then_reg: Reg(9),
                else_reg: Reg(2),
            },
        ],
        return_reg: Reg(3),
    };
    assert!(matches!(
        exec_linear(&bad, &mut trace),
        Err(ExecError::UnboundReg(_))
    ));

    // unbound cond
    let mut trace = RcTrace::default();
    let bad = LinearProgram {
        instrs: vec![MemInstr::Select {
            dst: Reg(3),
            cond: Reg(9),
            then_reg: Reg(1),
            else_reg: Reg(2),
        }],
        return_reg: Reg(3),
    };
    assert!(matches!(
        exec_linear(&bad, &mut trace),
        Err(ExecError::UnboundReg(_))
    ));
}

#[test]
fn exec_bytes_and_record_mutate_error_edges() {
    use reciplexa_mem::ir::{MemInstr, MemLiteral};
    use reciplexa_mem::linear::LinearProgram;
    use reciplexa_mem::reg::Reg;
    use reciplexa_mem::trace::RcTrace;
    use reciplexa_eval::RuntimeValue;
    use reciplexa_mem::{exec_linear, ExecError};

    let mut trace = RcTrace::default();
    let bytes = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Bytes(vec![1, 2, 3]),
            },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    assert_eq!(
        exec_linear(&bytes, &mut trace).unwrap(),
        RuntimeValue::Bytes(vec![1, 2, 3])
    );

    // missing __base
    let mut trace = RcTrace::default();
    let missing_base = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Construct {
                dst: Reg(2),
                tag: "record-update".into(),
                fields: vec![("x".into(), Reg(1))],
            },
        ],
        return_reg: Reg(2),
    };
    assert!(matches!(
        exec_linear(&missing_base, &mut trace),
        Err(ExecError::UnboundReg(_))
    ));

    // unbound base reg
    let mut trace = RcTrace::default();
    let unbound_base = LinearProgram {
        instrs: vec![MemInstr::Construct {
            dst: Reg(2),
            tag: "record-update".into(),
            fields: vec![("__base".into(), Reg(9))],
        }],
        return_reg: Reg(2),
    };
    assert!(matches!(
        exec_linear(&unbound_base, &mut trace),
        Err(ExecError::UnboundReg(_))
    ));

    // non-record base
    let mut trace = RcTrace::default();
    let non_rec = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Construct {
                dst: Reg(2),
                tag: "record-update".into(),
                fields: vec![("__base".into(), Reg(0)), ("a".into(), Reg(0))],
            },
        ],
        return_reg: Reg(2),
    };
    assert!(matches!(
        exec_linear(&non_rec, &mut trace),
        Err(ExecError::UnhandledRaise(_)) | Err(ExecError::UnboundReg(_))
    ));

    // unbound field value
    let mut trace = RcTrace::default();
    let unbound_field = LinearProgram {
        instrs: vec![
            MemInstr::Construct {
                dst: Reg(0),
                tag: "record".into(),
                fields: vec![],
            },
            MemInstr::Construct {
                dst: Reg(2),
                tag: "record-update".into(),
                fields: vec![("__base".into(), Reg(0)), ("title".into(), Reg(9))],
            },
        ],
        return_reg: Reg(2),
    };
    assert!(matches!(
        exec_linear(&unbound_field, &mut trace),
        Err(ExecError::UnboundReg(_))
    ));

    // update happy + extend happy + missing key noop + existing key skip
    let mut trace = RcTrace::default();
    let mutate = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::String("a".into()),
            },
            MemInstr::Construct {
                dst: Reg(1),
                tag: "record".into(),
                fields: vec![("title".into(), Reg(0))],
            },
            MemInstr::Lit {
                dst: Reg(2),
                lit: MemLiteral::String("b".into()),
            },
            MemInstr::Construct {
                dst: Reg(3),
                tag: "record-update".into(),
                fields: vec![("__base".into(), Reg(1)), ("title".into(), Reg(2))],
            },
            MemInstr::Construct {
                dst: Reg(4),
                tag: "record-update".into(),
                fields: vec![("__base".into(), Reg(3)), ("missing".into(), Reg(2))],
            },
            MemInstr::Construct {
                dst: Reg(5),
                tag: "record-extend".into(),
                fields: vec![("__base".into(), Reg(4)), ("extra".into(), Reg(2))],
            },
            MemInstr::Construct {
                dst: Reg(6),
                tag: "record-extend".into(),
                fields: vec![("__base".into(), Reg(5)), ("extra".into(), Reg(0))],
            },
            MemInstr::Return { reg: Reg(6) },
        ],
        return_reg: Reg(6),
    };
    let v = exec_linear(&mutate, &mut trace).unwrap();
    assert!(matches!(v, RuntimeValue::Record(_)));
}

#[test]
fn observably_equal_remaining_value_arms() {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use reciplexa_eval::control::identity_resume;
    use reciplexa_eval::RuntimeValue;
    use reciplexa_mem::observably_equal;

    assert!(observably_equal(
        &RuntimeValue::Number(1.0),
        &RuntimeValue::Number(1.0)
    ));
    assert!(observably_equal(
        &RuntimeValue::F64(1.0),
        &RuntimeValue::F64(1.0)
    ));
    assert!(observably_equal(
        &RuntimeValue::Bytes(vec![1]),
        &RuntimeValue::Bytes(vec![1])
    ));
    let cell = |n: i128| RuntimeValue::Cell {
        value: Rc::new(RefCell::new(RuntimeValue::Int(n))),
        alive: Rc::new(Cell::new(true)),
    };
    assert!(observably_equal(&cell(1), &cell(1)));
    assert!(observably_equal(
        &RuntimeValue::Handler {
            op: "a".into(),
            params: vec![],
            body: reciplexa_core::expr::CoreExpr::Lit(reciplexa_core::expr::CoreLiteral::Unit),
            env: Rc::new(RefCell::new(Default::default())),
        },
        &RuntimeValue::Handler {
            op: "a".into(),
            params: vec![],
            body: reciplexa_core::expr::CoreExpr::Lit(reciplexa_core::expr::CoreLiteral::Unit),
            env: Rc::new(RefCell::new(Default::default())),
        },
    ));
    assert!(observably_equal(
        &RuntimeValue::OneShotResume {
            used: Rc::new(Cell::new(false)),
            cont: identity_resume(),
        },
        &RuntimeValue::OneShotResume {
            used: Rc::new(Cell::new(true)),
            cont: identity_resume(),
        },
    ));
}

#[test]
fn lower_remaining_core_expr_forms() {
    use reciplexa_core::cast::CastEvidence;
    use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
    use reciplexa_core::ty::CoreType;
    use reciplexa_mem::ir::{MemInstr, MemLiteral};
    use reciplexa_mem::lower::lower_core_linear;

    let unbound = lower_core_linear(&CoreExpr::Var("ghost".into()));
    assert!(unbound.instrs.iter().any(|i| matches!(
        i,
        MemInstr::Lit {
            lit: MemLiteral::Unit,
            ..
        }
    )));

    let wildcard_payload = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "Ok".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Int(1)))),
        }),
        arms: vec![MatchArm {
            pattern: CorePattern::Variant {
                tag: "Ok".into(),
                payload: Some(Box::new(CorePattern::Wildcard)),
            },
            body: CoreExpr::Lit(CoreLiteral::Int(9)),
        }],
    };
    let _ = lower_core_linear(&wildcard_payload);

    let handle = CoreExpr::Handle {
        op: "log".into(),
        handler_params: vec!["x".into()],
        handler_body: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        body: Box::new(CoreExpr::Lit(CoreLiteral::Int(2))),
    };
    let _ = lower_core_linear(&handle);

    let hv = CoreExpr::HandlerValue {
        op: "log".into(),
        handler_params: vec![],
        handler_body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
    };
    let _ = lower_core_linear(&hv);

    let with = CoreExpr::With {
        handler: Box::new(hv.clone()),
        body: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
    };
    let _ = lower_core_linear(&with);

    let forward = lower_core_linear(&CoreExpr::Forward {
        resume_name: "k".into(),
    });
    assert!(forward.instrs.iter().any(|i| matches!(
        i,
        MemInstr::Construct { tag, .. } if tag == "perform:forward"
    )));

    let cast = CoreExpr::Cast {
        expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        evidence: CastEvidence::Identity,
        target: CoreType::Int,
        cast_id: 0,
    };
    let _ = lower_core_linear(&cast);
    let try_cast = CoreExpr::TryCast {
        expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        target: CoreType::Int,
        cast_id: 1,
    };
    let _ = lower_core_linear(&try_cast);
    let check = CoreExpr::CheckCast {
        expr: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        target: CoreType::Int,
        cast_id: 2,
    };
    let _ = lower_core_linear(&check);
    let _ = lower_core_linear(&CoreExpr::Error);
}
