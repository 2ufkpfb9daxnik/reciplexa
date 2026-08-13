//! Host helper: fontless [`MathBox`] from a package math entry `main`.
//!
//! Graphics pages embedding math-box records as children are unlikely; hosts
//! should call this (or `estimate_math_box_from_value`) on math package trees.
//! See `lang/host-layout-consume-plan.md` (HC4).

use std::path::Path;

use reciplexa_eval::{
    estimate_math_box_from_value_with_style, eval_expr, primitive_env, RuntimeValue, UnitHost,
};
use reciplexa_std::math::{EstimateStyle, MathBox};

use crate::load::{elaborate_with_packages, LocalPackageIndex, PackageLoadError};

/// Errors from package load/eval or math box estimate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MathBridgeError {
    Load(String),
    Eval(String),
    Estimate(String),
}

impl From<PackageLoadError> for MathBridgeError {
    fn from(e: PackageLoadError) -> Self {
        Self::Load(e.to_string())
    }
}

impl std::fmt::Display for MathBridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(m) | Self::Eval(m) | Self::Estimate(m) => write!(f, "{m}"),
        }
    }
}

/// Elaborate/eval `entry_path` unit `main`, then estimate a fontless [`MathBox`].
///
/// Host package mains use [`EstimateStyle::Display`] by default (display math),
/// ignoring any nested `style` field on the tree — inline text style is for
/// language `math-box` / record surfaces, not host inspect estimates.
///
/// When `main` is a `math-demo`-style record with a `tree` field, estimates that
/// subtree; otherwise estimates the whole value if it is a math tagged record.
pub fn estimate_package_math_main(
    entry_path: impl AsRef<Path>,
    index: &LocalPackageIndex,
) -> Result<MathBox, MathBridgeError> {
    let entry = entry_path.as_ref();
    let units = elaborate_with_packages(entry, index)?;
    let stem = entry
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");
    let demo = units
        .iter()
        .find(|u| u.name == stem)
        .ok_or_else(|| MathBridgeError::Load(format!("missing elaborated unit `{stem}`")))?;
    let v = eval_expr(&demo.expr, &primitive_env(), &mut UnitHost)
        .map_err(|e| MathBridgeError::Eval(e.message))?;
    let target = math_estimate_target(&v);
    estimate_math_box_from_value_with_style(target, EstimateStyle::Display)
        .map_err(|e| MathBridgeError::Estimate(e.message))
}

fn math_estimate_target(v: &RuntimeValue) -> &RuntimeValue {
    match v {
        RuntimeValue::Record(fields) => fields
            .iter()
            .find(|(k, _)| k == "tree")
            .map(|(_, tree)| tree)
            .unwrap_or(v),
        _ => v,
    }
}
