//! Final sealing pass — drop stray registers before return.

use std::collections::HashSet;

use crate::ir::MemInstr;
use crate::linear::LinearProgram;
use crate::reg::Reg;

/// Drop any live registers except the return value immediately before `Return`.
pub fn seal_before_return(mut prog: LinearProgram) -> LinearProgram {
    let mut alive: HashSet<Reg> = HashSet::new();
    let mut out = Vec::new();

    for instr in &prog.instrs {
        if let MemInstr::Return { reg } = instr {
            for r in alive.clone() {
                if r != *reg {
                    out.push(MemInstr::Drop { reg: r });
                }
            }
            out.push(instr.clone());
            continue;
        }
        apply_track(&mut alive, instr);
        out.push(instr.clone());
    }

    prog.instrs = out;
    prog
}

fn apply_track(alive: &mut HashSet<Reg>, instr: &MemInstr) {
    match instr {
        MemInstr::Lit { dst, .. }
        | MemInstr::Dup { dst, .. }
        | MemInstr::Project { dst, .. }
        | MemInstr::MakeClosure { dst, .. }
        | MemInstr::Call { dst, .. } => {
            alive.insert(*dst);
        }
        MemInstr::Move { src, dst } => {
            alive.remove(src);
            alive.insert(*dst);
        }
        MemInstr::Construct { dst, fields, .. } => {
            for (_, r) in fields {
                alive.remove(r);
            }
            alive.insert(*dst);
        }
        MemInstr::ConstructReuse {
            dst, reuse, fields, ..
        } => {
            alive.remove(reuse);
            for (_, r) in fields {
                alive.remove(r);
            }
            alive.insert(*dst);
        }
        MemInstr::Drop { reg } => {
            alive.remove(reg);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::MemLiteral;

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
}
