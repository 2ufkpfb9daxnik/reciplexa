//! Direct Native v2 runtime for `length/units` (DN2-1).

use crate::domain_native::DomainNativeOp;
use crate::value::RuntimeValue;
use crate::EvalError;

const PT_TO_MM: f64 = 0.3527777778;
const INCH_TO_MM: f64 = 25.4;
const Q_TO_MM: f64 = 0.25;
const PX_TO_MM: f64 = 0.2645833333;

pub fn call_length_units(
    op: DomainNativeOp,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, EvalError> {
    match op {
        DomainNativeOp::LengthUnitsMm => unit_ctor("mm", args),
        DomainNativeOp::LengthUnitsCm => unit_ctor("cm", args),
        DomainNativeOp::LengthUnitsPt => unit_ctor("pt", args),
        DomainNativeOp::LengthUnitsBp => unit_ctor("bp", args),
        DomainNativeOp::LengthUnitsInch => unit_ctor("inch", args),
        DomainNativeOp::LengthUnitsQ => unit_ctor("q", args),
        DomainNativeOp::LengthUnitsPx => unit_ctor("px", args),
        DomainNativeOp::LengthUnitsEm => unit_ctor("em", args),
        DomainNativeOp::LengthUnitsZero => {
            if !args.is_empty() {
                return Err(EvalError {
                    message: format!("`length/units zero` expects 0 args, got {}", args.len()),
                });
            }
            Ok(length_record("mm", 0.0))
        }
        DomainNativeOp::LengthUnitsToMm => {
            let [len] = take1(args, "`length/units to-mm`")?;
            Ok(RuntimeValue::Number(to_mm(len)?))
        }
        DomainNativeOp::LengthUnitsFromMm => {
            let [value] = take1(args, "`length/units from-mm`")?;
            Ok(length_record("mm", as_f64(value, "`from-mm` value")?))
        }
        DomainNativeOp::LengthUnitsAddMm => {
            let [a, b] = take2(args, "`length/units add-mm`")?;
            let sum = to_mm(a)? + to_mm(b)?;
            Ok(length_record("mm", sum))
        }
        DomainNativeOp::LengthUnitsScaleLength => {
            let [len, factor] = take2(args, "`length/units scale-length`")?;
            let unit = field_string(len, "unit")?;
            let value = field_number(len, "value")? * as_f64(factor, "`scale-length` factor")?;
            Ok(length_record(&unit, value))
        }
        other => Err(EvalError {
            message: format!("not a length/units op: {other:?}"),
        }),
    }
}

fn unit_ctor(unit: &str, args: &[RuntimeValue]) -> Result<RuntimeValue, EvalError> {
    let [value] = take1(args, &format!("`length/units {unit}`"))?;
    Ok(length_record(unit, as_f64(value, "value")?))
}

pub fn length_record(unit: &str, value: f64) -> RuntimeValue {
    RuntimeValue::Record(vec![
        ("unit".into(), RuntimeValue::String(unit.into())),
        ("value".into(), RuntimeValue::Number(value)),
    ])
}

pub fn to_mm(len: &RuntimeValue) -> Result<f64, EvalError> {
    let unit = field_string(len, "unit")?;
    let value = field_number(len, "value")?;
    Ok(match unit.as_str() {
        "mm" => value,
        "cm" => value * 10.0,
        "pt" => value * PT_TO_MM,
        "bp" => value * PT_TO_MM,
        "inch" => value * INCH_TO_MM,
        "q" => value * Q_TO_MM,
        "px" => value * PX_TO_MM,
        _ => value,
    })
}

fn as_f64(v: &RuntimeValue, ctx: &str) -> Result<f64, EvalError> {
    match v {
        RuntimeValue::Number(n) | RuntimeValue::F64(n) => Ok(*n),
        RuntimeValue::Int(n) => Ok(*n as f64),
        other => Err(EvalError {
            message: format!("{ctx}: expected number, got {other}"),
        }),
    }
}

fn field_string(rec: &RuntimeValue, key: &str) -> Result<String, EvalError> {
    match rec {
        RuntimeValue::Record(fields) => fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| match v {
                RuntimeValue::String(s) => Ok(s.clone()),
                other => Err(EvalError {
                    message: format!("field `{key}`: expected string, got {other}"),
                }),
            })
            .transpose()?
            .ok_or_else(|| EvalError {
                message: format!("record missing field `{key}`"),
            }),
        other => Err(EvalError {
            message: format!("expected record, got {other}"),
        }),
    }
}

fn field_number(rec: &RuntimeValue, key: &str) -> Result<f64, EvalError> {
    match rec {
        RuntimeValue::Record(fields) => fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| as_f64(v, &format!("field `{key}`")))
            .transpose()?
            .ok_or_else(|| EvalError {
                message: format!("record missing field `{key}`"),
            }),
        other => Err(EvalError {
            message: format!("expected record, got {other}"),
        }),
    }
}

fn take1<'a>(args: &'a [RuntimeValue], ctx: &str) -> Result<[&'a RuntimeValue; 1], EvalError> {
    match args {
        [a] => Ok([a]),
        _ => Err(EvalError {
            message: format!("{ctx} expects 1 arg, got {}", args.len()),
        }),
    }
}

fn take2<'a>(args: &'a [RuntimeValue], ctx: &str) -> Result<[&'a RuntimeValue; 2], EvalError> {
    match args {
        [a, b] => Ok([a, b]),
        _ => Err(EvalError {
            message: format!("{ctx} expects 2 args, got {}", args.len()),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_mm_converts_cm() {
        let len = length_record("cm", 2.0);
        assert!((to_mm(&len).unwrap() - 20.0).abs() < 1e-9);
    }
}
