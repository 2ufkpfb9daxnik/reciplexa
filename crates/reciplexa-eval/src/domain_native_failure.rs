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

pub(crate) fn expect_number(v: &RuntimeValue, export: &str) -> Result<f64, EvalError> {
    match v {
        RuntimeValue::Number(n) | RuntimeValue::F64(n) => Ok(*n),
        RuntimeValue::Int(n) => Ok(*n as f64),
        other => Err(type_error(export, "number", other)),
    }
}

pub(crate) fn expect_string<'a>(v: &'a RuntimeValue, export: &str) -> Result<&'a str, EvalError> {
    match v {
        RuntimeValue::String(s) => Ok(s.as_str()),
        other => Err(type_error(export, "string", other)),
    }
}

pub(crate) fn expect_nonempty_char(v: &RuntimeValue, export: &str) -> Result<char, EvalError> {
    let s = expect_string(v, export)?;
    s.chars()
        .next()
        .ok_or_else(|| type_error(export, "non-empty string", "empty string"))
}

pub(crate) fn expect_class_id(v: &RuntimeValue, export: &str) -> Result<u8, EvalError> {
    match v {
        RuntimeValue::Int(n) if (0..=255).contains(n) => Ok(*n as u8),
        RuntimeValue::Number(n) | RuntimeValue::F64(n)
            if n.fract() == 0.0 && (0.0..=255.0).contains(n) =>
        {
            Ok(*n as u8)
        }
        other => Err(type_error(export, "class id (0..255 int)", other)),
    }
}

pub(crate) fn expect_record_field(
    rec: &RuntimeValue,
    key: &str,
    export: &str,
) -> Result<RuntimeValue, EvalError> {
    match rec {
        RuntimeValue::Record(fields) => fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .ok_or_else(|| type_error(export, &format!("record field `{key}`"), "missing field")),
        other => Err(type_error(export, "record", other)),
    }
}

pub(crate) fn take1_class_id(args: &[RuntimeValue], export: &str) -> Result<u8, EvalError> {
    let [v] = take1(args, export)?;
    expect_class_id(v, export)
}

pub(crate) fn take2_class_ids(args: &[RuntimeValue], export: &str) -> Result<(u8, u8), EvalError> {
    let [a, b] = take2(args, export)?;
    Ok((expect_class_id(a, export)?, expect_class_id(b, export)?))
}

pub(crate) fn take0(args: &[RuntimeValue], export: &str) -> Result<(), EvalError> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(arity_error(export, 0, args.len()))
    }
}

pub(crate) fn take1<'a>(
    args: &'a [RuntimeValue],
    export: &str,
) -> Result<[&'a RuntimeValue; 1], EvalError> {
    match args {
        [a] => Ok([a]),
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

pub(crate) fn take3<'a>(
    args: &'a [RuntimeValue],
    export: &str,
) -> Result<[&'a RuntimeValue; 3], EvalError> {
    match args {
        [a, b, c] => Ok([a, b, c]),
        _ => Err(arity_error(export, 3, args.len())),
    }
}

pub(crate) fn take4<'a>(
    args: &'a [RuntimeValue],
    export: &str,
) -> Result<[&'a RuntimeValue; 4], EvalError> {
    match args {
        [a, b, c, d] => Ok([a, b, c, d]),
        _ => Err(arity_error(export, 4, args.len())),
    }
}

pub(crate) fn take5<'a>(
    args: &'a [RuntimeValue],
    export: &str,
) -> Result<[&'a RuntimeValue; 5], EvalError> {
    match args {
        [a, b, c, d, e] => Ok([a, b, c, d, e]),
        _ => Err(arity_error(export, 5, args.len())),
    }
}

pub(crate) fn take6<'a>(
    args: &'a [RuntimeValue],
    export: &str,
) -> Result<[&'a RuntimeValue; 6], EvalError> {
    match args {
        [a, b, c, d, e, f] => Ok([a, b, c, d, e, f]),
        _ => Err(arity_error(export, 6, args.len())),
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

    fn assert_arity(op: crate::DomainNativeOp, extra: &[RuntimeValue], expected: usize) {
        let err = crate::call_domain_native(op, extra).expect_err("arity");
        let report = err.failure_report().expect("structured");
        assert_eq!(report.code.as_path(), "package/arity", "{op:?}");
        assert!(
            report
                .message
                .contains(&format!("expects {expected} args, got {}", extra.len())),
            "{op:?}: {}",
            report.message
        );
    }

    #[test]
    fn std_module_arity_failures_are_package_arity() {
        use crate::domain_native::{
            DocumentPageOp, GraphicsShapesOp, JapaneseLinebreakOp, MathAtomsOp,
        };
        use crate::DomainNativeOp;
        assert_arity(DomainNativeOp::TestPing, &[crate::RuntimeValue::Int(1)], 0);
        assert_arity(
            DomainNativeOp::GraphicsShapes(GraphicsShapesOp::Circle),
            &[],
            3,
        );
        assert_arity(DomainNativeOp::MathAtoms(MathAtomsOp::Ord), &[], 1);
        assert_arity(
            DomainNativeOp::JapaneseLinebreak(JapaneseLinebreakOp::ClassifySample),
            &[],
            1,
        );
        assert_arity(
            DomainNativeOp::DocumentPage(DocumentPageOp::Heading),
            &[],
            2,
        );
    }

    fn assert_type(op: crate::DomainNativeOp, args: &[crate::RuntimeValue]) {
        let err = crate::call_domain_native(op, args).expect_err("type");
        let report = err.failure_report().expect("structured");
        assert_eq!(report.code.as_path(), "package/type", "{op:?}: {report:?}");
    }

    #[test]
    fn std_module_type_failures_are_package_type() {
        use crate::domain_native::{
            ColorSrgbOp, JapaneseClassesOp, JapaneseKihonOp, JapaneseLinebreakOp,
        };
        use crate::DomainNativeOp;
        let s = crate::RuntimeValue::String("x".into());
        assert_type(
            DomainNativeOp::ColorSrgb(ColorSrgbOp::FromByte),
            &[s.clone(), s.clone(), s.clone()],
        );
        assert_type(
            DomainNativeOp::ColorSrgb(ColorSrgbOp::WithAlpha),
            &[s.clone(), crate::RuntimeValue::Int(1)],
        );
        assert_type(
            DomainNativeOp::JapaneseLinebreak(JapaneseLinebreakOp::ClassifySample),
            &[crate::RuntimeValue::Int(1)],
        );
        assert_type(
            DomainNativeOp::JapaneseLinebreak(JapaneseLinebreakOp::ClassifySample),
            &[crate::RuntimeValue::String(String::new())],
        );
        assert_type(
            DomainNativeOp::JapaneseClasses(JapaneseClassesOp::ClassName),
            std::slice::from_ref(&s),
        );
        assert_type(
            DomainNativeOp::JapaneseKihon(JapaneseKihonOp::LineMetrics),
            &[s.clone(), s],
        );
    }
}
