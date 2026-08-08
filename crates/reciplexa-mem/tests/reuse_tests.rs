use reciplexa_mem::ir::{MemInstr, MemLiteral};
use reciplexa_mem::linear::LinearProgram;
use reciplexa_mem::reg::Reg;
use reciplexa_mem::reuse_pass;

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
