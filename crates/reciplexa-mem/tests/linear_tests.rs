use reciplexa_mem::ir::MemInstr;
use reciplexa_mem::linear::LinearProgram;
use reciplexa_mem::reg::Reg;

use reciplexa_mem::ir::MemLiteral;

#[test]
fn regs_dedupes_and_sorts() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(3),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Dup {
                dst: Reg(1),
                src: Reg(3),
            },
            MemInstr::Return { reg: Reg(1) },
        ],
        return_reg: Reg(1),
    };
    assert_eq!(prog.regs(), vec![Reg(1), Reg(3)]);
}

#[test]
fn effective_return_from_return_instr() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Return { reg: Reg(0) },
        ],
        return_reg: Reg(99),
    };
    assert_eq!(prog.effective_return_reg(), Reg(0));
}

#[test]
fn effective_return_falls_back() {
    let prog = LinearProgram {
        instrs: vec![MemInstr::Lit {
            dst: Reg(0),
            lit: MemLiteral::Number(1.0),
        }],
        return_reg: Reg(5),
    };
    assert_eq!(prog.effective_return_reg(), Reg(5));
}
