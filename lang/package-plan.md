# Package plan (PKG-001 + std domain libs)

Goal: land local-path packages and std domain libraries after Part II conformance, without waiting on `OPEN-PKG-001` registry distribution.

## Spec anchors

- `PKG-001` — `specification.md` §13.10 (~L15855+): `package.rpxm`, source/interface/resource roots, public modules, deps, lockfile, workspace, resources
- `MOD-001` — outer modules + `(import …)` (already done); package path resolution is the next link
- SYN — `circle` / `page` / colors / units are **not** language builtins; packages supply constructors
- `OPEN-PKG-001*` — registry protocol, signing, git/URL deps: **stub / defer**

## Current baseline

| Piece | Status |
|---|---|
| `crates/reciplexa-package` | Phase 10 skeleton: nested `(name)/(version)/(entry)/(dep)` `parse_rpxm`, JSON manifest, resolver/lockfile/build graph |
| `crates/reciplexa-bind` | `load_module_tree` sibling `.rpx` only; no package search path |
| Document surface | Interim `(page …) (circle …)` in lower/effect — **keep until migration** |
| `crates/reciplexa-std` | Rust typed façade (Length/Color/…); not RPX packages yet |

## Ordered work

### Slice A — plan + DD-001 manifest surface (this pass)

1. This plan file.
2. Extend `PackageManifest` / `parse_rpxm` toward DD-001:
   - `(package <name> format-version … version "…" …)` flat fields
   - `source-root` / `interface-root` / `resource-root`
   - `(public-modules …)`
   - optional `entry` / `(entry-points …)` for libraries
   - keep Phase 10 nested form working for existing tests
3. Std RPX packages on disk under `packages/`:
   - `graphics` — `shapes` (+ minimal `color` / page-size vals as needed by examples)
   - stubs or thin follow-ons: `length`, `color`, `math`, `japanese` / markup (as separate commits)
4. Wire local import: `(import graphics/shapes)` resolves via package search roots → `source-root` module file → existing `elaborate_units`.
5. One language example using package import for a shape constructor; do **not** break GUI interim `page`/`circle`.
6. Gate (fmt / clippy `-D warnings` / test workspace / check gui) after each commit batch.
7. Update `lang/part2-conformance.md` PKG-001 items from blanket `deferred` → `partial` / `ok` / `gap` where implemented; recompute stats JSON.

### Slice B — consumer packages + path deps

- [x] Consumer `package.rpxm` with `(dependencies (alias package name version path "…"))` (`examples/pkg_consumer`)
- [x] Alias → package instance mapping for import first segment (`LocalPackageIndex::aliases`)
- [x] `rpx.lock` write/read for path deps (no registry) (`Lockfile::from_consumer` / `write_rpx_lock`)
- [x] `packages/math` + `packages/japanese` stubs; `.rpi` stubs when `interface-root` set

### Slice C — std domain depth

- [x] `packages/graphics` depth (static): shapes (`circle`/`rect`/`ellipse`/`line`/`path`/`polyline`/`polygon`/`ring`/`frame`/`group`), `text`/`text-box`/`image`, transforms (`translate`/`rotate`/`scale`), `opacity`, `fill`/`stroke`/`paint`, page sizes (`a4`/`letter`/`a5`/`a3`/`legal`/`square`/`page-size`), color `rgb`/`rgba` + named — aligned to `reciplexa-std` visual / interim tags; examples `pkg_graphics_static` / `pkg_graphics_shapes` / `pkg_graphics_transform`; package-tree load tests in `reciplexa-package`
- `packages/length` — deepen beyond stub (`mm` / unit constructors; retire Number+Ident interim where safe)
- [x] `packages/length` — `mm`/`cm`/`pt`/`bp`/`inch`/`q`/`px`/`em` + `to-mm`/`add-mm`/`scale-length`; example `examples/pkg_length.rpx`
- `packages/color` — deepen beyond stub (`rgb` / named colors) *(graphics/color already hosts rgb/rgba; shared `packages/color` still thin)*
- [x] `packages/color` — `rgb`/`rgba`/`srgb` + named palette + `from-byte`/`with-alpha`; example `examples/pkg_color.rpx`
- [x] `packages/japanese` — JLReq layer deepened: extended `sample-pair-rules`, `numeric-before-close-prohibited?`, vertical-flow/stack/tategaki stubs **`vertical-text`/`place-vertical-text`/`tategaki-text` graphics bridge**, `markup-bridge`/`doc-with-markup` connecting to `examples/markup_ja.rpx`; examples **`pkg_japanese_vertical`** (vertical-text stubs); **not** full JLReq / OPEN-TEXT-JA-001
- [x] `packages/math` — SATySFi-inspired atoms/scripts/frac/sqrt/delimiters **+ matrix / accents / bigops / align / stack** as Core records; **`operatorname`/`mathrm`/`textop`**, **`matrix-env`/`bmatrix-env`**, **`cases-delim`/`left-cases`**; example `examples/pkg_math.rpx`

