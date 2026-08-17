# Package plan (PKG-001 + Direct Native v2)

**Role:** [`active-roadmap.md`](active-roadmap.md) Step 7 item 1の詳細。local/offline sliceは完了し、残るnetwork registry・full-tree hash・Core resource effectは別OPEN。現状は [`implemented-features.md`](implemented-features.md)。

Goal: local-path package **API** (`package.rpxm`, `.rpi`, `import`) after Part II conformance, with **Direct Native v2** bodies for hot domain packages (graphics / length / color / japanese / math / document). Hybrid v1 reference RPX remains for differential tests. Portable fallback is OPEN: [`direct-native-v2-plan.md`](direct-native-v2-plan.md) / `OPEN-NATIVE-PKG-001`.

## Spec anchors

- `PKG-001` — `specification.md` §13.10: manifest, public modules, deps, lockfile, workspace, resources
- **Compiler-native package** (~L17260): keep package API; DN2 uses typed Rust callables; Hybrid v1 RPX is differential reference only
- `KER-001` (~L17300, ~L26350): domain vocabulary stays package-shaped; Rust implements as typed intrinsics with same observable meaning
- SYN — `circle` / `page` / colors / units are **not** language builtins; packages supply constructors
- `OPEN-PKG-001*` / `OPEN-NATIVE-PKG-001` — registry / ABI details: stub / defer where needed

## Current baseline

| Piece | Status |
|---|---|
| `crates/reciplexa-package` | Manifest / path dep / lock / workspace stub |
| `crates/reciplexa-bind` | Module tree + package search path |
| Document surface | **N5.2 dual-path done** + **GUI CST sync v2 S5b/S6a/S6b:** product `examples/` package-shaped; production keyword arms gated (`interim-surface`); pipeline always refuses bare `(page …)` |
| `crates/reciplexa-std` | Rust typed façade (`visual` / `text` / `document` / `math` / …) — **authoritative body target** |
| `packages/*/src/*.rpx` | **Retired for std natives; non-native packages still load disk `.rpx`** |
| `packages/*/interface/*.rpi` | Keep as public API surface |

## Ordered work

### Slice A–B — manifest + local import (done)

Local `package.rpxm`, path deps, aliases, `rpx.lock`, search roots — done.

### Slice C — std domain depth (**corrected: Rust native**)

~~Deepen as RPX source under `packages/*/src`~~ → **implement in Rust** (`reciplexa-std` + native module registry); keep `.rpi` + package names.

Tracking: Direct Native v2 is **complete** for std domain modules ([`direct-native-v2-plan.md`](direct-native-v2-plan.md)). Production load uses DN2 stubs + typed callables. Hybrid v1 RPX remains for differential tests only. Portable fallback remains `OPEN-NATIVE-PKG-001`.

### Slice D — document migration (strangler) — **N5.2 dual-path done**

- Package-defined constructors consumed by lower/eval (bridge live); pipeline auto-routes `(import graphics|document`
- Markup expand emits graphics package `page`/`text`/`line`/`image` (not interim keyword heads)
- GUI: package-shaped sources writable (nudge/size); markup expand soft-refuses; golden `black_circle.rpx` is package AST
- **Keyword-table holdout (S6b done):** production keyword arms gated behind `interim-surface` / `cfg(test)`; fixture at `crates/reciplexa-lower/tests/fixtures/interim_page.rpx`; see `implemented-features.md`
- Tracking: Direct Native v2 is in production for std packages; Hybrid v1 RPX is differential reference only ([`direct-native-v2-plan.md`](direct-native-v2-plan.md))

### Slice E — workspace / resources / OPEN stubs

**Done (E0–E5):** `(resources …)` path rules; workspace member discovery; shared root `rpx.lock`; member→root lock; prefer workspace members + DAG; `OPEN-PKG-001` registry stub + optional listed-resource FS existence.

**R0 (package API):** `resolve_package_resource(package_root, manifest, rel)` validates under `resource_root` + listed `resources` (host helper).

**R2 (language light):** `(resource "rel")` elaborates to a pure tagged record
`(tag "package-resource") (path …) (note "resolve at package load")` — typechecks/evals without package root; hosts may resolve `path` via R0 when a root is available.

**R3 (host materialize):** `materialize_package_resource` / `resolve_resource_value(v, package_root, manifest)` — when a evaluated `package-resource` record is in hand and the package root is known, resolve to an absolute `PathBuf` via R0. **done** (helper + tests). [`eval_package_entry_main`] auto-materializes on every package load/eval path (graphics/math/doc/live bridges).

**Done (Step 7 lock reproducibility):** path dependencies require `rpx.lock` at load (PKG007); `examples/pkg_consumer/rpx.lock` is the committed fixture.

**Done (Step 7 typed resource light):** materialize attaches `resource-id` (`{name}@{version}/{path}`), SHA-256 `content-hash`, and `effect: Resource`; replay mismatch → PKG008 (`diagnose_package_resource_replay`). Full resource effect typing / bracket separation remains OPEN (RSC-001).

**Done (Step 7 registry mirror):** offline `registry/{name}/{version}/` mirror (or `RPIX_REGISTRY_ROOT`) resolves `source registry` / `registry:{name}@{version}` lock entries; network fetch remains OPEN.

**Still OPEN / deferred:**
- Full `package-resource` typed handle in Core types / resource effects bracket
- Network registry protocol / remote index fetch
- Full-tree / registry artifact content hashes (CS0 uses `package.rpxm` SHA-256 only)

## Conventions (v1)

```text
packages/<pkg>/
  package.rpxm
  interface/<module>.rpi    # public API (required for public modules)
  src/<module>.rpx          # INTERIM only — removed when native registry covers the module
```

- Package names: ASCII lowercase kebab-case
- Import: `(import graphics/shapes)` — resolves to **native body** when registered, else interim `.rpx`
- Native implementation lives in Rust crates; never expose raw FFI as the authoring API

## Gate

```
CARGO_TARGET_DIR=d:\reciplexa\target TEMP/TMP=d:\reciplexa\.tmp
cargo fmt --all
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
cargo check --offline -p reciplexa-gui
```

## Done when (domain migration)

1. `graphics` / `length` / `color` / `math` / `japanese` bodies are Rust native.
2. `(import …)` still works; examples green.
3. Interim `packages/*/src/*.rpx` for those packages removed (interfaces remain).
4. Filtered Rust region coverage ~99% (Phase N6).
