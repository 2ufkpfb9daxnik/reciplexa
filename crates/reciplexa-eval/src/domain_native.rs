//! Domain-native package callables (Direct Native v2).
//!
//! Separate from kernel [`crate::value::BuiltinOp`]: these implement standard
//! package exports keyed by `package/module/export`.

use crate::value::RuntimeValue;
use crate::EvalError;

/// Stable dispatch id for a typed package export callable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DomainNativeOp {
    /// DN2-0 harness: `(native/test ping)` → `42`.
    TestPing,
    /// DN2-1 — `length/units` unit constructors and helpers.
    LengthUnitsMm,
    LengthUnitsCm,
    LengthUnitsPt,
    LengthUnitsBp,
    LengthUnitsInch,
    LengthUnitsQ,
    LengthUnitsPx,
    LengthUnitsEm,
    LengthUnitsZero,
    LengthUnitsToMm,
    LengthUnitsFromMm,
    LengthUnitsAddMm,
    LengthUnitsScaleLength,
}

/// Qualified registry key: `package/module/export`.
pub fn qualified_export_key(module_path: &str, export_name: &str) -> String {
    format!("{module_path}/{export_name}")
}

/// Internal eval/typecheck slot for a native export binding (valid RPX kebab ident).
pub fn dn2_slot(module_path: &str, export_name: &str) -> String {
    format!("dn2slot-{}-{}", module_path.replace('/', "-"), export_name)
}

/// Apply a domain-native callable (arity-checked).
pub fn call_domain_native(
    op: DomainNativeOp,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, EvalError> {
    match op {
        DomainNativeOp::TestPing => {
            if !args.is_empty() {
                return Err(EvalError {
                    message: format!("`native/test ping` expects 0 args, got {}", args.len()),
                });
            }
            Ok(RuntimeValue::Int(42))
        }
        DomainNativeOp::LengthUnitsMm
        | DomainNativeOp::LengthUnitsCm
        | DomainNativeOp::LengthUnitsPt
        | DomainNativeOp::LengthUnitsBp
        | DomainNativeOp::LengthUnitsInch
        | DomainNativeOp::LengthUnitsQ
        | DomainNativeOp::LengthUnitsPx
        | DomainNativeOp::LengthUnitsEm
        | DomainNativeOp::LengthUnitsZero
        | DomainNativeOp::LengthUnitsToMm
        | DomainNativeOp::LengthUnitsFromMm
        | DomainNativeOp::LengthUnitsAddMm
        | DomainNativeOp::LengthUnitsScaleLength => {
            crate::domain_length_units::call_length_units(op, args)
        }
    }
}
