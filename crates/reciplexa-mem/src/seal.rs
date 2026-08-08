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
        MemInstr::Jump { .. }
        | MemInstr::Raise { .. }
        | MemInstr::Branch { .. }
        | MemInstr::Phi { .. }
        | MemInstr::RegisterCleanup { .. }
        | MemInstr::RunCleanup
        | MemInstr::Resume { .. }
        | MemInstr::DiscardCont { .. }
        | MemInstr::Return { .. } => {}
    }
}
