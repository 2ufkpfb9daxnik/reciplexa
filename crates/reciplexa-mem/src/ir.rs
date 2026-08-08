//! Ownership Core IR — Perceus target (MEM-001 §4).

use crate::reg::Reg;

/// Basic block label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId(pub u32);

impl BlockId {
    pub const ENTRY: BlockId = BlockId(0);
}

/// Literal payload in mem IR.
#[derive(Debug, Clone, PartialEq)]
pub enum MemLiteral {
    Number(f64),
    String(String),
    Unit,
}

/// Single ownership instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum MemInstr {
    /// `dst = lit`
    Lit { dst: Reg, lit: MemLiteral },
    /// `dst = dup src` — share ownership (refcount++)
    Dup { dst: Reg, src: Reg },
    /// `drop reg` — release if refcount hits zero
    Drop { reg: Reg },
    /// `dst = move src` — transfer unique ownership; `src` is invalidated
    Move { dst: Reg, src: Reg },
    /// `dst = record { fields }`
    Construct {
        dst: Reg,
        tag: String,
        fields: Vec<(String, Reg)>,
    },
    /// `dst = reuse src as tag { fields }` — in-place when `src` is unique
    ConstructReuse {
        dst: Reg,
        reuse: Reg,
        tag: String,
        fields: Vec<(String, Reg)>,
    },
    /// `dst = src.field`
    Project { dst: Reg, src: Reg, field: String },
    /// `dst = closure(param, body_block, captures)`
    MakeClosure {
        dst: Reg,
        param: String,
        body: BlockId,
        captures: Vec<Reg>,
    },
    /// `dst = call closure arg`
    Call { dst: Reg, closure: Reg, arg: Reg },
    /// Branch on numeric zero/nonzero.
    Branch {
        cond: Reg,
        then_block: BlockId,
        else_block: BlockId,
    },
    /// Unconditional jump.
    Jump { target: BlockId },
    /// Return value from function body.
    Return { reg: Reg },
    /// Failure path — run cleanup then raise.
    Raise { tag: String },
    /// Register a cleanup label (bracket / scope).
    RegisterCleanup { label: String },
    /// Run pending cleanups LIFO.
    RunCleanup,
    /// Resume one-shot continuation.
    Resume { cont: u64 },
    /// Mark continuation discarded — run cleanups, drop captures.
    DiscardCont { cont: u64 },
    /// Phi at join: `dst` merges values from predecessor blocks.
    Phi {
        dst: Reg,
        incoming: Vec<(BlockId, Reg)>,
    },
}

/// One basic block.
#[derive(Debug, Clone, PartialEq)]
pub struct BasicBlock {
    pub id: BlockId,
    pub instrs: Vec<MemInstr>,
    pub terminator: MemInstr,
}

/// Whole ownership program (CFG).
#[derive(Debug, Clone, PartialEq)]
pub struct OwningProgram {
    pub blocks: Vec<BasicBlock>,
    pub entry: BlockId,
    pub return_reg: Option<Reg>,
}

impl OwningProgram {
    pub fn block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.iter().find(|b| b.id == id)
    }

    pub fn block_mut(&mut self, id: BlockId) -> Option<&mut BasicBlock> {
        self.blocks.iter_mut().find(|b| b.id == id)
    }

    pub fn all_regs(&self) -> Vec<Reg> {
        let mut out = Vec::new();
        for bb in &self.blocks {
            collect_regs_instrs(&bb.instrs, &mut out);
            collect_regs_instr(&bb.terminator, &mut out);
        }
        out.sort_by_key(|r| r.0);
        out.dedup();
        out
    }
}

fn collect_regs_instrs(instrs: &[MemInstr], out: &mut Vec<Reg>) {
    for i in instrs {
        collect_regs_instr(i, out);
    }
}

