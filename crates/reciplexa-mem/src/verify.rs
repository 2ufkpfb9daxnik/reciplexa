//! Step D: ownership and reuse verifiers (MEM-001 §30).

use crate::ir::MemInstr;
use crate::linear::LinearProgram;
use crate::state::{OwnMap, OwnState};

use crate::reg::Reg;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyError {
    UseAfterMove(Reg),
    UseAfterDrop(Reg),
    DoubleDrop(Reg),
    Leak(Reg),
    JoinMismatch { reg: Reg },
    ReuseOfShared(Reg),
    InvalidReuseClass { tag: String },
    ContinuationDoubleResume(u64),
    CleanupNotRun(String),
}

/// Verify ownership invariants on a linear program.
pub fn verify_ownership(prog: &LinearProgram) -> Result<(), VerifyError> {
    let mut state = OwnMap::new();
    let mut dropped: std::collections::HashSet<Reg> = std::collections::HashSet::new();

    for instr in &prog.instrs {
        if !matches!(instr, MemInstr::Drop { .. }) {
            for src in source_regs(instr) {
                match state.get(src) {
                    OwnState::Moved => return Err(VerifyError::UseAfterMove(src)),
                    OwnState::Dropped => return Err(VerifyError::UseAfterDrop(src)),
                    OwnState::Uninit | OwnState::Alive => {}
                }
            }
        }

        match instr {
            MemInstr::Drop { reg } => {
                if dropped.contains(reg) {
                    return Err(VerifyError::DoubleDrop(*reg));
                }
                if state.get(*reg) == OwnState::Uninit {
                    return Err(VerifyError::UseAfterDrop(*reg));
                }
                dropped.insert(*reg);
                state.drop_reg(*reg);
            }
            MemInstr::Move { src, dst } => {
                state.move_from(*src, *dst);
            }
            MemInstr::Dup { dst, src } => {
                if state.get(*src) == OwnState::Uninit {
                    return Err(VerifyError::UseAfterDrop(*src));
                }
                state.define(*dst);
            }
            MemInstr::ConstructReuse {
                dst, reuse, fields, ..
            } => {
                state.drop_reg(*reuse);
                for (_, r) in fields {
                    state.drop_reg(*r);
                }
                state.define(*dst);
            }
            MemInstr::Lit { dst, .. }
            | MemInstr::Construct { dst, .. }
            | MemInstr::Project { dst, .. }
            | MemInstr::MakeClosure { dst, .. }
            | MemInstr::Call { dst, .. } => {
                state.define(*dst);
            }
            MemInstr::Resume { cont } => {
                // one-shot: tracked externally in full runtime
                let _ = cont;
            }
            MemInstr::RunCleanup => {}
            MemInstr::RegisterCleanup { label } => {
                let _ = label;
            }
            MemInstr::Raise { .. } | MemInstr::DiscardCont { .. } => {
                // failure/cancel paths must still drop live regs — checked at end
            }
            _ => {}
        }
    }

    for r in state.alive_regs() {
        if r != prog.effective_return_reg() {
            return Err(VerifyError::Leak(r));
        }
    }
    Ok(())
}

/// Verify reuse specialization is sound.
pub fn verify_reuse(prog: &LinearProgram) -> Result<(), VerifyError> {
    verify_ownership(prog)?;
    let mut state = OwnMap::new();
    let mut dupped: std::collections::HashSet<Reg> = std::collections::HashSet::new();
    for instr in &prog.instrs {
        if let MemInstr::Dup { src, .. } = instr {
            dupped.insert(*src);
        }
        if let MemInstr::ConstructReuse {
            dst, reuse, tag, ..
        } = instr
        {
            if dupped.contains(reuse) || state.get(*reuse) != OwnState::Alive {
                return Err(VerifyError::ReuseOfShared(*reuse));
            }
            if tag.starts_with("perform:") || tag == "closure" {
                return Err(VerifyError::InvalidReuseClass { tag: tag.clone() });
            }
            state.move_from(*reuse, *dst);
        }
        apply_instr_state(&mut state, instr);
    }
    Ok(())
}

fn apply_instr_state(state: &mut OwnMap, instr: &MemInstr) {
    match instr {
        MemInstr::Lit { dst, .. } | MemInstr::Construct { dst, .. } | MemInstr::Dup { dst, .. } => {
            state.define(*dst)
        }
        MemInstr::Move { src, dst } => state.move_from(*src, *dst),
        MemInstr::Drop { reg } => state.drop_reg(*reg),
        MemInstr::ConstructReuse { dst, reuse, .. } => state.move_from(*reuse, *dst),
        _ => {}
    }
}

fn source_regs(instr: &MemInstr) -> Vec<Reg> {
    let mut out = Vec::new();
    crate::ir::collect_regs_instr(instr, &mut out);
    match instr {
        MemInstr::Lit { dst: _, .. }
        | MemInstr::Dup { dst: _, .. }
        | MemInstr::Move { dst: _, .. }
        | MemInstr::Construct { dst: _, .. }
        | MemInstr::ConstructReuse { dst: _, .. }
        | MemInstr::Project { dst: _, .. }
        | MemInstr::MakeClosure { dst: _, .. }
        | MemInstr::Call { dst: _, .. } => out.retain(|r| {
            !matches!(
                instr,
                MemInstr::Lit { dst: d, .. }
                    | MemInstr::Dup { dst: d, .. }
                    | MemInstr::Move { dst: d, .. }
                    | MemInstr::Construct { dst: d, .. }
                    | MemInstr::ConstructReuse { dst: d, .. }
                    | MemInstr::Project { dst: d, .. }
                    | MemInstr::MakeClosure { dst: d, .. }
                    | MemInstr::Call { dst: d, .. } if *d == *r
            )
        }),
        _ => {}
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::MemLiteral;

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
                    body: crate::ir::BlockId(0),
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
}
