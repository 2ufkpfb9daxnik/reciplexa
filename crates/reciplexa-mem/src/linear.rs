//! Flat instruction-list programs (Step A primary representation).

use crate::ir::MemInstr;
use crate::reg::Reg;

/// Linear ownership program — one instruction sequence with explicit control ops.
#[derive(Debug, Clone, PartialEq)]
pub struct LinearProgram {
    pub instrs: Vec<MemInstr>,
    pub return_reg: Reg,
}

impl LinearProgram {
    pub fn regs(&self) -> Vec<Reg> {
        let mut out = Vec::new();
        for i in &self.instrs {
            crate::ir::collect_regs_instr(i, &mut out);
        }
        out.sort_by_key(|r| r.0);
        out.dedup();
        out
    }

    pub fn effective_return_reg(&self) -> Reg {
        for i in &self.instrs {
            if let MemInstr::Return { reg } = i {
                return *reg;
            }
        }
        self.return_reg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::MemLiteral;

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
}
