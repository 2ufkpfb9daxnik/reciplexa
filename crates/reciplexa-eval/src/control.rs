//! Evaluation control outcomes and effect host for deep handlers.

use std::rc::Rc;

use crate::value::RuntimeValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalError {
    pub message: String,
}

pub type EvalResult = Result<RuntimeValue, EvalError>;

pub trait EffectHost {
    fn perform(&mut self, op: &str, arg: RuntimeValue) -> EvalResult;
}

pub struct UnitHost;

impl EffectHost for UnitHost {
    fn perform(&mut self, op: &str, _arg: RuntimeValue) -> EvalResult {
        match op {
            "log" => Ok(RuntimeValue::Unit),
            "random" => Ok(RuntimeValue::Number(0.5)),
            other => Err(EvalError {
                message: format!("unknown op `{other}`"),
            }),
        }
    }
}

/// Continues a computation as if `perform` returned the given value (EFF-001 deep).
pub type ResumeCont = Rc<dyn Fn(RuntimeValue, &mut dyn EffectHost) -> Result<Outcome, EvalError>>;

/// Internal control for deep handlers / one-shot resume (EFF-001).
pub enum Outcome {
    Value(RuntimeValue),
    /// Uncaught perform — bubbles to the nearest matching handle.
    Performed {
        op: String,
        arg: RuntimeValue,
        resume: ResumeCont,
    },
    /// Resume was applied — aborts the handler; value is the handle result.
    Resumed(RuntimeValue),
}

pub fn identity_resume() -> ResumeCont {
    Rc::new(|v, _host| Ok(Outcome::Value(v)))
}
