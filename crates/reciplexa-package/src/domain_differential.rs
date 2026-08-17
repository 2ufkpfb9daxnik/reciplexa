//! Differential evaluation: Hybrid v1 reference vs Direct Native v2 on the same consumer.

use std::path::Path;

use reciplexa_eval::RuntimeValue;

use crate::load::{eval_package_entry_main, LocalPackageIndex, PackageLoadError};

/// Failure from differential comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DifferentialError {
    Load(PackageLoadError),
    HybridEval(String),
    DirectNativeEval(String),
    ValueMismatch {
        hybrid: String,
        direct_native: String,
    },
}

impl std::fmt::Display for DifferentialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(e) => write!(f, "load: {e}"),
            Self::HybridEval(e) => write!(f, "hybrid eval: {e}"),
            Self::DirectNativeEval(e) => write!(f, "direct native eval: {e}"),
            Self::ValueMismatch {
                hybrid,
                direct_native,
            } => {
                write!(f, "value mismatch: hybrid={hybrid} dn2={direct_native}")
            }
        }
    }
}

impl From<PackageLoadError> for DifferentialError {
    fn from(e: PackageLoadError) -> Self {
        Self::Load(e)
    }
}

/// Evaluate `entry_path` on Hybrid reference and Direct Native paths for `module_paths`.
pub fn differential_eval_package_modules(
    entry_path: impl AsRef<Path>,
    base_index: &LocalPackageIndex,
    module_paths: &[&str],
) -> Result<(), DifferentialError> {
    let entry_path = entry_path.as_ref();
    let hybrid_index = base_index
        .with_hybrid_reference_bodies(module_paths)
        .map_err(|e| DifferentialError::Load(PackageLoadError::NotFound(e)))?;
    let hybrid_v = eval_package_entry_main(entry_path, &hybrid_index)
        .map_err(|e| DifferentialError::HybridEval(e.to_string()))?;

    let dn2_v = eval_package_entry_main(entry_path, base_index)
        .map_err(|e| DifferentialError::DirectNativeEval(e.to_string()))?;

    if !runtime_values_equivalent(&hybrid_v, &dn2_v) {
        return Err(DifferentialError::ValueMismatch {
            hybrid: format!("{hybrid_v:?}"),
            direct_native: format!("{dn2_v:?}"),
        });
    }
    Ok(())
}

/// Both Hybrid reference and Direct Native must fail (arity / type Failure).
pub fn differential_eval_both_fail(
    entry_path: impl AsRef<Path>,
    base_index: &LocalPackageIndex,
    module_paths: &[&str],
) -> Result<(), DifferentialError> {
    let entry_path = entry_path.as_ref();
    let hybrid_index = base_index
        .with_hybrid_reference_bodies(module_paths)
        .map_err(|e| DifferentialError::Load(PackageLoadError::NotFound(e)))?;
    let hybrid = eval_package_entry_main(entry_path, &hybrid_index);
    let dn2 = eval_package_entry_main(entry_path, base_index);
    match (hybrid, dn2) {
        (Err(_), Err(_)) => Ok(()),
        (Ok(v), Err(e)) => Err(DifferentialError::ValueMismatch {
            hybrid: format!("ok {v:?}"),
            direct_native: format!("err {e}"),
        }),
        (Err(e), Ok(v)) => Err(DifferentialError::ValueMismatch {
            hybrid: format!("err {e}"),
            direct_native: format!("ok {v:?}"),
        }),
        (Ok(a), Ok(b)) => Err(DifferentialError::ValueMismatch {
            hybrid: format!("{a:?}"),
            direct_native: format!("{b:?}"),
        }),
    }
}

/// Typecheck `entry_path` on Hybrid reference and Direct Native paths.
pub fn differential_typecheck_package_modules(
    entry_path: impl AsRef<Path>,
    base_index: &LocalPackageIndex,
    module_paths: &[&str],
) -> Result<(), DifferentialError> {
    use crate::typecheck::typecheck_with_packages;

    let entry_path = entry_path.as_ref();
    let hybrid_index = base_index
        .with_hybrid_reference_bodies(module_paths)
        .map_err(|e| DifferentialError::Load(PackageLoadError::NotFound(e)))?;
    let hybrid_ty = typecheck_with_packages(entry_path, &hybrid_index)
        .map_err(|e| DifferentialError::HybridEval(e.to_string()))?;
    let dn2_ty = typecheck_with_packages(entry_path, base_index)
        .map_err(|e| DifferentialError::DirectNativeEval(e.to_string()))?;
    if hybrid_ty != dn2_ty {
        return Err(DifferentialError::ValueMismatch {
            hybrid: format!("{hybrid_ty:?}"),
            direct_native: format!("{dn2_ty:?}"),
        });
    }
    Ok(())
}

/// Evaluate with DN2 slot bindings injected (alias of production [`eval_package_entry_main`]).
pub fn eval_package_entry_main_dn2(
    entry_path: impl AsRef<Path>,
    index: &LocalPackageIndex,
) -> Result<RuntimeValue, PackageLoadError> {
    eval_package_entry_main(entry_path, index)
}

fn runtime_values_equivalent(a: &RuntimeValue, b: &RuntimeValue) -> bool {
    if numeric_equivalent(a, b) {
        return true;
    }
    match (a, b) {
        (RuntimeValue::Unit, RuntimeValue::Unit) => true,
        (RuntimeValue::String(x), RuntimeValue::String(y)) => x == y,
        (RuntimeValue::Bool(x), RuntimeValue::Bool(y)) => x == y,
        (RuntimeValue::ShapeTag(x), RuntimeValue::ShapeTag(y)) => x == y,
        (RuntimeValue::Record(xs), RuntimeValue::Record(ys)) => {
            if xs.len() != ys.len() {
                return false;
            }
            xs.iter().all(|(k, vx)| {
                ys.iter()
                    .find(|(ky, _)| ky == k)
                    .is_some_and(|(_, vy)| runtime_values_equivalent(vx, vy))
            })
        }
        (RuntimeValue::Bytes(xs), RuntimeValue::Bytes(ys)) => xs == ys,
        (
            RuntimeValue::Variant {
                tag: t1,
                payload: p1,
            },
            RuntimeValue::Variant {
                tag: t2,
                payload: p2,
            },
        ) => {
            t1 == t2
                && match (p1, p2) {
                    (None, None) => true,
                    (Some(a), Some(b)) => runtime_values_equivalent(a, b),
                    _ => false,
                }
        }
        _ => false,
    }
}

fn numeric_equivalent(a: &RuntimeValue, b: &RuntimeValue) -> bool {
    match (as_f64(a), as_f64(b)) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

fn as_f64(v: &RuntimeValue) -> Option<f64> {
    match v {
        RuntimeValue::Number(n) | RuntimeValue::F64(n) => Some(*n),
        RuntimeValue::Int(n) => Some(*n as f64),
        _ => None,
    }
}
