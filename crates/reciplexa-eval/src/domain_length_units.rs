//! Direct Native v2 runtime for `length/units` (DN2-1).

use crate::domain_native::DomainNativeOp;
use crate::domain_native_failure::{
    expect_number, expect_record_field, take0, take1, take2, type_error,
};
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
        DomainNativeOp::LengthUnitsMm => unit_ctor("length/units/mm", "mm", args),
        DomainNativeOp::LengthUnitsCm => unit_ctor("length/units/cm", "cm", args),
        DomainNativeOp::LengthUnitsPt => unit_ctor("length/units/pt", "pt", args),
        DomainNativeOp::LengthUnitsBp => unit_ctor("length/units/bp", "bp", args),
        DomainNativeOp::LengthUnitsInch => unit_ctor("length/units/inch", "inch", args),
        DomainNativeOp::LengthUnitsQ => unit_ctor("length/units/q", "q", args),
        DomainNativeOp::LengthUnitsPx => unit_ctor("length/units/px", "px", args),
        DomainNativeOp::LengthUnitsEm => unit_ctor("length/units/em", "em", args),
        DomainNativeOp::LengthUnitsZero => {
            take0(args, "length/units/zero")?;
            Ok(length_record("mm", 0.0))
        }
        DomainNativeOp::LengthUnitsToMm => {
            let [len] = take1(args, "length/units/to-mm")?;
            Ok(RuntimeValue::Number(to_mm(len, "length/units/to-mm")?))
        }
        DomainNativeOp::LengthUnitsFromMm => {
            let [value] = take1(args, "length/units/from-mm")?;
            Ok(length_record(
                "mm",
                expect_number(value, "length/units/from-mm")?,
            ))
        }
        DomainNativeOp::LengthUnitsAddMm => {
            let [a, b] = take2(args, "length/units/add-mm")?;
            let sum = to_mm(a, "length/units/add-mm")? + to_mm(b, "length/units/add-mm")?;
            Ok(length_record("mm", sum))
        }
        DomainNativeOp::LengthUnitsScaleLength => {
            let [len, factor] = take2(args, "length/units/scale-length")?;
            let unit = field_string(len, "unit", "length/units/scale-length")?;
            let value = field_number(len, "value", "length/units/scale-length")?
                * expect_number(factor, "length/units/scale-length")?;
            Ok(length_record(&unit, value))
        }
        other => Err(EvalError {
            message: format!("not a length/units op: {other:?}"),
        }),
    }
}

fn unit_ctor(export: &str, unit: &str, args: &[RuntimeValue]) -> Result<RuntimeValue, EvalError> {
    let [value] = take1(args, export)?;
    Ok(length_record(unit, expect_number(value, export)?))
}

pub fn length_record(unit: &str, value: f64) -> RuntimeValue {
    RuntimeValue::Record(vec![
        ("unit".into(), RuntimeValue::String(unit.into())),
        ("value".into(), RuntimeValue::Number(value)),
    ])
}

pub fn to_mm(len: &RuntimeValue, export: &str) -> Result<f64, EvalError> {
    let unit = field_string(len, "unit", export)?;
    let value = field_number(len, "value", export)?;
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

fn field_string(rec: &RuntimeValue, key: &str, export: &str) -> Result<String, EvalError> {
    match expect_record_field(rec, key, export)? {
        RuntimeValue::String(s) => Ok(s),
        other => Err(type_error(export, "string", other)),
    }
}

fn field_number(rec: &RuntimeValue, key: &str, export: &str) -> Result<f64, EvalError> {
    expect_number(&expect_record_field(rec, key, export)?, export)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::call_domain_native;
    use crate::domain_native_failure::{PACKAGE_FAILURE_ARITY, PACKAGE_FAILURE_TYPE};

    #[test]
    fn to_mm_converts_cm() {
        let len = length_record("cm", 2.0);
        assert!((to_mm(&len, "length/units/to-mm").unwrap() - 20.0).abs() < 1e-9);
    }

    #[test]
    fn mm_arity_is_package_failure() {
        let err = call_domain_native(DomainNativeOp::LengthUnitsMm, &[]).unwrap_err();
        let report = err.failure_report().expect("structured Failure");
        assert_eq!(report.code.namespace, "package");
        assert_eq!(report.code.code, PACKAGE_FAILURE_ARITY);
        assert!(report.message.contains("length/units/mm"));
    }

    #[test]
    fn mm_type_is_package_failure() {
        let err = call_domain_native(
            DomainNativeOp::LengthUnitsMm,
            &[RuntimeValue::String("nope".into())],
        )
        .unwrap_err();
        let report = err.failure_report().expect("structured Failure");
        assert_eq!(report.code.code, PACKAGE_FAILURE_TYPE);
        assert!(report.message.contains("length/units/mm"));
    }
}
