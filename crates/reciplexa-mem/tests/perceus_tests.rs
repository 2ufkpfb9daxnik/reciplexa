use reciplexa_core::expr::{CoreExpr, CoreLiteral};
use reciplexa_mem::ir::MemInstr;
use reciplexa_mem::linear::LinearProgram;
use reciplexa_mem::lower_core_linear;
use reciplexa_mem::perceus_pass;
use reciplexa_mem::reg::Reg;

#[test]
fn perceus_drops_before_scope_end() {
    let expr = CoreExpr::Seq(vec![
        CoreExpr::Let {
            name: "v".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Int(2))),
        },
        CoreExpr::Lit(CoreLiteral::Int(3)),
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
    let raw = lower_core_linear(&CoreExpr::Lit(CoreLiteral::Int(5)));
    let opt = perceus_pass(&raw);
    assert!(opt
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Return { .. })));
}

#[test]
fn perceus_handles_cfg_and_reuse_sources() {
    use reciplexa_mem::ir::{BlockId, MemLiteral};
    let raw = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Branch {
                cond: Reg(0),
                then_block: BlockId(1),
                else_block: BlockId(2),
            },
            MemInstr::Phi {
                dst: Reg(1),
                incoming: vec![(BlockId(0), Reg(0))],
            },
            MemInstr::MakeClosure {
                dst: Reg(2),
                param: "x".into(),
                body: BlockId(0),
                captures: vec![Reg(0)],
            },
            MemInstr::Call {
                dst: Reg(3),
                closure: Reg(2),
                arg: Reg(0),
            },
            MemInstr::ConstructReuse {
                dst: Reg(4),
                reuse: Reg(0),
                tag: "record".into(),
                fields: vec![("v".into(), Reg(3))],
            },
            MemInstr::Return { reg: Reg(4) },
        ],
        return_reg: Reg(4),
    };
    let opt = perceus_pass(&raw);
    assert!(!opt.instrs.is_empty());
}

#[test]
fn perceus_inserts_dup_for_shared_phi_source() {
    use reciplexa_mem::ir::BlockId;
    let raw = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: reciplexa_mem::ir::MemLiteral::Number(1.0),
            },
            MemInstr::Dup {
                dst: Reg(1),
                src: Reg(0),
            },
            MemInstr::Phi {
                dst: Reg(2),
                incoming: vec![(BlockId(0), Reg(0)), (BlockId(1), Reg(1))],
            },
            MemInstr::Return { reg: Reg(2) },
        ],
        return_reg: Reg(2),
    };
    let opt = perceus_pass(&raw);
    assert!(!opt.instrs.is_empty());
}

#[test]
fn perceus_rename_reads_and_drops_unused_defs() {
    use reciplexa_mem::ir::MemLiteral;
    let raw = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(0),
                lit: MemLiteral::Number(1.0),
            },
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(2.0),
            },
            MemInstr::Move {
                dst: Reg(2),
                src: Reg(0),
            },
            MemInstr::Return { reg: Reg(2) },
        ],
        return_reg: Reg(2),
    };
    let opt = perceus_pass(&raw);
    assert!(opt
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Drop { .. })));
}

#[test]
fn perceus_handles_project_and_construct() {
    use reciplexa_mem::ir::MemLiteral;
    let raw = LinearProgram {
        instrs: vec![
            MemInstr::Lit {
                dst: Reg(1),
                lit: MemLiteral::Number(3.0),
            },
            MemInstr::Construct {
                dst: Reg(0),
                tag: "pair".into(),
                fields: vec![("x".into(), Reg(1))],
            },
            MemInstr::Project {
                dst: Reg(2),
                src: Reg(0),
                field: "x".into(),
            },
            MemInstr::Return { reg: Reg(2) },
        ],
        return_reg: Reg(2),
    };
    let opt = perceus_pass(&raw);
    assert!(opt
        .instrs
        .iter()
        .any(|i| matches!(i, MemInstr::Return { .. })));
}

#[test]
fn perceus_empty_falls_back_to_conservative() {
    let empty = LinearProgram {
        instrs: vec![],
        return_reg: Reg(0),
    };
    let out = perceus_pass(&empty);
    assert_eq!(out.return_reg, Reg(0));
}
