use reciplexa_mem::ir::{
    collect_regs_instr, BasicBlock, BlockId, MemInstr, MemLiteral, OwningProgram,
};
use reciplexa_mem::linear::LinearProgram;
use reciplexa_mem::reg::Reg;

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

#[test]
fn collect_regs_branch_cond() {
    let mut out = Vec::new();
    collect_regs_instr(
        &MemInstr::Branch {
            cond: Reg(2),
            then_block: BlockId(0),
            else_block: BlockId(1),
        },
        &mut out,
    );
    assert_eq!(out, vec![Reg(2)]);
}
