//! Lower typed Core expressions to linear ownership IR.

use std::collections::HashMap;

use reciplexa_core::expr::{CoreExpr, CoreLiteral, MatchArm};

use crate::ir::{MemInstr, MemLiteral};
use crate::linear::LinearProgram;
use crate::reg::{Reg, RegAlloc};

/// Lower core expression to linear ownership IR (no dup/drop yet).
pub fn lower_core_linear(expr: &CoreExpr) -> LinearProgram {
    let mut lowerer = Lowerer::new();
    let ret = lowerer.lower_expr(expr, &HashMap::new());
    lowerer.emit(MemInstr::Return { reg: ret });
    LinearProgram {
        instrs: lowerer.instrs,
        return_reg: ret,
    }
}

struct Lowerer {
    alloc: RegAlloc,
    instrs: Vec<MemInstr>,
}

impl Lowerer {
    fn new() -> Self {
        Self {
            alloc: RegAlloc::default(),
            instrs: Vec::new(),
        }
    }

    fn emit(&mut self, instr: MemInstr) {
        self.instrs.push(instr);
    }

    fn lower_expr(&mut self, expr: &CoreExpr, env: &HashMap<String, Reg>) -> Reg {
        match expr {
            CoreExpr::Lit(lit) => {
                let dst = self.alloc.fresh();
                let mem_lit = match lit {
                    CoreLiteral::Number(n) => MemLiteral::Number(*n),
                    CoreLiteral::String(s) | CoreLiteral::Color(s) => MemLiteral::String(s.clone()),
                    CoreLiteral::Bool(b) => MemLiteral::String(b.to_string()),
                };
                self.emit(MemInstr::Lit { dst, lit: mem_lit });
                dst
            }
            CoreExpr::Var(name) => env.get(name).copied().unwrap_or_else(|| self.unit()),
            CoreExpr::Seq(items) => {
                let mut last = self.unit();
                for item in items {
                    last = self.lower_expr(item, env);
                }
                last
            }
            CoreExpr::Let { name, value, body } => {
                let v = self.lower_expr(value, env);
                let mut child = env.clone();
                child.insert(name.clone(), v);
                self.lower_expr(body, &child)
            }
            CoreExpr::Lambda { params, body } => {
                let dst = self.alloc.fresh();
                let captures: Vec<Reg> = env.values().copied().collect();
                self.emit(MemInstr::MakeClosure {
                    dst,
                    param: params.join(","),
                    body: crate::ir::BlockId(0),
                    captures,
                });
                // Body stored symbolically; exec inlines via closure table
                let _ = body;
                dst
            }
            CoreExpr::App { fun, args } => {
                let f = self.lower_expr(fun, env);
                let arg_regs: Vec<Reg> = args.iter().map(|a| self.lower_expr(a, env)).collect();
                let mut closure = f;
                if arg_regs.is_empty() {
                    let arg = self.unit();
                    let dst = self.alloc.fresh();
                    self.emit(MemInstr::Call {
                        dst,
                        closure,
                        arg,
                    });
                    return dst;
                }
                for a in arg_regs {
                    let dst = self.alloc.fresh();
                    self.emit(MemInstr::Call {
                        dst,
                        closure,
                        arg: a,
                    });
                    closure = dst;
                }
                closure
            }
            CoreExpr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let _ = self.lower_expr(cond, env);
                let t = self.lower_expr(then_branch, env);
                let _ = self.lower_expr(else_branch, env);
                t
            }
            CoreExpr::Record { fields } => {
                let dst = self.alloc.fresh();
                let lowered: Vec<_> = fields
                    .iter()
                    .map(|(k, v)| (k.clone(), self.lower_expr(v, env)))
                    .collect();
                self.emit(MemInstr::Construct {
                    dst,
                    tag: "record".into(),
                    fields: lowered,
                });
                dst
            }
            CoreExpr::RecordGet { record, field } => {
                let src = self.lower_expr(record, env);
                let dst = self.alloc.fresh();
                self.emit(MemInstr::Project {
                    dst,
                    src,
                    field: field.clone(),
                });
                dst
            }
            CoreExpr::Variant { tag, payload } => {
                let dst = self.alloc.fresh();
                let fields = if let Some(p) = payload {
                    vec![("payload".into(), self.lower_expr(p, env))]
                } else {
                    Vec::new()
                };
                self.emit(MemInstr::Construct {
                    dst,
                    tag: tag.clone(),
                    fields,
                });
                dst
            }
            CoreExpr::Match { scrutinee, arms } => {
                if let CoreExpr::Variant { tag, payload } = scrutinee.as_ref() {
                    if let Some(arm) = arms.iter().find(|a| &a.tag == tag) {
                        let mut child = env.clone();
                        if let Some(bind) = &arm.bind {
                            if let Some(p) = payload {
                                let payload_reg = self.lower_expr(p, env);
                                child.insert(bind.clone(), payload_reg);
                            }
                        }
                        return self.lower_expr(&arm.body, &child);
                    }
                }
                self.lower_match(scrutinee, arms, env)
            }
            CoreExpr::Perform { op, arg } => {
                let a = self.lower_expr(arg, env);
                let dst = self.alloc.fresh();
                self.emit(MemInstr::Construct {
                    dst,
                    tag: format!("perform:{op}"),
                    fields: vec![("arg".into(), a)],
                });
                dst
            }
            CoreExpr::Handle {
                handler_body,
                body,
                ..
            } => {
                // Mem lowering does not model handlers yet; evaluate body then handler stub.
                let _ = self.lower_expr(body, env);
                self.lower_expr(handler_body, env)
            }
        }
    }

    fn lower_match(
        &mut self,
        scrutinee: &CoreExpr,
        arms: &[MatchArm],
        env: &HashMap<String, Reg>,
    ) -> Reg {
        let scr = self.lower_expr(scrutinee, env);
        let dst = self.alloc.fresh();
        for (i, arm) in arms.iter().enumerate() {
            let scr_use = if i == 0 {
                scr
            } else {
                let dup = self.alloc.fresh();
                self.emit(MemInstr::Dup { dst: dup, src: scr });
                dup
            };
            let mut child = env.clone();
            if let Some(bind) = &arm.bind {
                let payload = self.alloc.fresh();
                self.emit(MemInstr::Project {
                    dst: payload,
                    src: scr_use,
                    field: "payload".into(),
                });
                child.insert(bind.clone(), payload);
            }
            let body_reg = self.lower_expr(&arm.body, &child);
            self.emit(MemInstr::Move { dst, src: body_reg });
        }
        dst
    }

    fn unit(&mut self) -> Reg {
        let dst = self.alloc.fresh();
        self.emit(MemInstr::Lit {
            dst,
            lit: MemLiteral::Unit,
        });
        dst
    }
}
