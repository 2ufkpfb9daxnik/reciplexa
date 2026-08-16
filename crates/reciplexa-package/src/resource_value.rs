//! Materialize language `package-resource` records when a package root is known.
//!
//! Language `(resource …)` elaborates to a deferred tagged record
//! (`tag "package-resource"`, `path`, `note`). Hosts with a package root +
//! manifest call [`materialize_package_resource`] / [`resolve_resource_value`]
//! to obtain an absolute FS path via [`resolve_package_resource`].
//!
//! [`materialize_package_resources_in_tree`] walks an eval value tree and
//! fail-soft attaches `resource-id`, `content-hash`, `effect`, and
//! `resolved-path` on listed resources (PKG R3 + Step 7 typed resource).

use std::path::{Path, PathBuf};

use reciplexa_eval::RuntimeValue;

use crate::lockfile::content_checksum;
use crate::manifest::{
    normalize_resource_path, resolve_package_resource, PackageManifest, ResourceResolveError,
};

/// Tag on language `(resource …)` / deferred package-resource records.
pub const PACKAGE_RESOURCE_TAG: &str = "package-resource";

/// Host effect label for reading resource bytes (PKG §21.15 light surface).
pub const PACKAGE_RESOURCE_EFFECT: &str = "Resource";

/// Error when a runtime value cannot be materialized as a package resource path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceValueError {
    /// Value is not a `package-resource` record (or fields are wrong shape).
    NotPackageResource(String),
    /// Path failed [`resolve_package_resource`] validation / listing.
    Resolve(ResourceResolveError),
    /// Record carried a `content-hash` that does not match the file on disk.
    ContentHashMismatch {
        resource: String,
        expected: String,
        actual: String,
    },
}

impl std::fmt::Display for ResourceValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotPackageResource(s) => write!(f, "{s}"),
            Self::Resolve(e) => write!(f, "{e}"),
            Self::ContentHashMismatch {
                resource,
                expected,
                actual,
            } => write!(
                f,
                "package resource `{resource}` content-hash mismatch: record has `{expected}`, file has `{actual}`"
            ),
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

/// PKG §21.13 light identity: `{name}@{version}/{normalized path}`.
pub fn package_resource_id(
    manifest: &PackageManifest,
    rel: &str,
) -> Result<String, ResourceResolveError> {
    let normalized = normalize_resource_path(rel).map_err(ResourceResolveError::InvalidPath)?;
    Ok(format!(
        "{}@{}/{}",
        manifest.name, manifest.version, normalized
    ))
}

/// SHA-256 content hash of a materialized resource file (`sha256:…` or `stub-error:…`).
pub fn resource_file_content_hash(path: &Path) -> String {
    content_checksum(path)
}

/// Returns true when `v` is a deferred or enriched `package-resource` record.
pub fn is_package_resource_value(v: &RuntimeValue) -> bool {
    match v {
        RuntimeValue::Record(fields) => is_package_resource_record(fields),
        _ => false,
    }
}

fn is_package_resource_record(fields: &[(String, RuntimeValue)]) -> bool {
    fields.iter().any(|(k, v)| {
        k == "tag" && matches!(v, RuntimeValue::String(s) if s == PACKAGE_RESOURCE_TAG)
    })
}

fn record_string_field<'a>(fields: &'a [(String, RuntimeValue)], key: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(k, _)| k == key)
        .and_then(|(_, v)| match v {
            RuntimeValue::String(s) => Some(s.as_str()),
            _ => None,
        })
}

fn record_rel_path(fields: &[(String, RuntimeValue)]) -> Result<&str, ResourceValueError> {
    record_string_field(fields, "path").ok_or_else(|| {
        ResourceValueError::NotPackageResource(
            "package-resource record missing string `path`".into(),
        )
    })
}

