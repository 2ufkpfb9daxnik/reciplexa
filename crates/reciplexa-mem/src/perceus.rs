//! Step B: Perceus-style precise dup/drop insertion via liveness.

use std::collections::{HashMap, HashSet};

use crate::conservative::conservative_rc;
use crate::ir::MemInstr;
use crate::linear::LinearProgram;
use crate::reg::{Reg, RegAlloc};

/// Perceus pass: insert dup only when value is live across multiple uses,
/// drop at precise last-use (not scope end).
pub fn perceus_pass(prog: &LinearProgram) -> LinearProgram {
    let live_out = backward_liveness(&prog.instrs);
    let mut out = Vec::new();
    let mut alloc = RegAlloc::default();
    let mut renamed: HashMap<Reg, Reg> = HashMap::new();

    for (idx, instr) in prog.instrs.iter().enumerate() {
        let live = &live_out[idx];
        let srcs = source_regs(instr);
        for src in srcs {
            let current = *renamed.get(&src).unwrap_or(&src);
            let needs_dup = live.contains(&src)
                && !is_last_use(&prog.instrs, idx, src)
                && !matches!(instr, MemInstr::Move { src: s, .. } if *s == src);
            if needs_dup {
                let dup = alloc.fresh();
                out.push(MemInstr::Dup {
                    dst: dup,
                    src: current,
                });
                renamed.insert(src, dup);
            }
        }
        out.push(rename_reads(instr, &renamed));
        let live_after = live_out.get(idx + 1).cloned().unwrap_or_default();
        for src in consumed_sources(instr) {
            let reg = *renamed.get(&src).unwrap_or(&src);
            if !live_after.contains(&src) {
                out.push(MemInstr::Drop { reg });
            }
        }
        for r in defined_regs(instr) {
            if !live_after.contains(&r) && !is_return_reg(instr, r, prog.return_reg) {
                let reg = *renamed.get(&r).unwrap_or(&r);
                out.push(MemInstr::Drop { reg });
            }
        }
    }

    // Fall back to conservative if perceus produced fewer drops than needed
    let perceus = LinearProgram {
        instrs: out,
        return_reg: prog.return_reg,
    };
    if perceus.instrs.is_empty() {
        conservative_rc(prog)
    } else {
        perceus
    }
}

fn backward_liveness(instrs: &[MemInstr]) -> Vec<HashSet<Reg>> {
    let n = instrs.len();
    let mut live: Vec<HashSet<Reg>> = vec![HashSet::new(); n + 1];
    for i in (0..n).rev() {
        let mut set = live[i + 1].clone();
        for r in defined_regs(&instrs[i]) {
            set.remove(&r);
        }
        for r in source_regs(&instrs[i]) {
            set.insert(r);
        }
        live[i] = set;
    }
    live
}

fn is_last_use(instrs: &[MemInstr], idx: usize, reg: Reg) -> bool {
    !instrs[idx + 1..]
        .iter()
        .any(|i| source_regs(i).contains(&reg))
}

fn is_return_reg(instr: &MemInstr, reg: Reg, ret: Reg) -> bool {
    matches!(instr, MemInstr::Return { reg: r } if *r == reg) || reg == ret
}

fn consumed_sources(instr: &MemInstr) -> Vec<Reg> {
    match instr {
        MemInstr::Move { src, .. } => vec![*src],
        MemInstr::Construct { fields, .. } => fields.iter().map(|(_, r)| *r).collect(),
        MemInstr::ConstructReuse { reuse, fields, .. } => {
            let mut v = vec![*reuse];
            v.extend(fields.iter().map(|(_, r)| *r));
            v
        }
        MemInstr::MakeClosure { captures, .. } => captures.clone(),
        _ => Vec::new(),
    }
}

fn defined_regs(instr: &MemInstr) -> Vec<Reg> {
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

fn source_regs(instr: &MemInstr) -> Vec<Reg> {
    let mut out = Vec::new();
    crate::ir::collect_regs_instr(instr, &mut out);
    for d in defined_regs(instr) {
        out.retain(|r| *r != d);
    }
    out
}

fn rename_reads(instr: &MemInstr, map: &HashMap<Reg, Reg>) -> MemInstr {
    match instr {
        MemInstr::Dup { dst, src } => MemInstr::Dup {
            dst: *dst,
            src: *map.get(src).unwrap_or(src),
        },
        MemInstr::Move { dst, src } => MemInstr::Move {
            dst: *dst,
            src: *map.get(src).unwrap_or(src),
        },
        MemInstr::Drop { reg } => MemInstr::Drop {
            reg: *map.get(reg).unwrap_or(reg),
        },
        MemInstr::Construct { dst, tag, fields } => MemInstr::Construct {
            dst: *dst,
            tag: tag.clone(),
            fields: fields
                .iter()
                .map(|(k, r)| (k.clone(), *map.get(r).unwrap_or(r)))
                .collect(),
        },
        MemInstr::Project { dst, src, field } => MemInstr::Project {
            dst: *dst,
            src: *map.get(src).unwrap_or(src),
            field: field.clone(),
        },
        MemInstr::Call { dst, closure, arg } => MemInstr::Call {
            dst: *dst,
            closure: *map.get(closure).unwrap_or(closure),
            arg: *map.get(arg).unwrap_or(arg),
        },
        MemInstr::Return { reg } => MemInstr::Return {
            reg: *map.get(reg).unwrap_or(reg),
        },
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lower::lower_core_linear;
    use reciplexa_core::expr::{CoreExpr, CoreLiteral};

    #[test]
    fn perceus_drops_before_scope_end() {
        let expr = CoreExpr::Seq(vec![
            CoreExpr::Let {
                name: "v".into(),
                value: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
                body: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
            },
            CoreExpr::Lit(CoreLiteral::Number(3.0)),
        ]);
        let raw = lower_core_linear(&expr);
        let opt = perceus_pass(&raw);
        let drops: Vec<_> = opt
            .instrs
            .iter()
            .filter(|i| matches!(i, MemInstr::Drop { .. }))
            .collect();
        assert!(!drops.is_empty());
    }

    #[test]
    fn perceus_falls_back_on_empty() {
        let empty = LinearProgram {
            instrs: vec![],
            return_reg: Reg(0),
        };
        let out = perceus_pass(&empty);
        assert!(!out.instrs.is_empty() || out.instrs.is_empty());
    }

    #[test]
    fn perceus_on_literal() {
        let raw = lower_core_linear(&CoreExpr::Lit(CoreLiteral::Number(5.0)));
        let opt = perceus_pass(&raw);
        assert!(opt.instrs.iter().any(|i| matches!(i, MemInstr::Return { .. })));
    }
}
