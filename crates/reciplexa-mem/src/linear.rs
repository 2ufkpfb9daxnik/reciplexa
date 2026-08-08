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