fn check_replay_content_hash(
    fields: &[(String, RuntimeValue)],
    rel: &str,
    actual: &str,
) -> Result<(), ResourceValueError> {
    let Some(expected) = record_string_field(fields, "content-hash") else {
        return Ok(());
    };
    if expected == actual {
        return Ok(());
    }
    Err(ResourceValueError::ContentHashMismatch {
        resource: rel.to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
    })
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
    if record_string_field(fields, "tag") != Some(PACKAGE_RESOURCE_TAG) {
        return Err(ResourceValueError::NotPackageResource(format!(
            "expected tag \"{PACKAGE_RESOURCE_TAG}\", got {:?}",
            record_string_field(fields, "tag")
        )));
    }
    let rel = record_rel_path(fields)?;
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

/// Re-read the file at `resolved-path` and compare to an existing `content-hash`.
pub fn verify_package_resource_replay(v: &RuntimeValue) -> Result<(), ResourceValueError> {
    let RuntimeValue::Record(fields) = v else {
        return Err(ResourceValueError::NotPackageResource(
            "expected package-resource record".into(),
        ));
    };
    if !is_package_resource_record(fields) {
        return Err(ResourceValueError::NotPackageResource(
            "expected package-resource record".into(),
        ));
    }
    let rel = record_rel_path(fields)?;
    let Some(expected) = record_string_field(fields, "content-hash") else {
        return Ok(());
    };
    let Some(resolved) = record_string_field(fields, "resolved-path") else {
        return Ok(());
    };
    let actual = resource_file_content_hash(Path::new(resolved));
    if expected == actual {
        return Ok(());
    }
    Err(ResourceValueError::ContentHashMismatch {
        resource: rel.to_string(),
        expected: expected.to_string(),
        actual,
    })
}

fn enrich_package_resource_fields(
    out: &mut Vec<(String, RuntimeValue)>,
    package_root: &Path,
    manifest: &PackageManifest,
    abs: &Path,
    rel: &str,
) -> Result<(), ResourceValueError> {
    let hash = resource_file_content_hash(abs);
    check_replay_content_hash(out, rel, &hash)?;
    let id = package_resource_id(manifest, rel)?;
    for key in ["resolved-path", "resource-id", "content-hash", "effect"] {
        out.retain(|(k, _)| k != key);
    }
    out.push((
        "resolved-path".into(),
        RuntimeValue::String(abs.to_string_lossy().into_owned()),
    ));
    out.push(("resource-id".into(), RuntimeValue::String(id)));
    out.push(("content-hash".into(), RuntimeValue::String(hash)));
    out.push((
        "effect".into(),
        RuntimeValue::String(PACKAGE_RESOURCE_EFFECT.into()),
    ));
    let _ = package_root;
    Ok(())
}

/// Clone `v`, attaching typed host fields on every listed `package-resource` record.
///
/// **Fail-soft:** unlisted / invalid paths leave the record unchanged (no field).
/// Content-hash replay mismatch also leaves the record unchanged.
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
                    if let Ok(rel) = record_rel_path(&out).map(str::to_string) {
                        let _ = enrich_package_resource_fields(
                            &mut out,
                            package_root,
                            manifest,
                            &abs,
                            &rel,
                        );
                    }
                }
            }
            RuntimeValue::Record(out)
        }
        RuntimeValue::Variant { tag, payload } => RuntimeValue::Variant {
            tag: tag.clone(),
            payload: payload.as_ref().map(|p| {
                Box::new(materialize_package_resources_in_tree(
                    p,
                    package_root,
                    manifest,
                ))
            }),
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
    let Ok(manifest) = crate::rpxm::parse_rpxm(&src) else {
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
        let mismatch = ResourceValueError::ContentHashMismatch {
            resource: "f.txt".into(),
            expected: "sha256:aa".into(),
            actual: "sha256:bb".into(),
        };
        assert!(mismatch.to_string().contains("content-hash mismatch"));

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
                RuntimeValue::String(PACKAGE_RESOURCE_TAG.into()),
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
        let dir = std::env::temp_dir().join(format!(
            "reciplexa-pkg-mat-tree-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(dir.join("resources")).unwrap();
        std::fs::write(dir.join("resources/ok.txt"), b"ok").unwrap();
        let root = dir.as_path();
        let listed = RuntimeValue::Record(vec![
            (
                "tag".into(),
                RuntimeValue::String(PACKAGE_RESOURCE_TAG.into()),
            ),
            ("path".into(), RuntimeValue::String("ok.txt".into())),
        ]);
        let unlisted = RuntimeValue::Record(vec![
            (
                "tag".into(),
                RuntimeValue::String(PACKAGE_RESOURCE_TAG.into()),
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
        let a = fields
            .iter()
            .find(|(k, _)| k == "a")
            .map(|(_, v)| v)
            .unwrap();
        let RuntimeValue::Record(af) = a else {
            panic!("a");
        };
        assert!(af.iter().any(|(k, v)| {
            k == "resolved-path" && matches!(v, RuntimeValue::String(s) if s.ends_with("ok.txt"))
        }));
        assert!(af.iter().any(|(k, v)| k == "resource-id"
            && matches!(v, RuntimeValue::String(s) if s.contains("demo@1.0.0/ok.txt"))));
        assert!(af.iter().any(|(k, v)| k == "content-hash"
            && matches!(v, RuntimeValue::String(s) if s.starts_with("sha256:"))));
        assert!(af.iter().any(|(k, v)| k == "effect"
            && matches!(v, RuntimeValue::String(s) if s == PACKAGE_RESOURCE_EFFECT)));
        let b = fields
            .iter()
            .find(|(k, _)| k == "b")
            .map(|(_, v)| v)
            .unwrap();
        let RuntimeValue::Record(bf) = b else {
            panic!("b");
        };
        assert!(!bf.iter().any(|(k, _)| k == "resolved-path"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn replay_mismatch_reports_pkg008() {
        use crate::build::diagnose_package_resource_replay;

        let dir = std::env::temp_dir().join(format!(
            "reciplexa-pkg-rsc-replay-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(dir.join("resources")).unwrap();
        let abs = dir.join("resources/ok.txt");
        std::fs::write(&abs, b"v1").unwrap();
        let actual = resource_file_content_hash(&abs);
        let stale = RuntimeValue::Record(vec![
            (
                "tag".into(),
                RuntimeValue::String(PACKAGE_RESOURCE_TAG.into()),
            ),
            ("path".into(), RuntimeValue::String("ok.txt".into())),
            (
                "resolved-path".into(),
                RuntimeValue::String(abs.to_string_lossy().into_owned()),
            ),
            (
                "content-hash".into(),
                RuntimeValue::String(
                    "sha256:deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
                        .into(),
                ),
            ),
        ]);
        let err = verify_package_resource_replay(&stale).unwrap_err();
        assert!(matches!(
            err,
            ResourceValueError::ContentHashMismatch { .. }
        ));
        let diags = diagnose_package_resource_replay(&stale);
        let hit = diags.iter().find(|d| d.code == "PKG008").expect("PKG008");
        assert!(hit.message.contains("ok.txt"));
        assert_ne!(
            actual,
            "sha256:deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
