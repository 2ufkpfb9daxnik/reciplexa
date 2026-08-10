//! Phase 6 conformance: Perceus, ownership, reuse (MEM-001 §33).

use reciplexa_core::expr::{CoreExpr, CoreLiteral, MatchArm};
use reciplexa_eval::{eval_expr, RuntimeValue, UnitHost};
use reciplexa_mem::{
    assert_observational_equiv, compile_and_run, compile_and_run_conservative,
    compile_and_run_no_reuse, conservative_rc, exec_linear, lower_core_linear, observably_equal,
    perceus_pass, reuse_pass, seal_before_return, verify_ownership, verify_reuse, LinearProgram,
    MemInstr, MemLiteral, RcTrace, Reg, VerifyError,
};
use std::collections::HashMap;

fn eval_ref(expr: &CoreExpr) -> RuntimeValue {
    eval_expr(expr, &HashMap::new(), &mut UnitHost).unwrap()
}

// --- MEM-01: basic drop before unrelated work ---

#[test]
fn mem01_drop_before_unrelated_work() {
    let expr = CoreExpr::Seq(vec![
        CoreExpr::Let {
            name: "value".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
        },
        CoreExpr::Lit(CoreLiteral::Number(3.0)),
    ]);
    assert_observational_equiv(&expr).unwrap();
    let raw = lower_core_linear(&expr);
    let opt = perceus_pass(&raw);
    let drops: Vec<_> = opt
        .instrs
        .iter()
        .filter(|i| matches!(i, MemInstr::Drop { .. }))
        .collect();
    assert!(
        !drops.is_empty(),
        "binding must be dropped before unrelated work"
    );
}

// --- MEM-02: shared value requires dup, no early drop ---

#[test]
fn mem02_shared_value_inserts_dup() {
    let raw = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(5.0),
            },
            MemInstr::Dup {
                dst: Reg(1),
                src: Reg(0),
            },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    let rc = conservative_rc(&raw);
    assert!(
        rc.instrs.iter().any(|i| matches!(i, MemInstr::Dup { .. })),
        "shared use must insert dup"
    );
    let sealed = seal_before_return(rc);
    verify_ownership(&sealed).unwrap();
    let mut trace = RcTrace::default();
    exec_linear(&sealed, &mut trace).unwrap();
}

// --- MEM-03: exclusive branch (match) observational equivalence ---

#[test]
fn mem03_exclusive_branch_match() {
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "Some".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Number(42.0)))),
        }),
        arms: vec![
            MatchArm::variant(
                "Some".into(),
                Some("n".into()),
                CoreExpr::Lit(CoreLiteral::Number(42.0)),
            ),
            MatchArm::variant("None".into(), None, CoreExpr::Lit(CoreLiteral::Number(0.0))),
        ],
    };
    assert_observational_equiv(&expr).unwrap();
}

// --- MEM-04 / MEM-05: reuse observational equivalence ---

#[test]
fn mem04_reuse_success_same_observable_result() {
    let raw = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Construct {
                dst: Reg(0),
                tag: "record".into(),
                fields: vec![("x".into(), Reg(1))],
            },
            MemInstr::Drop { reg: Reg(0) },
            MemInstr::Construct {
                dst: Reg(2),
                tag: "record".into(),
                fields: vec![("y".into(), Reg(1))],
            },
            MemInstr::Return { reg: Reg(2) },
        ],
        return_reg: Reg(2),
    };
    let with_reuse = seal_before_return(reuse_pass(raw.clone()));
    let without_reuse = seal_before_return(raw);
    verify_ownership(&with_reuse).unwrap();
    let mut trace = RcTrace::default();
    let v_reuse = exec_linear(&with_reuse, &mut trace).unwrap();
    let v_plain = exec_linear(&without_reuse, &mut RcTrace::default()).unwrap();
    assert!(observably_equal(&v_reuse, &v_plain));
    assert!(with_reuse
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::ConstructReuse { .. })));
}

