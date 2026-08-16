//! Lockfile for reproducible package graphs.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::manifest::{DependencySpec, PackageManifest};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    /// `workspace`, `path:<rel>`, or later `registry:…` (OPEN-PKG-001).
    pub source: String,
    /// Direct dependency edges (formal package names), for path-dep graphs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    /// Optional content checksum of that package's `package.rpxm`.
    ///
    /// Path-dep writers fill via [`content_checksum`] (SHA-256 of
    /// `package.rpxm`, not the full tree). Workspace lock writers
    /// ([`crate::WorkspaceIndex::build_lock`] / [`crate::resolve_workspace_dependencies`])
    /// fill the same hash for each member. Diagnose mismatch for `path:` sources
    /// with [`crate::diagnose_lockfile_checksums`] (PKG006).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

/// Compute a lockfile content-checksum string for `path`.
///
/// SHA-256 of the file bytes, encoded as `sha256:` plus 64 lowercase hex
/// characters. This is cryptographic integrity of **one file** (typically
/// `package.rpxm`); it is not a full package-tree hash and not registry
/// artifact verification (OPEN-PKG-001). Read errors become `stub-error:…`.
pub fn content_checksum(path: impl AsRef<Path>) -> String {
    match fs::read(path.as_ref()) {
        Ok(bytes) => format!("sha256:{}", hex_encode(&sha256(&bytes))),
        Err(e) => format!("stub-error:{e}"),
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

/// FIPS 180-4 SHA-256 (no extra crate; workspace stays offline).
fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (data.len() as u64).saturating_mul(8);
    let mut padded = data.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks_exact(4).enumerate().take(16) {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lockfile {
    pub packages: Vec<LockedPackage>,
}

impl Lockfile {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn from_graph(manifests: &[PackageManifest]) -> Self {
        let packages = manifests
            .iter()
            .map(|m| LockedPackage {
                name: m.name.clone(),
                version: m.version.clone(),
                source: "workspace".into(),
                dependencies: m
                    .dependencies
                    .iter()
                    .map(|d| d.package.clone().unwrap_or_else(|| d.name.clone()))
                    .collect(),
                checksum: None,
            })
            .collect();
        Self { packages }
    }

    /// Slice B+: lock a consumer plus its path dependencies (no registry).
    ///
    /// Records `path:<rel>` sources and dependency edges; suitable as a
    /// root `rpx.lock` for a single-package consumer or workspace root.
    /// Does **not** fill checksums (no package roots). Prefer
    /// [`Self::from_consumer_with_roots`] when writing a lock that should
    /// carry path-dep `package.rpxm` stubs (CS0).
    pub fn from_consumer(
        consumer: &PackageManifest,
        deps: &[(&DependencySpec, &PackageManifest)],
    ) -> Self {
        let triples: Vec<(&DependencySpec, &PackageManifest, Option<&Path>)> =
            deps.iter().map(|(s, m)| (*s, *m, None)).collect();
        Self::from_consumer_with_roots(consumer, &triples)
    }

    /// Like [`Self::from_consumer`], and for each path dep whose package root
    /// is provided, fills `checksum` from [`content_checksum`] of
    /// `{root}/package.rpxm` (manifest file only — not the full tree).
    ///
    /// Always fills the stub when a root is given (no feature gate). Missing
    /// roots leave `checksum: None` (same as pre-CS0 writers).
    pub fn from_consumer_with_roots(
        consumer: &PackageManifest,
        deps: &[(&DependencySpec, &PackageManifest, Option<&Path>)],
    ) -> Self {
        let mut packages = vec![LockedPackage {
            name: consumer.name.clone(),
            version: consumer.version.clone(),
            source: "workspace".into(),
            dependencies: deps
                .iter()
                .map(|(_, m, _)| m.name.clone())
                .collect::<Vec<_>>(),
            checksum: None,
        }];
        for (spec, manifest, root) in deps {
            let source = match &spec.path {
                Some(p) => format!("path:{p}"),
                None => "workspace".into(),
            };
            let checksum = if spec.path.is_some() {
                root.map(|r| content_checksum(r.join("package.rpxm")))
            } else {
                None
            };
            packages.push(LockedPackage {
                name: manifest.name.clone(),
                version: manifest.version.clone(),
                source,
                dependencies: manifest
                    .dependencies
                    .iter()
                    .filter(|d| d.path.is_some())
                    .map(|d| d.package.clone().unwrap_or_else(|| d.name.clone()))
                    .collect(),
                checksum,
            });
        }
        packages.sort_by(|a, b| a.name.cmp(&b.name));
        Self { packages }
    }

    /// PKG §17.5: exact version of each locked path dep must satisfy the
    /// consumer manifest's version requirement (exact match / `*` only for now).
    pub fn is_consistent_with_consumer(&self, consumer: &PackageManifest) -> Result<(), String> {
        for dep in &consumer.dependencies {
            if dep.path.is_none() {
                continue;
            }
            let formal = dep.package.as_deref().unwrap_or(dep.name.as_str());
            let Some(locked) = self.packages.iter().find(|p| p.name == formal) else {
                return Err(format!(
                    "lockfile is not consistent with the package manifest: missing `{formal}`"
                ));
            };
            if !locked.source.starts_with("path:") {
                return Err(format!(
                    "lockfile is not consistent with the package manifest: `{formal}` expected path source"
                ));
            }
            if dep.version_req != "*" && dep.version_req != locked.version {
                return Err(format!(
                    "lockfile is not consistent with the package manifest: `{formal}` locked as `{}` but manifest wants `{}`",
                    locked.version, dep.version_req
                ));
            }
        }
        Ok(())
    }

    pub fn write_rpx_lock(&self, path: impl AsRef<Path>) -> Result<(), String> {
        // `LockedPackage` is a closed Serialize schema; serialization cannot fail.
        let json = self
            .to_json()
            .unwrap_or_else(|_| unreachable!("Lockfile serde"));
        fs::write(path.as_ref(), json).map_err(|e| format!("write lockfile: {e}"))
    }

    pub fn read_rpx_lock(path: impl AsRef<Path>) -> Result<Self, String> {
        let src = fs::read_to_string(path.as_ref()).map_err(|e| format!("read lockfile: {e}"))?;
        Self::from_json(&src).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod sha256_tests {
    use super::{hex_encode, sha256};

    #[test]
    fn sha256_empty_and_abc_match_fips_vectors() {
        assert_eq!(
            hex_encode(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex_encode(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
