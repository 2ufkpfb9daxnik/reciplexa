//! Differential evaluation: Hybrid v1 vs Direct Native v2 on the same consumer.

use std::path::Path;

use reciplexa_eval::{eval_expr_with_extra, primitive_env, RuntimeValue, UnitHost};

use crate::domain_native::DomainNativeMode;
use crate::domain_native_env::build_domain_native_eval_env;
use crate::load::{
    elaborate_with_packages, eval_package_entry_main, LocalPackageIndex, PackageLoadError,
};

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

/// Evaluate `entry_path` on Hybrid and Direct Native paths for `module_paths`.
pub fn differential_eval_package_modules(
    entry_path: impl AsRef<Path>,
    base_index: &LocalPackageIndex,
    module_paths: &[&str],
) -> Result<(), DifferentialError> {
    let entry_path = entry_path.as_ref();
    let hybrid_v = eval_package_entry_main(entry_path, base_index)
        .map_err(|e| DifferentialError::HybridEval(e.to_string()))?;

    let mut dn2_native = base_index.native().clone();
    for path in module_paths {
        if !dn2_native.set_mode(path, DomainNativeMode::DirectNative) {
            return Err(DifferentialError::Load(PackageLoadError::NotFound(
                format!("differential: unknown native module `{path}`"),
            )));
        }
    }
    let dn2_index = base_index.clone().with_native(dn2_native);
    let dn2_v = eval_package_entry_main_dn2(entry_path, &dn2_index)
        .map_err(|e| DifferentialError::DirectNativeEval(e.to_string()))?;

    if !runtime_values_equivalent(&hybrid_v, &dn2_v) {
        return Err(DifferentialError::ValueMismatch {
            hybrid: format!("{hybrid_v:?}"),
            direct_native: format!("{dn2_v:?}"),
        });
    }
    Ok(())
}

/// Evaluate with DN2 slot bindings injected (Direct Native path).
pub fn eval_package_entry_main_dn2(
    entry_path: impl AsRef<Path>,
    index: &LocalPackageIndex,
) -> Result<RuntimeValue, PackageLoadError> {
    let entry_path = entry_path.as_ref();
    let units = elaborate_with_packages(entry_path, index)?;
    let stem = entry_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| PackageLoadError::NotFound("entry path has no stem".into()))?;
    let demo = units
        .iter()
        .find(|u| u.name == stem)
        .ok_or_else(|| PackageLoadError::NotFound(format!("missing elaborated unit `{stem}`")))?;
    let extra = build_domain_native_eval_env(index.native());
    let v =
        eval_expr_with_extra(&demo.expr, &primitive_env(), &extra, &mut UnitHost).map_err(|e| {
            PackageLoadError::Module(reciplexa_bind::ModuleError { message: e.message })
        })?;
    Ok(crate::resource_value::maybe_materialize_package_resources_for_entry(&v, entry_path))
}

fn runtime_values_equivalent(a: &RuntimeValue, b: &RuntimeValue) -> bool {
    match (a, b) {
        (RuntimeValue::Unit, RuntimeValue::Unit) => true,
        (RuntimeValue::Number(x), RuntimeValue::Number(y)) => x == y,
        (RuntimeValue::Int(x), RuntimeValue::Int(y)) => x == y,
        (RuntimeValue::F64(x), RuntimeValue::F64(y)) => x == y,
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