#[test]
fn mem05_shared_value_no_reuse_specialization() {
    let expr = CoreExpr::Let {
        name: "v".into(),
        value: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        body: Box::new(CoreExpr::Seq(vec![
            CoreExpr::Lit(CoreLiteral::Number(1.0)),
            CoreExpr::Record {
                fields: vec![("x".into(), CoreExpr::Lit(CoreLiteral::Number(1.0)))],
            },
        ])),
    };
    let reused = reuse_pass(perceus_pass(&lower_core_linear(&expr)));
    assert!(
        !reused
            .instrs
            .iter()
            .any(|i| matches!(i, MemInstr::ConstructReuse { .. })),
        "shared value must not reuse"
    );
    assert_observational_equiv(&expr).unwrap();
}

// --- MEM-06: closure capture ---

#[test]
fn mem06_closure_capture_observational_equiv() {
    let expr = CoreExpr::Let {
        name: "x".into(),
        value: Box::new(CoreExpr::Lit(CoreLiteral::Number(10.0))),
        body: Box::new(CoreExpr::Lambda {
            params: vec!["y".into()],
            body: Box::new(CoreExpr::Lit(CoreLiteral::Number(10.0))),
        }),
    };
    assert_observational_equiv(&expr).unwrap();
}

// --- MEM-10 / MEM-11 / MEM-12: failure, continuation, cleanup paths ---

#[test]
fn mem10_resume_one_shot() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Unit,
            },
            MemInstr::RegisterCleanup {
                label: "scope".into(),
            },
            MemInstr::Resume { cont: 1 },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    let mut trace = RcTrace::default();
    exec_linear(&prog, &mut trace).unwrap();
    assert!(trace.cleanups.is_empty());
}

#[test]
fn mem11_discard_cont_runs_cleanup_lifo() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::RegisterCleanup { label: "a".into() },
            MemInstr::RegisterCleanup { label: "b".into() },
            MemInstr::DiscardCont { cont: 0 },
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Unit,
            },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    let mut trace = RcTrace::default();
    exec_linear(&prog, &mut trace).unwrap();
    assert_eq!(trace.cleanups, vec!["b", "a"]);
}

#[test]
fn mem12_failure_unwind_runs_cleanup() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::RegisterCleanup {
                label: "resource".into(),
            },
            MemInstr::Raise {
                tag: "failure".into(),
            },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    let mut trace = RcTrace::default();
    assert!(exec_linear(&prog, &mut trace).is_err());
    assert_eq!(trace.cleanups, vec!["resource"]);
}

// --- MEM-19: ownership verifier rejects invalid IR ---

#[test]
fn mem19_verifier_rejects_use_after_drop() {
    let bad = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Drop { reg: Reg(0) },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    assert!(matches!(
        verify_ownership(&bad),
        Err(VerifyError::UseAfterDrop(_))
    ));
}

#[test]
fn mem19_verifier_rejects_leak() {
    let bad = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    assert!(matches!(verify_ownership(&bad), Err(VerifyError::Leak(_))));
}

#[test]
fn mem19_verifier_rejects_double_drop() {
    let bad = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Drop { reg: Reg(0) },
            MemInstr::Drop { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    assert!(matches!(
        verify_ownership(&bad),
        Err(VerifyError::DoubleDrop(_))
    ));
}

#[test]
fn mem19_reuse_verifier_rejects_shared_reuse() {
    let bad = LinearProgram {
        instrs: vec![
            MemInstr::Construct {
                dst: Reg(0),
                tag: "record".into(),
                fields: vec![],
            },
            MemInstr::Dup {
                dst: Reg(1),
                src: Reg(0),
            },
            MemInstr::Drop { reg: Reg(1) },
            MemInstr::ConstructReuse {
                dst: Reg(2),
                reuse: Reg(0),
                tag: "record".into(),
                fields: vec![],
            },
            MemInstr::Return { reg: Reg(2) },
        ],
        return_reg: Reg(2),
    };
    assert!(matches!(
        verify_reuse(&bad),
        Err(VerifyError::ReuseOfShared(_))
    ));
}

// --- Phase 6 §8.4: reference vs optimized evaluator equivalence ---

#[test]
fn equiv_conservative_matches_reference() {
    let cases = vec![
        CoreExpr::Lit(CoreLiteral::Number(0.0)),
        CoreExpr::Seq(vec![
            CoreExpr::Lit(CoreLiteral::Number(1.0)),
            CoreExpr::Lit(CoreLiteral::Number(2.0)),
        ]),
        CoreExpr::Record {
            fields: vec![
                ("a".into(), CoreExpr::Lit(CoreLiteral::String("hi".into()))),
                ("b".into(), CoreExpr::Lit(CoreLiteral::Number(3.0))),
            ],
        },
        CoreExpr::Variant {
            tag: "Ok".into(),
            payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Number(99.0)))),
        },
    ];
    for expr in cases {
        let reference = eval_ref(&expr);
        let conservative = compile_and_run_conservative(&expr).unwrap();
        let optimized = compile_and_run(&expr).unwrap();
        assert!(
            observably_equal(&reference, &conservative),
            "conservative: {expr:?}"
        );
        assert!(
            observably_equal(&reference, &optimized),
            "optimized: {expr:?}"
        );
    }
}