### Slice D — document migration (strangler)

- Package-defined shape/page constructors consumed by lower/eval
- Keep interim surface until GUI + examples migrate atomically *(GUI interim retained; package import path live for examples, now covering interim text/image/transform/opacity tags as package constructors)*
- Retire hard-coded `"circle"` / `"page"` keyword tables gradually
- **Graphics strangler progress:**
  1. [x] Map interim lower tags (`circle`/`rect`/`page`/`text`/…) 1:1 onto `graphics/*` package records (already tag-compatible).
  2. [x] Bridge: `reciplexa_eval::document_from_graphics_value` recognizes package `page`/`fill`/`circle`/`rgb` records (ShapeTag-aware); golden vs `lower_source` in package tests. CST keyword tables + GUI unchanged.
  3. [x] Migrate `examples/*.rpx` that still use interim `(page …)(circle …)` to `(import graphics/…)` — `shapes`, `transforms`, `image`, `paths`, **`text_line`**, **`letter_opacity`**, **`two_pages`**, **`japanese_page`**, **`effects`**, **`macros`**, **`decls_stub`** → language-only or package bridge; **interim kept**: `black_circle` (GUI golden + CST sync), `markup_ja` / `markup_doc` (SYN `@`-markup macro → interim expand; package mirrors: `pkg_markup_ja`, vertical stubs: `pkg_japanese_vertical`).
  4. [x] Flip GUI scene ingest to the same constructors; keep keyword parse as a thin compatibility adapter. *(auto-detect package-shaped `(import graphics` + `(val main` without top-level interim `(page` → package bridge in `document_from_source`; expanded source + effect-form strip before package elaboration; `RECIPLEXA_PACKAGE_GRAPHICS=0|1` override; GUI `pipeline_doc` inherits via shared pipeline; CST sync/edit still interim-only via `black_circle.rpx`.)*
  5. [ ] Delete interim keyword tables only after GUI + export golden paths stay green.
  6. [x] Grow bridge tags: `rect`/`ellipse`/transforms/`group`/stroke/paint/text/image/opacity (+ line/polyline/polygon/ring/frame).
  7. [x] Multipage package trees: `graphics/page.pages` + cons-list / `pages` record in `document_from_graphics_value`.
### Slice E — workspace / resources / OPEN stubs

- [x] `workspace.rpxm` stub parse (`parse_workspace_rpxm` / format-version + members)
- [ ] Shared lockfile wired to workspace member discovery (root `rpx.lock` helpers exist for path deps)
- [ ] Resource root + declared distributable resources
- Registry client: stub errors only (`OPEN-PKG-001`)

## Conventions (v1 local)

```text
packages/<pkg>/
  package.rpxm
  src/<module>.rpx          # module path = relative to source-root
  interface/<module>.rpi    # required later for public modules (may stub)
  resources/                # optional
```

- Package names: ASCII lowercase kebab-case
- Search roots: repo `packages/` (and later env / workspace members)
- Script without manifest may import std packages by **package name as first path segment** (`graphics/shapes`)
- Registry / multi-version / features: out of scope until Slice E

## Gate

```
CARGO_TARGET_DIR=d:\reciplexa\target TEMP/TMP=d:\reciplexa\.tmp
cargo fmt --all
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
cargo check --offline -p reciplexa-gui
```

## Done when (Slice A)

- [x] DD-001-ish manifests parse for std packages
- [x] `(import graphics/shapes)` loads from `packages/graphics`
- [x] Example elaborates/evaluates via package import
- [x] Interim document `page`/`circle` still green
- [x] PKG-001 conformance rows updated; stats recomputed

## Done when (Slice B)

- [x] Consumer path deps + alias import (`g/shapes` → graphics)
- [x] `rpx.lock` path sources round-trip
- [x] math / japanese package stubs + public-module `.rpi` stubs
