//! Step C: reuse analysis — unique constructor recycling.

use crate::ir::MemInstr;
use crate::linear::LinearProgram;
use crate::reg::Reg;
use crate::state::{OwnMap, OwnState};

/// Attempt reuse specialization on constructs following unique drops.
pub fn reuse_pass(prog: LinearProgram) -> LinearProgram {
    let mut out = Vec::new();
    let mut state = OwnMap::new();
    let mut reuse_pool: Vec<(String, Reg)> = Vec::new();

    for instr in &prog.instrs {
        match instr {
            MemInstr::Construct { dst, tag, fields } => {
                if let Some((_, reuse_reg)) = find_reuse_candidate(&reuse_pool, tag) {
                    if state.get(reuse_reg) == OwnState::Alive {
                        out.push(MemInstr::ConstructReuse {
                            dst: *dst,
                            reuse: reuse_reg,
                            tag: tag.clone(),
                            fields: fields.clone(),
                        });
                        state.move_from(reuse_reg, *dst);
                        reuse_pool.retain(|(_, r)| *r != reuse_reg);
                        continue;
                    }
                }
                out.push(instr.clone());
                state.define(*dst);
            }
            MemInstr::Drop { reg } => {
                if state.get(*reg) == OwnState::Alive {
                    if let Some(tag) = tag_for_reg(&out, *reg) {
                        if is_reusable_tag(&tag) {
                            reuse_pool.push((tag, *reg));
                            continue;
                        }
                    }
                }
                out.push(instr.clone());
                state.drop_reg(*reg);
            }
            _ => {
                apply_effect(&mut state, instr);
                out.push(instr.clone());
            }
        }
    }

    for (_, reg) in reuse_pool {
        out.push(MemInstr::Drop { reg });
        state.drop_reg(reg);
    }

    LinearProgram {
        instrs: out,
        return_reg: prog.return_reg,
    }
}

fn find_reuse_candidate(pool: &[(String, Reg)], tag: &str) -> Option<(String, Reg)> {
    pool.iter()
        .find(|(t, _)| t == tag)
        .map(|(t, r)| (t.clone(), *r))
}

fn is_reusable_tag(tag: &str) -> bool {
    !tag.starts_with("perform:") && tag != "closure"
}

fn tag_for_reg(instrs: &[MemInstr], reg: Reg) -> Option<String> {
    for i in instrs {
        if let MemInstr::Construct { dst, tag, .. } = i {
            if *dst == reg {
                return Some(tag.clone());
            }
        }
    }
    None
}

fn apply_effect(state: &mut OwnMap, instr: &MemInstr) {
    match instr {
        MemInstr::Move { src, dst } => state.move_from(*src, *dst),
        MemInstr::Dup { dst, .. } => state.define(*dst),
        MemInstr::Lit { dst, .. } | MemInstr::ConstructReuse { dst, .. } => state.define(*dst),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::MemLiteral;

    #[test]
    fn reuse_when_unique() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Construct {
                    dst: Reg(0),
                    tag: "record".into(),
                    fields: vec![],
                },
                MemInstr::Drop { reg: Reg(0) },
                MemInstr::Construct {
                    dst: Reg(1),
                    tag: "record".into(),
                    fields: vec![],
                },
            ],
            return_reg: Reg(1),
        };
        let out = reuse_pass(prog);
        assert!(out
            .instrs
            .iter()
            .any(|i| matches!(i, MemInstr::ConstructReuse { .. })));
    }

    #[test]
    fn no_reuse_for_perform_tag() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Construct {
                    dst: Reg(0),
                    tag: "perform:log".into(),
                    fields: vec![],
                },
                MemInstr::Drop { reg: Reg(0) },
                MemInstr::Construct {
                    dst: Reg(1),
                    tag: "perform:log".into(),
                    fields: vec![],
                },
            ],
            return_reg: Reg(1),
        };
        let out = reuse_pass(prog);
        assert!(!out
            .instrs
            .iter()
            .any(|i| matches!(i, MemInstr::ConstructReuse { .. })));
    }

    #[test]
    fn shared_value_falls_back_to_construct() {
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
                MemInstr::Drop { reg: Reg(0) },
                MemInstr::Construct {
                    dst: Reg(2),
                    tag: "record".into(),
                    fields: vec![("x".into(), Reg(1))],
                },
            ],
            return_reg: Reg(2),
        };
        let out = reuse_pass(prog);
        assert!(
            out.instrs
                .iter()
                .filter(|i| matches!(i, MemInstr::ConstructReuse { .. }))
                .count()
                == 0
        );
    }
}
