//! Lowered IR interpreter (Phase 5 §7.2).

use std::collections::HashMap;

use reciplexa_eval::{EffectHost, RuntimeValue, UnitHost};

use crate::continuation::{Continuation, ContinuationError, ContinuationId};
use crate::handler::{HandlerFrame, HandlerKind, LoweredOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpretError {
    UnknownContinuation(ContinuationId),
    DoubleResume(ContinuationId),
    UnhandledRaise(String),
    HandlerMismatch,
}

pub struct Interpreter {
    pub continuations: HashMap<ContinuationId, Continuation>,
    pub handlers: Vec<HandlerFrame>,
    pub host: UnitHost,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            continuations: HashMap::new(),
            handlers: Vec::new(),
            host: UnitHost,
        }
    }

    pub fn register_continuation(&mut self, id: ContinuationId) {
        self.continuations
            .entry(id)
            .or_insert_with(|| Continuation::new(id));
    }

    pub fn push_handler(&mut self, frame: HandlerFrame) {
        self.handlers.push(frame);
    }

    pub fn run(&mut self, ops: &[LoweredOp]) -> Result<RuntimeValue, InterpretError> {
        let mut pc = 0usize;
        let mut value = RuntimeValue::Unit;
        while pc < ops.len() {
            match &ops[pc] {
                LoweredOp::Perform { op } => {
                    value = self
                        .host
                        .perform(op, RuntimeValue::Unit)
                        .map_err(|_| InterpretError::HandlerMismatch)?;
                }
                LoweredOp::Resume { cont } => {
                    let id = ContinuationId(*cont);
                    let slot = self
                        .continuations
                        .get_mut(&id)
                        .ok_or(InterpretError::UnknownContinuation(id))?;
                    slot.resume().map_err(|e| match e {
                        ContinuationError::DoubleResume(id) => InterpretError::DoubleResume(id),
                    })?;
                    value = RuntimeValue::Unit;
                }
                LoweredOp::Raise { tag } => {
                    if let Some(frame) = self.handlers.last() {
                        match &frame.kind {
                            HandlerKind::Failure => {
                                return Err(InterpretError::UnhandledRaise(tag.clone()));
                            }
                            HandlerKind::Op(name) if name == tag => {
                                pc = frame.handler_pc;
                                continue;
                            }
                            _ => {}
                        }
                    }
                    return Err(InterpretError::UnhandledRaise(tag.clone()));
                }
                LoweredOp::Return => {
                    return Ok(value);
                }
            }
            pc += 1;
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perform_and_return() {
        let mut interp = Interpreter::new();
        let ops = vec![
            LoweredOp::Perform {
                op: "log".to_string(),
            },
            LoweredOp::Return,
        ];
        assert!(interp.run(&ops).is_ok());
    }

    #[test]
    fn resume_once_succeeds() {
        let mut interp = Interpreter::new();
        interp.register_continuation(ContinuationId(1));
        let ops = vec![LoweredOp::Resume { cont: 1 }, LoweredOp::Return];
        assert!(interp.run(&ops).is_ok());
    }

    #[test]
    fn double_resume_fails() {
        let mut interp = Interpreter::new();
        interp.register_continuation(ContinuationId(1));
        let ops = vec![LoweredOp::Resume { cont: 1 }, LoweredOp::Resume { cont: 1 }];
        assert!(matches!(
            interp.run(&ops),
            Err(InterpretError::DoubleResume(_))
        ));
    }
}
