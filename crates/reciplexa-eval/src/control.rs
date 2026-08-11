//! Evaluation control outcomes and effect host for deep handlers.

use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_outcome::unhandled_failure_report;

use crate::value::RuntimeValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalError {
    pub message: String,
}

impl EvalError {
    pub fn unhandled_failure(payload: impl std::fmt::Display) -> Self {
        let report = unhandled_failure_report(0, payload.to_string());
        Self {
            message: format!("unhandled failure: {report}"),
        }
    }
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
            "random" => Ok(RuntimeValue::F64(0.5)),
            "failure" => Err(EvalError::unhandled_failure(&_arg)),
            "read-file" | "write-file" => Err(EvalError {
                message: format!("unhandled residual effect `{op}`"),
            }),
            other => Err(EvalError {
                message: format!("unknown op `{other}`"),
            }),
        }
    }
}

/// RSC-001 in-memory filesystem host for `read-file` / `write-file`.
#[derive(Debug, Default, Clone)]
pub struct MemoryFsHost {
    pub files: HashMap<String, String>,
}

impl EffectHost for MemoryFsHost {
    fn perform(&mut self, op: &str, arg: RuntimeValue) -> EvalResult {
        match op {
            "log" => Ok(RuntimeValue::Unit),
            "random" => Ok(RuntimeValue::F64(0.5)),
            "read-file" => {
                let RuntimeValue::String(path) = arg else {
                    return Err(EvalError {
                        message: "read-file expects a string path".into(),
                    });
                };
                self.files
                    .get(&path)
                    .cloned()
                    .map(RuntimeValue::String)
                    .ok_or_else(|| EvalError {
                        message: format!("read-file: missing `{path}`"),
                    })
            }
            "write-file" => {
                // Arg encoding: "path\\0content" (NUL-separated).
                let RuntimeValue::String(raw) = arg else {
                    return Err(EvalError {
                        message: "write-file expects a string `path\\0content`".into(),
                    });
                };
                let (path, content) = raw.split_once('\0').ok_or_else(|| EvalError {
                    message: "write-file expects `path\\0content`".into(),
                })?;
                self.files.insert(path.to_string(), content.to_string());
                Ok(RuntimeValue::Unit)
            }
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
    /// DD-EFF-006: handler delegated the current operation to an outer handler.
    Forward,
}

pub fn identity_resume() -> ResumeCont {
    Rc::new(|v, _host| Ok(Outcome::Value(v)))
}
