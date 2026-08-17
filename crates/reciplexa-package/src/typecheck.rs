//! Package-entry Core typecheck (evaluate-before-bridge).
//!
//! Document surface `typecheck_source` does not understand `(import …)`. Package
//! ingest instead elaborates with the local index, then runs language-kernel
//! `infer_expr` on the entry unit.

use std::path::Path;

use reciplexa_core::{typecheck_core_expr_with_vars, CheckError, CoreType, DataEnv};

use crate::domain_native_env::build_domain_native_type_env;
use crate::load::{elaborate_with_packages, LocalPackageIndex, PackageLoadError};

/// Failures from package Core typecheck.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageTypecheckError {
    Load(PackageLoadError),
    Check(CheckError),
}

impl std::fmt::Display for PackageTypecheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(e) => write!(f, "{e}"),
            Self::Check(e) => {
                let start = e.range.start().get();
                let end = e.range.end().get();
                write!(f, "{} @{start}..{end}", e.message)
            }
        }
    }
}

impl From<PackageLoadError> for PackageTypecheckError {
    fn from(e: PackageLoadError) -> Self {
        Self::Load(e)
    }
}

impl From<CheckError> for PackageTypecheckError {
    fn from(e: CheckError) -> Self {
        Self::Check(e)
    }
}

/// Elaborate `entry_path` with `index`, then infer the entry unit's Core type.
pub fn typecheck_with_packages(
    entry_path: impl AsRef<Path>,
    index: &LocalPackageIndex,
) -> Result<CoreType, PackageTypecheckError> {
    let units = elaborate_with_packages(entry_path.as_ref(), index)?;
    let stem = entry_path
        .as_ref()
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");
    let demo = units
        .iter()
        .find(|u| u.name == stem)
        .ok_or_else(|| PackageLoadError::NotFound(format!("missing elaborated unit `{stem}`")))?;
    Ok(typecheck_core_expr_with_vars(
        &demo.expr,
        DataEnv::default(),
        &build_domain_native_type_env(index.native()),
    )?)
}

/// Write `source` to a temp entry and [`typecheck_with_packages`].
pub fn typecheck_package_source(
    source: &str,
    entry_stem: &str,
    index: &LocalPackageIndex,
) -> Result<CoreType, PackageTypecheckError> {
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-pkg-tc-{}-{entry_stem}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).map_err(|e| PackageLoadError::Io(e.to_string()))?;
    let entry = dir.join(format!("{entry_stem}.rpx"));
    std::fs::write(&entry, source).map_err(|e| PackageLoadError::Io(e.to_string()))?;
    typecheck_with_packages(&entry, index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load::LocalPackageIndex;
    use std::path::PathBuf;

    fn repo_packages() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages")
    }

    fn index() -> LocalPackageIndex {
        LocalPackageIndex::discover(&[repo_packages().as_path()]).expect("packages/")
    }

    const PKG_CIRCLE: &str = r#"
(import graphics/shapes only circle fill)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main (page a4 (fill (circle 105 148.5 40) black)))
"#;

    #[test]
    fn typecheck_package_source_happy_path() {
        typecheck_package_source(PKG_CIRCLE, "entry", &index()).expect("valid package typechecks");
    }

    #[test]
    fn typecheck_package_source_unknown_import() {
        let src = "(import no-such/pkg only x)\n(val main 1)\n";
        let err = typecheck_package_source(src, "entry", &index()).unwrap_err();
        match err {
            PackageTypecheckError::Load(_) => {}
            other => panic!("expected load error, got {other}"),
        }
    }
}
