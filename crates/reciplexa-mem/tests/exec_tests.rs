use reciplexa_eval::RuntimeValue;
use reciplexa_mem::exec::{exec_linear, ExecError};
use reciplexa_mem::ir::{MemInstr, MemLiteral};
use reciplexa_mem::linear::LinearProgram;
use reciplexa_mem::reg::Reg;
use reciplexa_mem::trace::RcTrace;

#[test]
fn exec_literal() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(42.0),
            },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    let mut trace = RcTrace::default();
    let v = exec_linear(&prog, &mut trace).unwrap();
    assert_eq!(v, RuntimeValue::Number(42.0));
}

#[test]
fn exec_lit_reg1() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(3.0),
            },
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    let mut trace = RcTrace::default();
    let v = exec_linear(&prog, &mut trace).unwrap();
    assert_eq!(v, RuntimeValue::Number(3.0));
}

#[test]
fn exec_construct_only() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(3.0),
            },
            MemInstr::Construct {
                dst: Reg(0),
                tag: "record".into(),
                fields: vec![("x".into(), Reg(1))],
            },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    let mut trace = RcTrace::default();
    let v = exec_linear(&prog, &mut trace).unwrap();
    assert!(matches!(v, RuntimeValue::Record(_)));
}

#[test]
fn exec_record_with_field_drop() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(3.0),
            },
            MemInstr::Construct {
                dst: Reg(0),
                tag: "record".into(),
                fields: vec![("x".into(), Reg(1))],
            },
            MemInstr::Drop { reg: Reg(1) },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    let mut trace = RcTrace::default();
    let v = exec_linear(&prog, &mut trace).unwrap();
    assert!(matches!(v, RuntimeValue::Record(_)));
}

#[test]
fn refcount_dup_drop() {
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
            MemInstr::Drop { reg: Reg(0) },
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    let mut trace = RcTrace::default();
    exec_linear(&prog, &mut trace).unwrap();
    assert_eq!(trace.dups.len(), 1);
    assert_eq!(trace.drops.len(), 1);
}

#[test]
fn exec_unit_literal() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Unit,
            },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    let mut trace = RcTrace::default();
    assert_eq!(exec_linear(&prog, &mut trace).unwrap(), RuntimeValue::Unit);
}

#[test]
fn exec_string_literal() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::String("hi".into()),
            },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    let mut trace = RcTrace::default();
    assert_eq!(
        exec_linear(&prog, &mut trace).unwrap(),
        RuntimeValue::String("hi".into())
    );
}

#[test]
fn exec_variant_construct() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(7.0),
            },
            MemInstr::Construct {
                dst: Reg(0),
                tag: "Some".into(),
                fields: vec![("payload".into(), Reg(1))],
            },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    let mut trace = RcTrace::default();
    let v = exec_linear(&prog, &mut trace).unwrap();
    assert!(matches!(v, RuntimeValue::Variant { .. }));
}

#[test]
fn exec_project_record_field() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(9.0),
            },
            MemInstr::Construct {
                dst: Reg(0),
                tag: "record".into(),
                fields: vec![("x".into(), Reg(1))],
            },
            MemInstr::Project {
                dst: Reg(2),
                src: Reg(0),
                field: "x".into(),
            },
            MemInstr::Return { reg: Reg(2) },
        ],
        return_reg: Reg(2),
    };
    let mut trace = RcTrace::default();
    assert_eq!(
        exec_linear(&prog, &mut trace).unwrap(),
        RuntimeValue::Number(9.0)
    );
}

#[test]
fn exec_raise_runs_cleanups() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::RegisterCleanup {
                label: "scope".into(),
            },
            MemInstr::Raise { tag: "fail".into() },
        ],
        return_reg: Reg(0),
    };
    let mut trace = RcTrace::default();
    let err = exec_linear(&prog, &mut trace).unwrap_err();
    assert!(matches!(err, ExecError::UnhandledRaise(_)));
    assert_eq!(trace.cleanups, vec!["scope"]);
}

#[test]
fn exec_unbound_reg_errors() {
    let prog = LinearProgram {
        instrs: vec![MemInstr::Return { reg: Reg(99) }],
        return_reg: Reg(99),
    };
    let mut trace = RcTrace::default();
    assert!(matches!(
        exec_linear(&prog, &mut trace),
        Err(ExecError::UnboundReg(_))
    ));
}