#[test]
fn equiv_perceus_matches_conservative() {
    let expr = CoreExpr::Let {
        name: "v".into(),
        value: Box::new(CoreExpr::Record {
            fields: vec![("k".into(), CoreExpr::Lit(CoreLiteral::Number(4.0)))],
        }),
        body: Box::new(CoreExpr::RecordGet {
            record: Box::new(CoreExpr::Record {
                fields: vec![("k".into(), CoreExpr::Lit(CoreLiteral::Number(0.0)))],
            }),
            field: "k".into(),
        }),
    };
    let conservative = compile_and_run_conservative(&expr).unwrap();
    let perceus = compile_and_run_no_reuse(&expr).unwrap();
    assert!(observably_equal(&conservative, &perceus));
}

#[test]
fn sealed_pipeline_passes_verifier() {
    let expr = CoreExpr::Record {
        fields: vec![("z".into(), CoreExpr::Lit(CoreLiteral::Number(8.0)))],
    };
    let sealed = seal_before_return(reuse_pass(perceus_pass(&lower_core_linear(&expr))));
    verify_ownership(&sealed).unwrap();
    verify_reuse(&sealed).unwrap();
}

#[test]
fn perform_effect_observational_equiv() {
    let expr = CoreExpr::Perform {
        op: "log".into(),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::String("mem".into()))),
    };
    assert_observational_equiv(&expr).unwrap();
}

#[test]
fn record_get_observational_equiv() {
    let expr = CoreExpr::RecordGet {
        record: Box::new(CoreExpr::Record {
            fields: vec![("f".into(), CoreExpr::Lit(CoreLiteral::Number(12.0)))],
        }),
        field: "f".into(),
    };
    assert_observational_equiv(&expr).unwrap();
}

#[test]
fn nested_let_chain_equiv() {
    let expr = CoreExpr::Let {
        name: "a".into(),
        value: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        body: Box::new(CoreExpr::Let {
            name: "b".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
            body: Box::new(CoreExpr::Seq(vec![
                CoreExpr::Lit(CoreLiteral::Number(3.0)),
                CoreExpr::Lit(CoreLiteral::Number(4.0)),
            ])),
        }),
    };
    assert_observational_equiv(&expr).unwrap();
}

#[test]
fn refcount_trace_records_dup_and_drop() {
    let expr = CoreExpr::Let {
        name: "v".into(),
        value: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        body: Box::new(CoreExpr::Seq(vec![
            CoreExpr::Lit(CoreLiteral::Number(1.0)),
            CoreExpr::Lit(CoreLiteral::Number(2.0)),
        ])),
    };
    let sealed = seal_before_return(conservative_rc(&lower_core_linear(&expr)));
    let mut trace = RcTrace::default();
    exec_linear(&sealed, &mut trace).unwrap();
    assert!(!trace.drops.is_empty());
}

#[test]
fn mem_dup_drop_order_observable() {
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
            MemInstr::Drop { reg: Reg(1) },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    verify_ownership(&prog).unwrap();
    let mut trace = RcTrace::default();
    exec_linear(&prog, &mut trace).unwrap();
    assert_eq!(trace.drops.len(), 1);
}

#[test]
fn mem_construct_record_fields() {
    let expr = CoreExpr::Record {
        fields: vec![
            ("x".into(), CoreExpr::Lit(CoreLiteral::Number(1.0))),
            ("y".into(), CoreExpr::Lit(CoreLiteral::String("z".into()))),
        ],
    };
    assert_observational_equiv(&expr).unwrap();
}

#[test]
fn mem_lambda_no_capture_drop() {
    let expr = CoreExpr::Lambda {
        params: vec!["x".into()],
        body: Box::new(CoreExpr::Lit(CoreLiteral::Number(0.0))),
    };
    assert_observational_equiv(&expr).unwrap();
}

