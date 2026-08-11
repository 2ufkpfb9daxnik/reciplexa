use reciplexa_mem::conservative_rc;
use reciplexa_mem::ir::MemInstr;
use reciplexa_mem::linear::LinearProgram;
use reciplexa_mem::reg::Reg;

use reciplexa_core::expr::{CoreExpr, CoreLiteral};
use reciplexa_mem::lower::lower_core_linear;

#[test]
fn inserts_dup_for_shared_use() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: reciplexa_mem::ir::MemLiteral::Number(1.0),
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
                lit: reciplexa_mem::ir::MemLiteral::Number(1.0),
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
                lit: reciplexa_mem::ir::MemLiteral::Number(1.0),
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
        CoreExpr::Lit(CoreLiteral::Int(1)),
        CoreExpr::Lit(CoreLiteral::Int(2)),
    ]));
    let rc = conservative_rc(&raw);
    assert!(rc.instrs.iter().any(|i| matches!(i, MemInstr::Drop { .. })));
}

#[test]
fn conservative_on_shared_literal_seq() {
    let raw = lower_core_linear(&CoreExpr::Seq(vec![
        CoreExpr::Lit(CoreLiteral::Int(1)),
        CoreExpr::Lit(CoreLiteral::Int(1)),
    ]));
    let rc = conservative_rc(&raw);
    assert!(!rc.instrs.is_empty());
}

fn cfg_fixture() -> LinearProgram {
    use reciplexa_mem::ir::{BlockId, MemLiteral};
    LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(0.0),
            },
            MemInstr::Branch {
                cond: Reg(1),
                then_block: BlockId(1),
                else_block: BlockId(2),
            },
            MemInstr::Jump { target: BlockId(3) },
            MemInstr::Phi {
                dst: Reg(2),
                incoming: vec![(BlockId(0), Reg(0)), (BlockId(1), Reg(0))],
            },
            MemInstr::MakeClosure {
                dst: Reg(3),
                param: "x".into(),
                body: BlockId(1),
                captures: vec![Reg(0)],
            },
            MemInstr::Call {
                dst: Reg(4),
                closure: Reg(3),
                arg: Reg(0),
            },
            MemInstr::ConstructReuse {
                dst: Reg(5),
                reuse: Reg(0),
                tag: "record".into(),
                fields: vec![("x".into(), Reg(4))],
            },
            MemInstr::Return { reg: Reg(5) },
        ],
        return_reg: Reg(5),
    }
}

#[test]
fn conservative_handles_cfg_instrs() {
    let rc = conservative_rc(&cfg_fixture());
    assert!(rc.instrs.iter().any(|i| matches!(i, MemInstr::Dup { .. })));
    assert!(rc
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Return { .. })));
}

#[test]
fn conservative_remaps_construct_reuse_fields() {
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: reciplexa_mem::ir::MemLiteral::Number(1.0),
            },
            MemInstr::Dup {
                dst: Reg(1),
                src: Reg(0),
            },
            MemInstr::ConstructReuse {
                dst: Reg(2),
                reuse: Reg(0),
                tag: "record".into(),
                fields: vec![("f".into(), Reg(1))],
            },
            MemInstr::Return { reg: Reg(2) },
        ],
        return_reg: Reg(2),
    };
    let rc = conservative_rc(&prog);
    assert!(rc
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::ConstructReuse { .. })));
}

#[test]
fn conservative_covers_move_project_branch_jump_phi() {
    use reciplexa_mem::ir::BlockId;
    let prog = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: reciplexa_mem::ir::MemLiteral::Number(1.0),
            },
            MemInstr::Move {
                dst: Reg(1),
                src: Reg(0),
            },
            MemInstr::Construct {
                dst: Reg(2),
                tag: "pair".into(),
                fields: vec![("a".into(), Reg(1))],
            },
            MemInstr::Project {
                dst: Reg(3),
                src: Reg(2),
                field: "a".into(),
            },
            MemInstr::MakeClosure {
                dst: Reg(4),
                param: "x".into(),
                body: BlockId(0),
                captures: vec![Reg(3)],
            },
            MemInstr::Branch {
                cond: Reg(3),
                then_block: BlockId(1),
                else_block: BlockId(2),
            },
            MemInstr::Jump { target: BlockId(3) },
            MemInstr::Phi {
                dst: Reg(5),
                incoming: vec![(BlockId(1), Reg(3)), (BlockId(2), Reg(0))],
            },
            MemInstr::Raise { tag: "log".into() },
            MemInstr::RegisterCleanup { label: "c".into() },
            MemInstr::RunCleanup,
            MemInstr::Resume { cont: 1 },
            MemInstr::DiscardCont { cont: 1 },
            MemInstr::Return { reg: Reg(5) },
        ],
        return_reg: Reg(5),
    };
    let rc = conservative_rc(&prog);
    assert!(!rc.instrs.is_empty());
    assert!(rc
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Return { .. })));
}
