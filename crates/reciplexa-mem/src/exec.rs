//! Reference-counting executor with debug trace (Step A).

use std::collections::HashMap;

use reciplexa_core::expr::CoreExpr;
use reciplexa_eval::RuntimeValue;

use crate::ir::{MemInstr, MemLiteral};
use crate::linear::LinearProgram;
use crate::reg::Reg;
use crate::trace::RcTrace;

#[derive(Debug, Clone, PartialEq)]
pub enum ExecError {
    UseAfterDrop(Reg),
    DoubleDrop(Reg),
    RefCountUnderflow(Reg),
    RefCountOverflow(Reg),
    UnboundReg(Reg),
    UnhandledRaise(String),
}

/// Heap cell with reference count.
#[derive(Debug, Clone)]
struct Cell {
    value: RuntimeValue,
    refcount: u32,
    unique: bool,
}

/// Execute a linear ownership program with RC semantics.
pub fn exec_linear(prog: &LinearProgram, trace: &mut RcTrace) -> Result<RuntimeValue, ExecError> {
    let mut heap: HashMap<Reg, Cell> = HashMap::new();
    let mut reg_map: HashMap<Reg, Reg> = HashMap::new();
    let mut cleanups: Vec<String> = Vec::new();
    let mut resumed: HashMap<u64, bool> = HashMap::new();

    for instr in &prog.instrs {
        match instr {
            MemInstr::Lit { dst, lit } => {
                let v = lit_to_value(lit);
                heap_insert(&mut heap, *dst, v, trace);
                reg_map.insert(*dst, *dst);
            }
            MemInstr::Dup { dst, src } => {
                let src = resolve(*src, &reg_map);
                let cell = heap.get(&src).ok_or(ExecError::UnboundReg(src))?;
                if cell.refcount == u32::MAX {
                    return Err(ExecError::RefCountOverflow(src));
                }
                let v = cell.value.clone();
                heap_insert(&mut heap, *dst, v, trace);
                if let Some(c) = heap.get_mut(&src) {
                    c.refcount += 1;
                    c.unique = false;
                }
                trace.record_dup(*dst, src);
                reg_map.insert(*dst, *dst);
            }
            MemInstr::Drop { reg } => {
                let reg = resolve(*reg, &reg_map);
                drop_reg(&mut heap, reg, trace)?;
            }
            MemInstr::Move { dst, src } => {
                let src = resolve(*src, &reg_map);
                let cell = heap.remove(&src).ok_or(ExecError::UnboundReg(src))?;
                heap.insert(*dst, cell);
                reg_map.insert(*dst, *dst);
                reg_map.insert(src, *dst);
            }
            MemInstr::Construct { dst, tag, fields } => {
                let v = construct_value(tag, fields, &heap, &reg_map)?;
                heap_insert(&mut heap, *dst, v, trace);
                reg_map.insert(*dst, *dst);
            }
            MemInstr::ConstructReuse {
                dst,
                reuse,
                tag,
                fields,
            } => {
                let reuse = resolve(*reuse, &reg_map);
                let cell = heap.get(&reuse).ok_or(ExecError::UnboundReg(reuse))?;
                if !cell.unique && cell.refcount > 1 {
                    // fallback to fresh alloc (MEM-05)
                    let v = construct_value(tag, fields, &heap, &reg_map)?;
                    heap_insert(&mut heap, *dst, v, trace);
                } else {
                    let v = construct_value(tag, fields, &heap, &reg_map)?;
                    trace.record_reuse(*dst, reuse);
                    heap.remove(&reuse);
                    heap_insert(&mut heap, *dst, v, trace);
                }
                reg_map.insert(*dst, *dst);
            }
            MemInstr::Project { dst, src, field } => {
                let src = resolve(*src, &reg_map);
                let cell = heap.get(&src).ok_or(ExecError::UnboundReg(src))?;
                let v = project_value(&cell.value, field)?;
                heap_insert(&mut heap, *dst, v, trace);
                reg_map.insert(*dst, *dst);
            }
            MemInstr::Return { reg } => {
                let reg = resolve(*reg, &reg_map);
                let cell = heap.get(&reg).ok_or(ExecError::UnboundReg(reg))?;
                return Ok(cell.value.clone());
            }
            MemInstr::RegisterCleanup { label } => {
                cleanups.push(label.clone());
            }
            MemInstr::RunCleanup => {
                while let Some(label) = cleanups.pop() {
                    trace.record_cleanup(label);
                }
            }
            MemInstr::Raise { tag } => {
                run_cleanups(&mut cleanups, trace);
                return Err(ExecError::UnhandledRaise(tag.clone()));
            }
            MemInstr::Resume { cont } => {
                if resumed.insert(*cont, true).is_some() {
                    return Err(ExecError::DoubleDrop(Reg(*cont as u32)));
                }
            }
            MemInstr::DiscardCont { cont: _ } => {
                run_cleanups(&mut cleanups, trace);
            }
            MemInstr::MakeClosure {
                dst,
                param,
                body: _,
                captures,
            } => {
                let captured: Vec<RuntimeValue> = captures
                    .iter()
                    .map(|r| {
                        let r = resolve(*r, &reg_map);
                        heap.get(&r)
                            .ok_or(ExecError::UnboundReg(r))
                            .map(|c| c.value.clone())
                    })
                    .collect::<Result<_, _>>()?;
                let mut env = HashMap::new();
                for (i, value) in captured.into_iter().enumerate() {
                    env.insert(format!("cap{i}"), value);
                }
                let closure = RuntimeValue::Closure {
                    param: param.clone(),
                    body: CoreExpr::Lit(reciplexa_core::expr::CoreLiteral::Number(0.0)),
                    env,
                };
                heap_insert(&mut heap, *dst, closure, trace);
                reg_map.insert(*dst, *dst);
            }
            _ => {}
        }
    }

    let reg = resolve(prog.effective_return_reg(), &reg_map);
    heap.get(&reg)
        .map(|c| c.value.clone())
        .ok_or(ExecError::UnboundReg(reg))
}