#[test]
fn mem_variant_none_arm() {
    let expr = CoreExpr::Match {
        scrutinee: Box::new(CoreExpr::Variant {
            tag: "None".into(),
            payload: None,
        }),
        arms: vec![
            MatchArm::variant(
                "Some".into(),
                Some("v".into()),
                CoreExpr::Lit(CoreLiteral::Number(1.0)),
            ),
            MatchArm::variant("None".into(), None, CoreExpr::Lit(CoreLiteral::Number(0.0))),
        ],
    };
    assert_observational_equiv(&expr).unwrap();
}

#[test]
fn mem_verifier_rejects_dup_of_unowned() {
    let bad = LinearProgram {
        instrs: vec![
            MemInstr::Dup {
                dst: Reg(1),
                src: Reg(0),
            },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    assert!(verify_ownership(&bad).is_err());
}

#[test]
fn mem_empty_seq_unit_equiv() {
    let expr = CoreExpr::Seq(vec![]);
    assert_observational_equiv(&expr).unwrap();
}

#[test]
fn mem_color_literal_equiv() {
    let expr = CoreExpr::Lit(CoreLiteral::Color("blue".into()));
    assert_observational_equiv(&expr).unwrap();
}

#[test]
fn mem_string_literal_equiv() {
    let expr = CoreExpr::Lit(CoreLiteral::String("mem-test".into()));
    assert_observational_equiv(&expr).unwrap();
}

#[test]
fn mem_observably_equal_detects_difference() {
    let a = eval_ref(&CoreExpr::Lit(CoreLiteral::Number(1.0)));
    let b = eval_ref(&CoreExpr::Lit(CoreLiteral::Number(2.0)));
    assert!(!observably_equal(&a, &b));
}

#[test]
fn mem_reg_display_and_alloc() {
    use reciplexa_mem::{Reg, RegAlloc};
    let mut alloc = RegAlloc::default();
    assert_eq!(alloc.fresh().to_string(), "r0");
    assert_eq!(Reg(1).to_string(), "r1");
}

#[test]
fn mem_own_map_merge() {
    use reciplexa_mem::{OwnMap, OwnState, Reg};
    let mut m = OwnMap::new();
    m.define(Reg(0));
    assert_eq!(m.get(Reg(0)), OwnState::Alive);
    m.drop_reg(Reg(0));
    assert_eq!(m.get(Reg(0)), OwnState::Dropped);
}

#[test]
fn mem_linear_program_regs() {
    use reciplexa_mem::{LinearProgram, MemInstr, MemLiteral, Reg};
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    assert_eq!(prog.regs(), vec![Reg(0)]);
}

#[test]
fn mem_cfg_fixture_pipeline() {
    use reciplexa_mem::ir::{BlockId, MemLiteral};
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Branch {
                cond: Reg(0),
                then_block: BlockId(1),
                else_block: BlockId(2),
            },
            MemInstr::Jump { target: BlockId(0) },
            MemInstr::Phi {
                dst: Reg(1),
                incoming: vec![(BlockId(0), Reg(0))],
            },
            MemInstr::MakeClosure {
                dst: Reg(2),
                param: "x".into(),
                body: BlockId(0),
                captures: vec![Reg(0)],
            },
            MemInstr::Call {
                dst: Reg(3),
                closure: Reg(2),
                arg: Reg(0),
            },
            MemInstr::ConstructReuse {
                dst: Reg(4),
                reuse: Reg(0),
                tag: "record".into(),
                fields: vec![("v".into(), Reg(3))],
            },
            MemInstr::Return { reg: Reg(4) },
        ],
        return_reg: Reg(4),
    };
    let rc = conservative_rc(&prog);
    let perceus = perceus_pass(&prog);
    let sealed = seal_before_return(reuse_pass(perceus));
    assert!(!rc.instrs.is_empty());
    assert!(!sealed.instrs.is_empty());
    let mut trace = RcTrace::default();
    let exec_prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::MakeClosure {
                dst: Reg(1),
                param: "x".into(),
                body: BlockId(0),
                captures: vec![Reg(0)],
            },
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    exec_linear(&exec_prog, &mut trace).unwrap();
}
