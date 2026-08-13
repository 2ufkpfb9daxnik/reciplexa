//! Materialize language `package-resource` records when a package root is known.
//!
//! Language `(resource …)` elaborates to a deferred tagged record
//! (`tag "package-resource"`, `path`, `note`). Hosts with a package root +
//! manifest call [`materialize_package_resource`] / [`resolve_resource_value`]
//! to obtain an absolute FS path via [`resolve_package_resource`].
//!
//! [`materialize_package_resources_in_tree`] walks an eval value tree and
//! fail-soft attaches `resolved-path` on listed resources (Wave 5 Y2).

use std::path::{Path, PathBuf};

use reciplexa_eval::RuntimeValue;

use crate::manifest::{resolve_package_resource, PackageManifest, ResourceResolveError};
use crate::rpxm::parse_rpxm;

/// Error when a runtime value cannot be materialized as a package resource path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceValueError {
    /// Value is not a `package-resource` record (or fields are wrong shape).
    NotPackageResource(String),
    /// Path failed [`resolve_package_resource`] validation / listing.
    Resolve(ResourceResolveError),
}

impl std::fmt::Display for ResourceValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotPackageResource(s) => write!(f, "{s}"),
            Self::Resolve(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ResourceValueError {}

impl From<ResourceResolveError> for ResourceValueError {
    fn from(e: ResourceResolveError) -> Self {
        Self::Resolve(e)
    }
}

/// Walk parents of `start` (file or dir) looking for `package.rpxm`.
pub fn find_enclosing_package_root(start: &Path) -> Option<PathBuf> {
    let mut cur = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        if cur.join("package.rpxm").is_file() {
            return Some(cur);
        }
        cur = cur.parent()?.to_path_buf();
    }
}

/// If `v` is a `package-resource` record, resolve its `path` under `package_root`.
///
/// Returns the absolute path on success. Does not rewrite `v`; callers keep the
/// deferred record for language identity and use this for host FS access.
pub fn materialize_package_resource(
    v: &RuntimeValue,
    package_root: &Path,
    manifest: &PackageManifest,
) -> Result<PathBuf, ResourceValueError> {
    let fields = match v {
        RuntimeValue::Record(fields) => fields.as_slice(),
        _ => {
            return Err(ResourceValueError::NotPackageResource(
                "expected package-resource record".into(),
            ));
        }
    };
    let tag = fields
        .iter()
        .find(|(k, _)| k == "tag")
        .and_then(|(_, v)| match v {
            RuntimeValue::String(s) => Some(s.as_str()),
            _ => None,
        });
    if tag != Some("package-resource") {
        return Err(ResourceValueError::NotPackageResource(format!(
            "expected tag \"package-resource\", got {tag:?}"
        )));
    }
    let rel = fields
        .iter()
        .find(|(k, _)| k == "path")
        .and_then(|(_, v)| match v {
            RuntimeValue::String(s) => Some(s.as_str()),
            _ => None,
        })
        .ok_or_else(|| {
            ResourceValueError::NotPackageResource(
                "package-resource record missing string `path`".into(),
            )
        })?;
    resolve_package_resource(package_root, manifest, rel).map_err(Into::into)
}

/// Alias for [`materialize_package_resource`] (host-facing name).
pub fn resolve_resource_value(
    v: &RuntimeValue,
    package_root: &Path,
    manifest: &PackageManifest,
) -> Result<PathBuf, ResourceValueError> {
    materialize_package_resource(v, package_root, manifest)
}

fn is_package_resource_record(fields: &[(String, RuntimeValue)]) -> bool {
    fields.iter().any(|(k, v)| {
        k == "tag" && matches!(v, RuntimeValue::String(s) if s == "package-resource")
    })
}

/// Clone `v`, attaching `resolved-path` on every listed `package-resource` record.
///
/// **Fail-soft:** unlisted / invalid paths leave the record unchanged (no field).
/// Closures, builtins, and cells are left as-is (not walked).
pub fn materialize_package_resources_in_tree(
    v: &RuntimeValue,
    package_root: &Path,
    manifest: &PackageManifest,
) -> RuntimeValue {
    match v {
        RuntimeValue::Record(fields) => {
            let mut out: Vec<(String, RuntimeValue)> = fields
                .iter()
                .map(|(k, child)| {
                    (
                        k.clone(),
                        materialize_package_resources_in_tree(child, package_root, manifest),
                    )
                })
                .collect();
            if is_package_resource_record(&out) {
                if let Ok(abs) = materialize_package_resource(v, package_root, manifest) {
                    out.retain(|(k, _)| k != "resolved-path");
                    out.push((
                        "resolved-path".into(),
                        RuntimeValue::String(abs.to_string_lossy().into_owned()),
                    ));
                }
            }
            RuntimeValue::Record(out)
        }
        RuntimeValue::Variant { tag, payload } => RuntimeValue::Variant {
            tag: tag.clone(),
            payload: payload
                .as_ref()
                .map(|p| Box::new(materialize_package_resources_in_tree(p, package_root, manifest))),
        },
        other => other.clone(),
    }
}