fn resolve(r: Reg, map: &HashMap<Reg, Reg>) -> Reg {
    *map.get(&r).unwrap_or(&r)
}

fn heap_insert(heap: &mut HashMap<Reg, Cell>, reg: Reg, value: RuntimeValue, trace: &mut RcTrace) {
    trace.record_alloc(reg);
    heap.insert(
        reg,
        Cell {
            value,
            refcount: 1,
            unique: true,
        },
    );
}

fn drop_reg(heap: &mut HashMap<Reg, Cell>, reg: Reg, trace: &mut RcTrace) -> Result<(), ExecError> {
    let cell = heap.get_mut(&reg).ok_or(ExecError::UnboundReg(reg))?;
    if cell.refcount == 0 {
        return Err(ExecError::RefCountUnderflow(reg));
    }
    cell.refcount -= 1;
    trace.record_drop(reg);
    if cell.refcount == 0 {
        heap.remove(&reg);
    }
    Ok(())
}

fn run_cleanups(cleanups: &mut Vec<String>, trace: &mut RcTrace) {
    while let Some(label) = cleanups.pop() {
        trace.record_cleanup(label);
    }
}

fn lit_to_value(lit: &MemLiteral) -> RuntimeValue {
    match lit {
        MemLiteral::Number(n) => RuntimeValue::Number(*n),
        MemLiteral::String(s) => RuntimeValue::String(s.clone()),
        MemLiteral::Unit => RuntimeValue::Unit,
    }
}

