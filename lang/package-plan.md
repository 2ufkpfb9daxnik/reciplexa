# Package plan (PKG-001 + native std domain)

Goal: local-path package **API** (`package.rpxm`, `.rpi`, `import`) after Part II conformance, with **Rust native bodies** for hot domain packages (graphics / length / color / japanese / math / document). Portable `.rpx` bodies under `packages/*/src/` are interim only and are being retired per `lang/native-domain-plan.md`.

## Spec anchors

- `PKG-001` — `specification.md` §13.10: manifest, public modules, deps, lockfile, workspace, resources
- **Compiler-native package** (~L17260): keep package API; replace body with Rust native
- `KER-001` (~L17300, ~L26350): domain vocabulary stays package-shaped; Rust implements as typed intrinsics with same observable meaning
- SYN — `circle` / `page` / colors / units are **not** language builtins; packages supply constructors
- `OPEN-PKG-001*` / `OPEN-NATIVE-PKG-001` — registry / ABI details: stub / defer where needed

## Current baseline

| Piece | Status |
|---|---|
| `crates/reciplexa-package` | Manifest / path dep / lock / workspace stub |
| `crates/reciplexa-bind` | Module tree + package search path |
| Document surface | **N5.2 dual-path done** + **GUI CST sync v2 S5b/S6a:** product `examples/` package-shaped; keyword fixture quarantined; `RECIPLEXA_REQUIRE_PACKAGE` gate; keyword tables retained until S6b |
| `crates/reciplexa-std` | Rust typed façade (`visual` / `text` / `document` / `math` / …) — **authoritative body target** |
| `packages/*/src/*.rpx` | **Interim portable bodies — migrate to native (see native-domain-plan)** |
| `packages/*/interface/*.rpi` | Keep as public API surface |

## Ordered work

### Slice A–B — manifest + local import (done)

Local `package.rpxm`, path deps, aliases, `rpx.lock`, search roots — done.

### Slice C — std domain depth (**corrected: Rust native**)

~~Deepen as RPX source under `packages/*/src`~~ → **implement in Rust** (`reciplexa-std` + native module registry); keep `.rpi` + package names.

Tracking: **`lang/native-domain-plan.md`** (units N0–N6).

Interim RPX depth that already exists (`graphics` / `length` / `color` / `math` / `japanese`) remains until each unit’s native replacement is wired and examples pass; then delete the `.rpx` body only.

### Slice D — document migration (strangler) — **N5.2 dual-path done**

- Package-defined constructors consumed by lower/eval (bridge live); pipeline auto-routes `(import graphics|document`
- Markup expand emits graphics package `page`/`text`/`line`/`image` (not interim keyword heads)
- GUI: package-shaped sources writable (nudge/size); markup expand soft-refuses; golden `black_circle.rpx` is package AST
- **Keyword-table holdout:** interim CST retained for crate fixtures / coverage until sync v2 **S6b**; S6a deprecate gate is `RECIPLEXA_REQUIRE_PACKAGE`; see `gui-cst-sync-v2-plan.md`
- Tracking: `lang/native-domain-plan.md` Phase N5

### Slice E — workspace / resources / OPEN stubs

Unchanged: workspace stub parse done; shared lock / resource root / registry deferred.

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
