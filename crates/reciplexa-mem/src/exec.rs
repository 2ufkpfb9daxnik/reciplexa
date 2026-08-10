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
                let v = cell.value.clone();
                heap_insert(&mut heap, *dst, v, trace);
                let c = heap
                    .get_mut(&src)
                    .expect("src cell exists after successful get");
                c.refcount += 1;
                c.unique = false;
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
                    params: vec![param.clone()],
                    body: CoreExpr::Lit(reciplexa_core::expr::CoreLiteral::Number(0.0)),
                    env: std::rc::Rc::new(std::cell::RefCell::new(env)),
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