fn construct_value(
    tag: &str,
    fields: &[(String, Reg)],
    heap: &HashMap<Reg, Cell>,
    map: &HashMap<Reg, Reg>,
) -> Result<RuntimeValue, ExecError> {
    if tag.starts_with("perform:") {
        return Ok(RuntimeValue::Unit);
    }
    if tag == "record" {
        let mut rec = Vec::new();
        for (k, r) in fields {
            let r = resolve(*r, map);
            let v = heap.get(&r).ok_or(ExecError::UnboundReg(r))?.value.clone();
            rec.push((k.clone(), v));
        }
        return Ok(RuntimeValue::Record(rec));
    }
    if let Some(payload) = fields.first() {
        let r = resolve(payload.1, map);
        let v = heap.get(&r).ok_or(ExecError::UnboundReg(r))?.value.clone();
        return Ok(RuntimeValue::Variant {
            tag: tag.to_string(),
            payload: Some(Box::new(v)),
        });
    }
    Ok(RuntimeValue::Variant {
        tag: tag.to_string(),
        payload: None,
    })
}

fn project_value(value: &RuntimeValue, field: &str) -> Result<RuntimeValue, ExecError> {
    match value {
        RuntimeValue::Record(fields) => fields
            .iter()
            .find(|(k, _)| k == field)
            .map(|(_, v)| v.clone())
            .ok_or(ExecError::UnboundReg(Reg(0))),
        RuntimeValue::Variant { payload, .. } if field == "payload" => payload
            .as_ref()
            .map(|b| (**b).clone())
            .ok_or(ExecError::UnboundReg(Reg(0))),
        _ => Err(ExecError::UnboundReg(Reg(0))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::MemLiteral;

    #[test]
    fn exec_literal() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(0),
                    lit: MemLiteral::Number(42.0),
                },
                MemInstr::Return { reg: Reg(0) },
            ],
            return_reg: Reg(0),
        };
        let mut trace = RcTrace::default();
        let v = exec_linear(&prog, &mut trace).unwrap();
        assert_eq!(v, RuntimeValue::Number(42.0));
    }

    #[test]
    fn exec_lit_reg1() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(1),
                    lit: MemLiteral::Number(3.0),
                },
                MemInstr::Return { reg: Reg(1) },
            ],
            return_reg: Reg(1),
        };
        let mut trace = RcTrace::default();
        let v = exec_linear(&prog, &mut trace).unwrap();
        assert_eq!(v, RuntimeValue::Number(3.0));
    }

    #[test]
    fn exec_construct_only() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(1),
                    lit: MemLiteral::Number(3.0),
                },
                MemInstr::Construct {
                    dst: Reg(0),
                    tag: "record".into(),
                    fields: vec![("x".into(), Reg(1))],
                },
                MemInstr::Return { reg: Reg(0) },
            ],
            return_reg: Reg(0),
        };
        let mut trace = RcTrace::default();
        let v = exec_linear(&prog, &mut trace).unwrap();
        assert!(matches!(v, RuntimeValue::Record(_)));
    }

    #[test]
    fn exec_record_with_field_drop() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(1),
                    lit: MemLiteral::Number(3.0),
                },
                MemInstr::Construct {
                    dst: Reg(0),
                    tag: "record".into(),
                    fields: vec![("x".into(), Reg(1))],
                },
                MemInstr::Drop { reg: Reg(1) },
                MemInstr::Return { reg: Reg(0) },
            ],
            return_reg: Reg(0),
        };
        let mut trace = RcTrace::default();
        let v = exec_linear(&prog, &mut trace).unwrap();
        assert!(matches!(v, RuntimeValue::Record(_)));
    }

    #[test]
    fn refcount_dup_drop() {
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
                MemInstr::Return { reg: Reg(1) },
            ],
            return_reg: Reg(1),
        };
        let mut trace = RcTrace::default();
        exec_linear(&prog, &mut trace).unwrap();
        assert_eq!(trace.dups.len(), 1);
        assert_eq!(trace.drops.len(), 1);
    }

    #[test]
    fn exec_unit_literal() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(0),
                    lit: MemLiteral::Unit,
                },
                MemInstr::Return { reg: Reg(0) },
            ],
            return_reg: Reg(0),
        };
        let mut trace = RcTrace::default();
        assert_eq!(exec_linear(&prog, &mut trace).unwrap(), RuntimeValue::Unit);
    }

    #[test]
    fn exec_string_literal() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(0),
                    lit: MemLiteral::String("hi".into()),
                },
                MemInstr::Return { reg: Reg(0) },
            ],
            return_reg: Reg(0),
        };
        let mut trace = RcTrace::default();
        assert_eq!(
            exec_linear(&prog, &mut trace).unwrap(),
            RuntimeValue::String("hi".into())
        );
    }

    #[test]
    fn exec_variant_construct() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(1),
                    lit: MemLiteral::Number(7.0),
                },
                MemInstr::Construct {
                    dst: Reg(0),
                    tag: "Some".into(),
                    fields: vec![("payload".into(), Reg(1))],
                },
                MemInstr::Return { reg: Reg(0) },
            ],
            return_reg: Reg(0),
        };
        let mut trace = RcTrace::default();
        let v = exec_linear(&prog, &mut trace).unwrap();
        assert!(matches!(v, RuntimeValue::Variant { .. }));
    }

    #[test]
    fn exec_project_record_field() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(1),
                    lit: MemLiteral::Number(9.0),
                },
                MemInstr::Construct {
                    dst: Reg(0),
                    tag: "record".into(),
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
        let mut trace = RcTrace::default();
        assert_eq!(
            exec_linear(&prog, &mut trace).unwrap(),
            RuntimeValue::Number(9.0)
        );
    }

    #[test]
    fn exec_raise_runs_cleanups() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::RegisterCleanup {
                    label: "scope".into(),
                },
                MemInstr::Raise { tag: "fail".into() },
            ],
            return_reg: Reg(0),
        };
        let mut trace = RcTrace::default();
        let err = exec_linear(&prog, &mut trace).unwrap_err();
        assert!(matches!(err, ExecError::UnhandledRaise(_)));
        assert_eq!(trace.cleanups, vec!["scope"]);
    }

    #[test]
    fn exec_unbound_reg_errors() {
        let prog = LinearProgram {
            instrs: vec![MemInstr::Return { reg: Reg(99) }],
            return_reg: Reg(99),
        };
        let mut trace = RcTrace::default();
        assert!(matches!(
            exec_linear(&prog, &mut trace),
            Err(ExecError::UnboundReg(_))
        ));
    }

    #[test]
    fn exec_perform_tag_returns_unit() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(1),
                    lit: MemLiteral::String("msg".into()),
                },
                MemInstr::Construct {
                    dst: Reg(0),
                    tag: "perform:log".into(),
                    fields: vec![("arg".into(), Reg(1))],
                },
                MemInstr::Return { reg: Reg(0) },
            ],
            return_reg: Reg(0),
        };
        let mut trace = RcTrace::default();
        assert_eq!(exec_linear(&prog, &mut trace).unwrap(), RuntimeValue::Unit);
    }

    #[test]
    fn exec_make_closure_captures_env() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(0),
                    lit: MemLiteral::Number(9.0),
                },
                MemInstr::MakeClosure {
                    dst: Reg(1),
                    param: "x".into(),
                    body: crate::ir::BlockId(0),
                    captures: vec![Reg(0)],
                },
                MemInstr::Return { reg: Reg(1) },
            ],
            return_reg: Reg(1),
        };
        let mut trace = RcTrace::default();
        let v = exec_linear(&prog, &mut trace).unwrap();
        assert!(matches!(v, RuntimeValue::Closure { .. }));
    }

    #[test]
    fn exec_construct_reuse_unique_records_reuse() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(1),
                    lit: MemLiteral::Number(3.0),
                },
                MemInstr::Construct {
                    dst: Reg(0),
                    tag: "record".into(),
                    fields: vec![("x".into(), Reg(1))],
                },
                MemInstr::ConstructReuse {
                    dst: Reg(2),
                    reuse: Reg(0),
                    tag: "record".into(),
                    fields: vec![("y".into(), Reg(1))],
                },
                MemInstr::Return { reg: Reg(2) },
            ],
            return_reg: Reg(2),
        };
        let mut trace = RcTrace::default();
        exec_linear(&prog, &mut trace).unwrap();
        assert!(!trace.reuses.is_empty());
    }

    #[test]
    fn exec_construct_reuse_shared_falls_back() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(1),
                    lit: MemLiteral::Number(1.0),
                },
                MemInstr::Construct {
                    dst: Reg(0),
                    tag: "record".into(),
                    fields: vec![("x".into(), Reg(1))],
                },
                MemInstr::Dup {
                    dst: Reg(2),
                    src: Reg(0),
                },
                MemInstr::ConstructReuse {
                    dst: Reg(3),
                    reuse: Reg(0),
                    tag: "record".into(),
                    fields: vec![("y".into(), Reg(1))],
                },
                MemInstr::Return { reg: Reg(3) },
            ],
            return_reg: Reg(3),
        };
        let mut trace = RcTrace::default();
        exec_linear(&prog, &mut trace).unwrap();
        assert!(trace.reuses.is_empty());
    }

    #[test]
    fn exec_move_transfers_ownership() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(0),
                    lit: MemLiteral::Number(5.0),
                },
                MemInstr::Move {
                    src: Reg(0),
                    dst: Reg(1),
                },
                MemInstr::Return { reg: Reg(1) },
            ],
            return_reg: Reg(1),
        };
        let mut trace = RcTrace::default();
        assert_eq!(
            exec_linear(&prog, &mut trace).unwrap(),
            RuntimeValue::Number(5.0)
        );
    }

    #[test]
    fn exec_run_cleanup_drains_labels() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::RegisterCleanup { label: "a".into() },
                MemInstr::RegisterCleanup { label: "b".into() },
                MemInstr::RunCleanup,
                MemInstr::Lit {
                    dst: Reg(0),
                    lit: MemLiteral::Unit,
                },
                MemInstr::Return { reg: Reg(0) },
            ],
            return_reg: Reg(0),
        };
        let mut trace = RcTrace::default();
        exec_linear(&prog, &mut trace).unwrap();
        assert_eq!(trace.cleanups, vec!["b", "a"]);
    }

    #[test]
    fn exec_double_resume_errors() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Resume { cont: 1 },
                MemInstr::Resume { cont: 1 },
                MemInstr::Return { reg: Reg(0) },
            ],
            return_reg: Reg(0),
        };
        let mut trace = RcTrace::default();
        assert!(matches!(
            exec_linear(&prog, &mut trace),
            Err(ExecError::DoubleDrop(_))
        ));
    }

    #[test]
    fn exec_project_variant_payload() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(1),
                    lit: MemLiteral::Number(8.0),
                },
                MemInstr::Construct {
                    dst: Reg(0),
                    tag: "Some".into(),
                    fields: vec![("payload".into(), Reg(1))],
                },
                MemInstr::Project {
                    dst: Reg(2),
                    src: Reg(0),
                    field: "payload".into(),
                },
                MemInstr::Return { reg: Reg(2) },
            ],
            return_reg: Reg(2),
        };
        let mut trace = RcTrace::default();
        assert_eq!(
            exec_linear(&prog, &mut trace).unwrap(),
            RuntimeValue::Number(8.0)
        );
    }

    #[test]
    fn exec_project_missing_field_errors() {
        let prog = LinearProgram {
            instrs: vec![
                MemInstr::Lit {
                    dst: Reg(1),
                    lit: MemLiteral::Number(1.0),
                },
                MemInstr::Construct {
                    dst: Reg(0),
                    tag: "record".into(),
                    fields: vec![("x".into(), Reg(1))],
                },
                MemInstr::Project {
                    dst: Reg(2),
                    src: Reg(0),
                    field: "missing".into(),
                },
                MemInstr::Return { reg: Reg(2) },
            ],
            return_reg: Reg(2),
        };
        let mut trace = RcTrace::default();
        assert!(exec_linear(&prog, &mut trace).is_err());
    }
}
