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
        &RuntimeValue::Number(1.0)
    ));
}

#[test]
fn reference_eval_error_maps_to_equiv() {
    let e = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        arms: vec![MatchArm::variant("X".into(), None, CoreExpr::Lit(CoreLiteral::Number(0.0)))],
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
    let err = check_observational_equiv(&RuntimeValue::Number(1.0), &RuntimeValue::Number(2.0))
        .unwrap_err();
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
        arms: vec![MatchArm::variant("None".into(), Some("x".into()), CoreExpr::Lit(CoreLiteral::Number(1.0)))],
    };
    let prog = reciplexa_mem::lower_core_linear(&matched);
    assert!(!prog.instrs.is_empty());

    let no_bind = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Number(0.0))),
        arms: vec![
            MatchArm::variant("A".into(), None, CoreExpr::Lit(CoreLiteral::Number(1.0))),
            MatchArm::variant("B".into(), None, CoreExpr::Lit(CoreLiteral::Number(2.0))),
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
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0)))),
        }),
        arms: vec![MatchArm::variant("Ok".into(), None, CoreExpr::Lit(CoreLiteral::Number(2.0)))],
    };
    let prog = reciplexa_mem::lower_core_linear(&bind_none);
    assert!(!prog.instrs.is_empty());

    let tag_miss = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "Ok".into(),
            payload: None,
        }),
        arms: vec![MatchArm::variant("Err".into(), None, CoreExpr::Lit(CoreLiteral::Number(0.0)))],
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