/// If `entry` sits under a package with a parseable `package.rpxm`, materialize
/// resources in `v`; otherwise return `v` unchanged.
pub fn maybe_materialize_package_resources_for_entry(
    v: &RuntimeValue,
    entry_path: &Path,
) -> RuntimeValue {
    let Some(root) = find_enclosing_package_root(entry_path) else {
        return v.clone();
    };
    let Ok(src) = std::fs::read_to_string(root.join("package.rpxm")) else {
        return v.clone();
    };
    let Ok(manifest) = parse_rpxm(&src) else {
        return v.clone();
    };
    materialize_package_resources_in_tree(v, &root, &manifest)
}

#[cfg(test)]
mod tip_tests {
    use super::*;
    use crate::rpxm::parse_rpxm;

    #[test]
    fn tip_display_and_non_string_path() {
        let err = ResourceValueError::NotPackageResource("x".into());
        assert_eq!(err.to_string(), "x");
        let resolve_err = ResourceValueError::Resolve(ResourceResolveError::NotListed {
            resource: "a".into(),
        });
        assert!(resolve_err.to_string().contains("not listed"));

        let m = parse_rpxm(
            r#"(package demo
  format-version 1
  version "1.0.0"
  (public-modules main)
  (resources "f.txt"))"#,
        )
        .unwrap();
        let bad_path = RuntimeValue::Record(vec![
            (
                "tag".into(),
                RuntimeValue::String("package-resource".into()),
            ),
            ("path".into(), RuntimeValue::Int(1)),
        ]);
        let err = materialize_package_resource(&bad_path, Path::new("."), &m).unwrap_err();
        assert!(matches!(err, ResourceValueError::NotPackageResource(_)));

        let bad_tag_ty = RuntimeValue::Record(vec![
            ("tag".into(), RuntimeValue::Int(0)),
            ("path".into(), RuntimeValue::String("f.txt".into())),
        ]);
        assert!(materialize_package_resource(&bad_tag_ty, Path::new("."), &m).is_err());
    }

    #[test]
    fn tip_tree_materialize_fail_soft() {
        let m = parse_rpxm(
            r#"(package demo
  format-version 1
  version "1.0.0"
  (public-modules main)
  (resources "ok.txt"))"#,
        )
        .unwrap();
        let root = Path::new(".");
        let listed = RuntimeValue::Record(vec![
            (
                "tag".into(),
                RuntimeValue::String("package-resource".into()),
            ),
            ("path".into(), RuntimeValue::String("ok.txt".into())),
        ]);
        let unlisted = RuntimeValue::Record(vec![
            (
                "tag".into(),
                RuntimeValue::String("package-resource".into()),
            ),
            ("path".into(), RuntimeValue::String("nope.txt".into())),
        ]);
        let tree = RuntimeValue::Record(vec![
            ("a".into(), listed),
            ("b".into(), unlisted),
            ("n".into(), RuntimeValue::Int(1)),
        ]);
        let out = materialize_package_resources_in_tree(&tree, root, &m);
        let RuntimeValue::Record(fields) = out else {
            panic!("record");
        };
        let a = fields.iter().find(|(k, _)| k == "a").map(|(_, v)| v).unwrap();
        let RuntimeValue::Record(af) = a else {
            panic!("a");
        };
        assert!(af.iter().any(|(k, v)| {
            k == "resolved-path" && matches!(v, RuntimeValue::String(s) if s.ends_with("ok.txt"))
        }));
        let b = fields.iter().find(|(k, _)| k == "b").map(|(_, v)| v).unwrap();
        let RuntimeValue::Record(bf) = b else {
            panic!("b");
        };
        assert!(!bf.iter().any(|(k, _)| k == "resolved-path"));
    }
}