#[test]
fn exec_perform_tag_returns_unit() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::String("msg".into()),
            },
            MemInstr::Construct {
                dst: Reg(0),
                tag: "perform:log".into(),
                fields: vec![("arg".into(), Reg(1))],
            },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    let mut trace = RcTrace::default();
    assert_eq!(exec_linear(&prog, &mut trace).unwrap(), RuntimeValue::Unit);
}

#[test]
fn exec_make_closure_captures_env() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(9.0),
            },
            MemInstr::MakeClosure {
                dst: Reg(1),
                param: "x".into(),
                body: reciplexa_mem::ir::BlockId(0),
                captures: vec![Reg(0)],
            },
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    let mut trace = RcTrace::default();
    let v = exec_linear(&prog, &mut trace).unwrap();
    assert!(matches!(v, RuntimeValue::Closure { .. }));
}

#[test]
fn exec_construct_reuse_unique_records_reuse() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(3.0),
            },
            MemInstr::Construct {
                dst: Reg(0),
                tag: "record".into(),
                fields: vec![("x".into(), Reg(1))],
            },
            MemInstr::ConstructReuse {
                dst: Reg(2),
                reuse: Reg(0),
                tag: "record".into(),
                fields: vec![("y".into(), Reg(1))],
            },
            MemInstr::Return { reg: Reg(2) },
        ],
        return_reg: Reg(2),
    };
    let mut trace = RcTrace::default();
    exec_linear(&prog, &mut trace).unwrap();
    assert!(!trace.reuses.is_empty());
}

#[test]
fn exec_construct_reuse_shared_falls_back() {
    let prog = LinearProgram {
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
            MemInstr::Dup {
                dst: Reg(2),
                src: Reg(0),
            },
            MemInstr::ConstructReuse {
                dst: Reg(3),
                reuse: Reg(0),
                tag: "record".into(),
                fields: vec![("y".into(), Reg(1))],
            },
            MemInstr::Return { reg: Reg(3) },
        ],
        return_reg: Reg(3),
    };
    let mut trace = RcTrace::default();
    exec_linear(&prog, &mut trace).unwrap();
    assert!(trace.reuses.is_empty());
}

#[test]
fn exec_move_transfers_ownership() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(5.0),
            },
            MemInstr::Move {
                src: Reg(0),
                dst: Reg(1),
            },
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    let mut trace = RcTrace::default();
    assert_eq!(
        exec_linear(&prog, &mut trace).unwrap(),
        RuntimeValue::Number(5.0)
    );
}

#[test]
fn exec_run_cleanup_drains_labels() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::RegisterCleanup { label: "a".into() },
            MemInstr::RegisterCleanup { label: "b".into() },
            MemInstr::RunCleanup,
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
fn exec_double_resume_errors() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Resume { cont: 1 },
            MemInstr::Resume { cont: 1 },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    let mut trace = RcTrace::default();
    assert!(matches!(
        exec_linear(&prog, &mut trace),
        Err(ExecError::DoubleDrop(_))
    ));
}

#[test]
fn exec_project_variant_payload() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(8.0),
            },
            MemInstr::Construct {
                dst: Reg(0),
                tag: "Some".into(),
                fields: vec![("payload".into(), Reg(1))],
            },
            MemInstr::Project {
                dst: Reg(2),
                src: Reg(0),
                field: "payload".into(),
            },
            MemInstr::Return { reg: Reg(2) },
        ],
        return_reg: Reg(2),
    };
    let mut trace = RcTrace::default();
    assert_eq!(
        exec_linear(&prog, &mut trace).unwrap(),
        RuntimeValue::Number(8.0)
    );
}

#[test]
fn exec_project_missing_field_errors() {
    let prog = LinearProgram {
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
            MemInstr::Project {
                dst: Reg(2),
                src: Reg(0),
                field: "missing".into(),
            },
            MemInstr::Return { reg: Reg(2) },
        ],
        return_reg: Reg(2),
    };
    let mut trace = RcTrace::default();
    assert!(exec_linear(&prog, &mut trace).is_err());
}
