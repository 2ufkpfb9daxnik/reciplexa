use reciplexa_mem::ir::{MemInstr, MemLiteral};
use reciplexa_mem::linear::LinearProgram;
use reciplexa_mem::reg::Reg;
use reciplexa_mem::seal_before_return;

#[test]
fn drops_stray_regs_before_return() {
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
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    let sealed = seal_before_return(prog);
    assert!(sealed
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Drop { reg: Reg(0) })));
}

#[test]
fn seals_closure_call_and_reuse_paths() {
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
            MemInstr::MakeClosure {
                dst: Reg(2),
                param: "x".into(),
                body: reciplexa_mem::ir::BlockId(0),
                captures: vec![Reg(0)],
            },
            MemInstr::Call {
                dst: Reg(3),
                closure: Reg(2),
                arg: Reg(1),
            },
            MemInstr::ConstructReuse {
                dst: Reg(4),
                reuse: Reg(0),
                tag: "record".into(),
                fields: vec![("f".into(), Reg(3))],
            },
            MemInstr::Return { reg: Reg(4) },
        ],
        return_reg: Reg(4),
    };
    let sealed = seal_before_return(prog);
    assert!(sealed
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Drop { .. })));
}
