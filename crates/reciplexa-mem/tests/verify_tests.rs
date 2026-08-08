use reciplexa_mem::ir::{MemInstr, MemLiteral};
use reciplexa_mem::linear::LinearProgram;
use reciplexa_mem::reg::Reg;
use reciplexa_mem::verify::{verify_ownership, verify_reuse, VerifyError};

#[test]
fn detects_use_after_drop() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Drop { reg: Reg(0) },
            MemInstr::Dup {
                dst: Reg(1),
                src: Reg(0),
            },
        ],
        return_reg: Reg(1),
    };
    assert!(matches!(
        verify_ownership(&prog),
        Err(VerifyError::UseAfterDrop(_))
    ));
}

#[test]
fn detects_double_drop() {
    let prog = LinearProgram {
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
        verify_ownership(&prog),
        Err(VerifyError::DoubleDrop(_))
    ));
}

#[test]
fn detects_leak() {
    let prog = LinearProgram {
        instrs: vec![MemInstr::Lit {
            dst: Reg(0),
            lit: MemLiteral::Number(1.0),
        }],
        return_reg: Reg(1),
    };
    assert!(matches!(verify_ownership(&prog), Err(VerifyError::Leak(_))));
}

#[test]
fn accepts_valid_program() {
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
    assert!(verify_ownership(&prog).is_ok());
}

#[test]
fn detects_use_after_move() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Move {
                src: Reg(0),
                dst: Reg(1),
            },
            MemInstr::Dup {
                dst: Reg(2),
                src: Reg(0),
            },
        ],
        return_reg: Reg(1),
    };
    assert!(matches!(
        verify_ownership(&prog),
        Err(VerifyError::UseAfterMove(_))
    ));
}

#[test]
fn verify_reuse_rejects_shared() {
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
            MemInstr::ConstructReuse {
                dst: Reg(2),
                reuse: Reg(0),
                tag: "record".into(),
                fields: vec![],
            },
            MemInstr::Drop { reg: Reg(1) },
            MemInstr::Return { reg: Reg(2) },
        ],
        return_reg: Reg(2),
    };
    assert!(matches!(
        verify_reuse(&prog),
        Err(VerifyError::ReuseOfShared(_))
    ));
}

#[test]
fn verify_reuse_rejects_perform_tag() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::ConstructReuse {
                dst: Reg(1),
                reuse: Reg(0),
                tag: "perform:log".into(),
                fields: vec![],
            },
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    assert!(matches!(
        verify_reuse(&prog),
        Err(VerifyError::InvalidReuseClass { .. })
    ));
}

#[test]
fn verify_accepts_make_closure_and_call() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::MakeClosure {
                dst: Reg(1),
                param: "x".into(),
                body: reciplexa_mem::ir::BlockId(0),
                captures: vec![],
            },
            MemInstr::Drop { reg: Reg(0) },
            MemInstr::Lit {
                dst: Reg(2),
                lit: MemLiteral::Number(2.0),
            },
            MemInstr::Call {
                dst: Reg(3),
                closure: Reg(1),
                arg: Reg(2),
            },
            MemInstr::Drop { reg: Reg(1) },
            MemInstr::Drop { reg: Reg(2) },
            MemInstr::Return { reg: Reg(3) },
        ],
        return_reg: Reg(3),
    };
    assert!(verify_ownership(&prog).is_ok());
}

#[test]
fn verify_construct_reuse_drops_field_regs() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(2.0),
            },
            MemInstr::ConstructReuse {
                dst: Reg(2),
                reuse: Reg(0),
                tag: "record".into(),
                fields: vec![("f".into(), Reg(1))],
            },
            MemInstr::Return { reg: Reg(2) },
        ],
        return_reg: Reg(2),
    };
    assert!(verify_ownership(&prog).is_ok());
}

#[test]
fn verify_reuse_rejects_closure_tag() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::ConstructReuse {
                dst: Reg(1),
                reuse: Reg(0),
                tag: "closure".into(),
                fields: vec![],
            },
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    assert!(matches!(
        verify_reuse(&prog),
        Err(VerifyError::InvalidReuseClass { .. })
    ));
}

#[test]
fn verify_raise_and_discard_paths_do_not_leak() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::RegisterCleanup {
                label: "scope".into(),
            },
            MemInstr::Raise { tag: "fail".into() },
        ],
        return_reg: Reg(0),
    };
    assert!(verify_ownership(&prog).is_ok());

    let prog2 = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::DiscardCont { cont: 0 },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    assert!(verify_ownership(&prog2).is_ok());
}

#[test]
fn verify_reuse_accepts_valid_record_reuse() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
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
    assert!(verify_reuse(&prog).is_ok());
}

#[test]
fn verify_drop_uninit_is_use_after_drop() {
    let prog = LinearProgram {
        instrs: vec![MemInstr::Drop { reg: Reg(0) }],
        return_reg: Reg(0),
    };
    assert!(matches!(
        verify_ownership(&prog),
        Err(VerifyError::UseAfterDrop(_))
    ));
}

#[test]
fn verify_resume_and_cleanup_do_not_panic() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Resume { cont: 1 },
            MemInstr::RunCleanup,
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(0),
    };
    assert!(verify_ownership(&prog).is_ok());
}