pub(crate) fn collect_regs_instr(instr: &MemInstr, out: &mut Vec<Reg>) {
    match instr {
        MemInstr::Lit { dst, .. } => push_reg(out, *dst),
        MemInstr::Dup { dst, src } | MemInstr::Move { dst, src } => {
            push_reg(out, *dst);
            push_reg(out, *src);
        }
        MemInstr::Drop { reg } | MemInstr::Return { reg } => push_reg(out, *reg),
        MemInstr::Construct { dst, fields, .. } => {
            push_reg(out, *dst);
            for (_, r) in fields {
                push_reg(out, *r);
            }
        }
        MemInstr::ConstructReuse {
            dst, reuse, fields, ..
        } => {
            push_reg(out, *dst);
            push_reg(out, *reuse);
            for (_, r) in fields {
                push_reg(out, *r);
            }
        }
        MemInstr::Project { dst, src, .. } => {
            push_reg(out, *dst);
            push_reg(out, *src);
        }
        MemInstr::MakeClosure { dst, captures, .. } => {
            push_reg(out, *dst);
            for r in captures {
                push_reg(out, *r);
            }
        }
        MemInstr::Call { dst, closure, arg } => {
            push_reg(out, *dst);
            push_reg(out, *closure);
            push_reg(out, *arg);
        }
        MemInstr::Branch { cond, .. } => push_reg(out, *cond),
        MemInstr::Phi { dst, incoming } => {
            push_reg(out, *dst);
            for (_, r) in incoming {
                push_reg(out, *r);
            }
        }
        MemInstr::Jump { .. }
        | MemInstr::Raise { .. }
        | MemInstr::RegisterCleanup { .. }
        | MemInstr::RunCleanup
        | MemInstr::Resume { .. }
        | MemInstr::DiscardCont { .. } => {}
    }
}

fn push_reg(out: &mut Vec<Reg>, r: Reg) {
    if !out.contains(&r) {
        out.push(r);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linear::LinearProgram;

    fn lit_instr(dst: Reg) -> MemInstr {
        MemInstr::Lit {
            dst,
            lit: MemLiteral::Number(1.0),
        }
    }

    #[test]
    fn block_id_entry_is_zero() {
        assert_eq!(BlockId::ENTRY, BlockId(0));
    }

    #[test]
    fn mem_literal_eq() {
        assert_eq!(MemLiteral::Unit, MemLiteral::Unit);
        assert_ne!(MemLiteral::Number(1.0), MemLiteral::Number(2.0));
    }

    #[test]
    fn owning_program_block_lookup() {
        let bb = BasicBlock {
            id: BlockId(0),
            instrs: vec![lit_instr(Reg(0))],
            terminator: MemInstr::Return { reg: Reg(0) },
        };
        let prog = OwningProgram {
            blocks: vec![bb],
            entry: BlockId::ENTRY,
            return_reg: Some(Reg(0)),
        };
        assert!(prog.block(BlockId(0)).is_some());
        assert!(prog.block(BlockId(1)).is_none());
        let mut mut_prog = prog.clone();
        assert!(mut_prog.block_mut(BlockId(0)).is_some());
    }

    #[test]
    fn all_regs_collects_unique_sorted() {
        let prog = LinearProgram {
            instrs: vec![
                lit_instr(Reg(2)),
                MemInstr::Dup {
                    dst: Reg(1),
                    src: Reg(2),
                },
                MemInstr::Return { reg: Reg(1) },
            ],
            return_reg: Reg(1),
        };
        let owning = OwningProgram {
            blocks: vec![BasicBlock {
                id: BlockId(0),
                instrs: prog.instrs.clone(),
                terminator: MemInstr::Return { reg: Reg(1) },
            }],
            entry: BlockId::ENTRY,
            return_reg: Some(Reg(1)),
        };
        let regs = owning.all_regs();
        assert_eq!(regs, vec![Reg(1), Reg(2)]);
    }

    #[test]
    fn collect_regs_skips_control_only_instrs() {
        let mut out = Vec::new();
        collect_regs_instr(&MemInstr::Jump { target: BlockId(1) }, &mut out);
        assert!(out.is_empty());
        collect_regs_instr(
            &MemInstr::Phi {
                dst: Reg(0),
                incoming: vec![(BlockId(0), Reg(1))],
            },
            &mut out,
        );
        assert_eq!(out, vec![Reg(0), Reg(1)]);
    }
}
