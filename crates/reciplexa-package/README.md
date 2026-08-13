# reciplexa-package

Local `package.rpxm`, path dependencies, `rpx.lock`, and workspace discovery
(`PKG-001`). Domain bodies live in Rust (`reciplexa-std` + native registry);
this crate is the host loader / lock / diagnose surface.

## Lock checksums (OPEN stub)

A dedicated `examples/pkg_checksum_demo.rpx` is **not** shipped — the
contract is small enough to document here.

### What is hashed

[`content_checksum`](src/lockfile.rs) reads one file and returns:

| Result | Meaning |
|--------|---------|
| `stub-fnv1a64:` + 16 hex chars | FNV-1a 64-bit of the file bytes |
| `stub-error:…` | read failed (missing path, I/O) |

**OPEN:** this is **not** blake3 / sha256 and **not** registry artifact
integrity. Only `package.rpxm` is hashed (not the full package tree).

### Who writes `LockedPackage.checksum`

| Writer | Fills checksum? |
|--------|-----------------|
| `Lockfile::from_consumer_with_roots` / `LocalPackageIndex::lock_consumer` | **yes** — each `path:` dep, from `{root}/package.rpxm` (CS0) |
| `WorkspaceIndex::build_lock` / `write_lock` / `resolve_workspace_dependencies` | **yes** — each workspace member's `package.rpxm` |
| `Lockfile::from_consumer` (no roots) / `Lockfile::from_graph` | **no** (`None`; pre-CS0 compatible) |

### Who diagnoses (PKG006)

[`diagnose_lockfile_checksums`](src/build.rs) compares a lock entry to the
current `{resolve_root}/{path}/package.rpxm` **only when**:

1. `checksum` is `Some(…)` (missing checksums are skipped), and
2. `source` starts with `path:`.

Mismatch → diagnostic code **PKG006**. Workspace-source entries may *carry*
a stub checksum but are **not** compared here (no `path:` prefix).

Hosts that want a demo: write a path-dep `rpx.lock` via `lock_consumer`,
then call `diagnose_lockfile_checksums`. Tip tests live in
`tests/pkg_diagnose_tip_tests.rs`.
