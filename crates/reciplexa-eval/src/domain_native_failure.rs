//! Structured Failure conversion for Direct Native v2 contract violations.
//!
//! Domain-native arity and type errors abort eval like Hybrid RPX apply errors,
//! but the message carries a [`FailureReport`] so hosts can classify them as
//! expected package Failure rather than an unstructured string. They are not
//! yet raised as the `failure` effect (that would change typed effect rows).

use reciplexa_outcome::{FailureCode, FailureReport};

use crate::value::RuntimeValue;
use crate::EvalError;

/// FailureCode namespace for standard-package native callables.
pub const PACKAGE_FAILURE_NS: &str = "package";

pub const PACKAGE_FAILURE_ARITY: &str = "arity";
pub const PACKAGE_FAILURE_TYPE: &str = "type";

/// Build an [`EvalError`] whose message encodes [`FailureReport`].
pub fn package_failure(code: &str, export: &str, detail: impl Into<String>) -> EvalError {
    let detail = detail.into();
    let report = FailureReport::new(
        0,
        FailureCode::new(PACKAGE_FAILURE_NS, code),
        format!("{export}: {detail}"),
    );
    EvalError {
        message: format_package_failure(&report),
    }
}

pub fn arity_error(export: &str, expected: usize, got: usize) -> EvalError {
    package_failure(
        PACKAGE_FAILURE_ARITY,
        export,
        format!("expects {expected} args, got {got}"),
    )
}

pub fn type_error(export: &str, expected: &str, got: impl std::fmt::Display) -> EvalError {
    package_failure(
        PACKAGE_FAILURE_TYPE,
        export,
        format!("expected {expected}, got {got}"),
    )
}

pub(crate) fn take1<'a>(
    args: &'a [RuntimeValue],
    export: &str,
) -> Result<&'a RuntimeValue, EvalError> {
    match args {
        [a] => Ok(a),
        _ => Err(arity_error(export, 1, args.len())),
    }
}

pub(crate) fn take2<'a>(
    args: &'a [RuntimeValue],
    export: &str,
) -> Result<[&'a RuntimeValue; 2], EvalError> {
    match args {
        [a, b] => Ok([a, b]),
        _ => Err(arity_error(export, 2, args.len())),
    }
}

/// Parse a domain-native structured Failure from an eval abort message.
pub fn parse_package_failure(message: &str) -> Option<FailureReport> {
    let rest = message.strip_prefix('[')?;
    let (path, detail) = rest.split_once("] ")?;
    let (namespace, code) = path.split_once('/')?;
    if namespace != PACKAGE_FAILURE_NS || code.is_empty() || detail.is_empty() {
        return None;
    }
    Some(FailureReport::new(
        0,
        FailureCode::new(namespace, code),
        detail,
    ))
}

fn format_package_failure(report: &FailureReport) -> String {
    format!("[{}] {}", report.code.as_path(), report.message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_arity_failure() {
        let err = arity_error("length/units/mm", 1, 0);
        let report = parse_package_failure(&err.message).expect("structured");
        assert_eq!(report.code.as_path(), "package/arity");
        assert!(report.message.contains("length/units/mm"));
        assert!(report.message.contains("expects 1 args, got 0"));
    }

    #[test]
    fn kernel_arity_message_is_not_package_failure() {
        assert!(parse_package_failure("arity mismatch: expected 1 args, got 0").is_none());
    }
}
