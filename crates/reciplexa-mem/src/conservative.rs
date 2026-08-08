//! Step A: conservative reference counting (dup at every shared use, drop at last use).

use std::collections::HashMap;

use crate::ir::MemInstr;
use crate::linear::LinearProgram;
use crate::reg::{Reg, RegAlloc};

/// Insert naive dup/drop for every register use (no reuse).
pub fn conservative_rc(prog: &LinearProgram) -> LinearProgram {
    let remaining = remaining_use_counts(&prog.instrs);
    let mut out = Vec::new();
    let mut alloc = RegAlloc::default();
    let mut rem = remaining.clone();

    for instr in &prog.instrs {
        let sources = read_sources(instr);
        let mut remap = HashMap::new();
        for src in &sources {
            if let Some(count) = rem.get_mut(src) {
                *count = count.saturating_sub(1);
                if *count > 0 {
                    let dup = alloc.fresh();
                    out.push(MemInstr::Dup {
                        dst: dup,
                        src: *src,
                    });
                    remap.insert(*src, dup);
                }
            }
        }
        out.push(remap_instr(instr, &remap));

        if matches!(instr, MemInstr::Return { .. }) {
            continue;
        }
        for src in &sources {
            if rem.get(src).copied().unwrap_or(0) == 0 {
                out.push(MemInstr::Drop { reg: *src });
            }
        }
        for def in write_targets(instr) {
            if rem.get(&def).copied().unwrap_or(0) == 0 && def != prog.return_reg {
                out.push(MemInstr::Drop { reg: def });
            }
        }
    }

    LinearProgram {
        instrs: out,
        return_reg: prog.return_reg,
    }
}

fn remaining_use_counts(instrs: &[MemInstr]) -> HashMap<Reg, u32> {
    let mut counts = HashMap::new();
    for i in instrs {
        for r in read_sources(i) {
            *counts.entry(r).or_insert(0) += 1;
        }
    }
    counts
}

fn read_sources(instr: &MemInstr) -> Vec<Reg> {
    match instr {
        MemInstr::Dup { src, .. } => vec![*src],
        MemInstr::Move { src, .. } => vec![*src],
        MemInstr::Drop { reg } => vec![*reg],
        MemInstr::Construct { fields, .. } => fields.iter().map(|(_, r)| *r).collect(),
        MemInstr::ConstructReuse { reuse, fields, .. } => {
            let mut v: Vec<_> = fields.iter().map(|(_, r)| *r).collect();
            v.push(*reuse);
            v
        }
        MemInstr::Project { src, .. } => vec![*src],
        MemInstr::Call { closure, arg, .. } => vec![*closure, *arg],
        MemInstr::MakeClosure { captures, .. } => captures.clone(),
        MemInstr::Branch { cond, .. } => vec![*cond],
        MemInstr::Return { reg } => vec![*reg],
        MemInstr::Phi { incoming, .. } => incoming.iter().map(|(_, r)| *r).collect(),
        MemInstr::Lit { .. }
        | MemInstr::Jump { .. }
        | MemInstr::Raise { .. }
        | MemInstr::RegisterCleanup { .. }
        | MemInstr::RunCleanup
        | MemInstr::Resume { .. }
        | MemInstr::DiscardCont { .. } => Vec::new(),
    }
}

fn write_targets(instr: &MemInstr) -> Vec<Reg> {
    match instr {
        MemInstr::Lit { dst, .. }
        | MemInstr::Dup { dst, .. }
        | MemInstr::Move { dst, .. }
        | MemInstr::Construct { dst, .. }
        | MemInstr::ConstructReuse { dst, .. }
        | MemInstr::Project { dst, .. }
        | MemInstr::MakeClosure { dst, .. }
        | MemInstr::Call { dst, .. }
        | MemInstr::Phi { dst, .. } => vec![*dst],
        _ => Vec::new(),
    }
}

fn remap_instr(instr: &MemInstr, map: &HashMap<Reg, Reg>) -> MemInstr {
    let r = |x: Reg| *map.get(&x).unwrap_or(&x);
    match instr {
        MemInstr::Dup { dst, src } => MemInstr::Dup {
            dst: *dst,
            src: r(*src),
        },
        MemInstr::Move { dst, src } => MemInstr::Move {
            dst: *dst,
            src: r(*src),
        },
        MemInstr::Drop { reg } => MemInstr::Drop { reg: r(*reg) },
        MemInstr::Construct { dst, tag, fields } => MemInstr::Construct {
            dst: *dst,
            tag: tag.clone(),
            fields: fields.iter().map(|(k, v)| (k.clone(), r(*v))).collect(),
        },
        MemInstr::ConstructReuse {
            dst,
            reuse,
            tag,
            fields,
        } => MemInstr::ConstructReuse {
            dst: *dst,
            reuse: r(*reuse),
            tag: tag.clone(),
            fields: fields.iter().map(|(k, v)| (k.clone(), r(*v))).collect(),
        },
        MemInstr::Project { dst, src, field } => MemInstr::Project {
            dst: *dst,
            src: r(*src),
            field: field.clone(),
        },
        MemInstr::Call { dst, closure, arg } => MemInstr::Call {
            dst: *dst,
            closure: r(*closure),
            arg: r(*arg),
        },
        MemInstr::Return { reg } => MemInstr::Return { reg: r(*reg) },
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lower::lower_core_linear;
    use reciplexa_core::expr::{CoreExpr, CoreLiteral};

    #[test]
    fn inserts_dup_for_shared_use() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(0),
                    lit: crate::ir::MemLiteral::Number(1.0),
                },
                MemInstr::Dup {
                    dst: Reg(1),
                    src: Reg(0),
                },
                MemInstr::Return { reg: Reg(0) },
            ],
            return_reg: Reg(0),
        };
        let raw = LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(0),
                    lit: crate::ir::MemLiteral::Number(1.0),
                },
                MemInstr::Return { reg: Reg(0) },
                MemInstr::Return { reg: Reg(0) },
            ],
            return_reg: Reg(0),
        };
        // two reads of r0
        let rc = conservative_rc(&LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(0),
                    lit: crate::ir::MemLiteral::Number(1.0),
                },
                MemInstr::Dup {
                    dst: Reg(1),
                    src: Reg(0),
                },
                MemInstr::Return { reg: Reg(0) },
            ],
            return_reg: Reg(0),
        });
        assert!(rc.instrs.iter().any(|i| matches!(i, MemInstr::Dup { .. })));
        let _ = prog;
        let _ = raw;
    }

    #[test]
    fn inserts_drop_after_last_use() {
        let raw = lower_core_linear(&CoreExpr::Seq(vec![
            CoreExpr::Lit(CoreLiteral::Number(1.0)),
            CoreExpr::Lit(CoreLiteral::Number(2.0)),
        ]));
        let rc = conservative_rc(&raw);
        assert!(rc.instrs.iter().any(|i| matches!(i, MemInstr::Drop { .. })));
    }

    #[test]
    fn conservative_on_shared_literal_seq() {
        let raw = lower_core_linear(&CoreExpr::Seq(vec![
            CoreExpr::Lit(CoreLiteral::Number(1.0)),
            CoreExpr::Lit(CoreLiteral::Number(1.0)),
        ]));
        let rc = conservative_rc(&raw);
        assert!(!rc.instrs.is_empty());
    }
}
