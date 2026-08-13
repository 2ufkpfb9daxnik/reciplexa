# Native domain packages plan (Rust implementation + RPX surface)

**Status:** active plan (supersedes “implement domain libs as `.rpx` bodies” in earlier Slice C notes).  
**Normative anchors:** `specification.md` §13.10 Compiler-native package / `KER-001` (~L17260+, ~L17300+, ~L26350+).

## Clarifications (answered)

### What llvm-cov / the `_*.py` helpers were measuring

- **llvm-cov measures Rust crates only** (`crates/**`). It does **not** measure `.rpx` package source coverage.
- Root `_*.py` / `_*.txt` files were **temporary agent helpers** (parse llvm-cov JSON, update Part II conformance markdown, dump miss clusters). They are **not** part of the product and are being deleted.
- RPX under `packages/*.rpx` was an interim portable body. Domain vocabulary still appears as **package import paths** (`graphics/shapes`, …); the **body** for hot packages must be **Rust native**.

### Spec (not wrong — we mis-prioritized)

Spec already requires:

1. Domain words (`circle`, layout, math, …) are **not** kernel syntax — they stay **package API** (`.rpi` + `import`).
2. Hot / standard packages **may** (and for visual / text / document / math / Japanese should) have a **Rust native implementation**.
3. Do **not** expose raw native as a privileged API; observers use the same typed package surface.
4. Native vs portable must share observable behavior (conformance). Portable fallback is OPEN-NATIVE-PKG-001 — for v1 we ship **native-only** for these std packages and keep thin `.rpi` + registry stubs.

Earlier `package-plan.md` Slice C wording that said “deepen `packages/*.rpx`” is **corrected**: deepen **Rust** (`reciplexa-std` + native module registry), keep `.rpi` / import paths.

## Target architecture (three layers)

```text
Package API (.rpi + import path)     ← RPX authors see this
        ↓
Native adapter (KER / eval binding)  ← maps BindingId → Rust fn
        ↓
Rust domain impl (reciplexa-std / scene / …)  ← real logic
```

`(import graphics/shapes only circle)` must resolve to a **native binding**, not elaborate `packages/graphics/src/shapes.rpx`.

## Inventory to migrate (current RPX bodies → Rust)

| Package | Modules (today) | Rust home (target) |
|---------|-----------------|--------------------|
| `graphics` | `shapes`, `page`, `color` | `reciplexa-std::visual` + `scene` + native registry |
| `length` | `units` | `reciplexa-std::core` (`Length`) |
| `color` | `srgb` | `reciplexa-std::core` (`Color`) — merge with `graphics/color` |
| `japanese` | `classes`, `linebreak`, `kihon`, `markup` | new `reciplexa-std::japanese` (or `reciplexa-text-ja`) |
| `math` | `atoms`, `scripts`, `frac`, `sqrt`, `delimiters`, `matrix`, `accents`, `bigops`, `cases`, `align`, `stack` | deepen `reciplexa-std::math` |

Document / page constructors stay package-shaped (`graphics/page` or later `document/*`) but **implemented in Rust**.

## Fine-grained work units (do one commit / gate green each)

### Phase N0 — registry seam (no behavior change yet)

| ID | Unit | Done when |
|----|------|-----------|
| N0.1 | Doc: this plan + correct `package-plan.md` / `implemented-features.md` | **done** |
| N0.2 | Delete root `_*.py` / `_*.txt` / `_*.json` agent junk | **done** (kept `_gate.ps1`) |
| N0.3 | `DomainNativeRegistry` stub in `reciplexa-package` | **done** |
| N0.4 | Bind hook: skip `.rpx` when registered; pure `native/test` | **done** |
| N0.5 | Gate: fmt / clippy / test package | **done** (package-scoped) |

### Phase N1 — length + color

| ID | Unit | Status |
|----|------|--------|
| N1.1–N1.3 | `length/units` native + retire `.rpx` + example | **done** |
| N1.4–N1.5 | `color/srgb` + `graphics/color` native; retire `.rpx` | **done** |

### Phase N3 — math

| ID | Unit | Status |
|----|------|--------|
| N3.1–N3.6 | All `math/*` modules native; retire `.rpx` | **done** |

### Phase N4 — japanese

| ID | Unit | Status |
|----|------|--------|
| N4.1–N4.5 | classes / linebreak / kihon / markup native | **done** |

### Phase N1 — length + color (smallest domain)

| ID | Unit |
|----|------|
| N1.1 | Native `length/units`: `mm`/`cm`/`pt`/`bp`/`inch` → record values (parity with current RPX) |
| N1.2 | Wire `(import length/units)`; retire `packages/length/src/units.rpx` body (keep `.rpi` + `package.rpxm`) |
| N1.3 | Example `pkg_length.rpx` still works via native |
| N1.4 | Native `color/srgb` + `graphics/color` shared Rust impl |
| N1.5 | Retire RPX bodies; examples green |
| N1.6 | llvm-cov tip for new native modules ≥99% regions |

### Phase N2 — graphics shapes / page (replace RPX + strangler)

| ID | Unit |
|----|------|
| N2.1 | Native `graphics/color` named + rgb/rgba |
| N2.2 | Native `graphics/page` paper sizes + `page` constructor |
| N2.3 | Native `graphics/shapes`: circle/rect/ellipse |
| N2.4 | line / polyline / polygon / ring / frame |
| N2.5 | text / text-box / image |
| N2.6 | group / translate / rotate / scale / opacity |
| N2.7 | fill / stroke / paint |
| N2.8 | Point document pipeline at native values (reuse / shrink `graphics_value` bridge) |
| N2.9 | Retire `packages/graphics/src/*.rpx`; keep interfaces |
| N2.10 | Migrate remaining interim CST examples only after GUI golden path uses native |
| N2.11 | Coverage tip graphics native + eval bridge |

### Phase N3 — math (Satysfi-shaped trees in Rust)

| ID | Unit |
|----|------|
| N3.1 | `math/atoms` + `operatorname` |
| N3.2 | `scripts` / `frac` / `sqrt` |
| N3.3 | `delimiters` / `cases` |
| N3.4 | `matrix` / `accents` / `bigops` |
| N3.5 | `align` / `stack` |
| N3.6 | Retire `packages/math/src/*.rpx`; example `pkg_math.rpx` |
| N3.7 | Coverage tip |

### Phase N4 — japanese / JLReq stubs in Rust

| ID | Unit |
|----|------|
| N4.1 | Character class tables (cl-01..cl-30 aliases) in Rust |
| N4.2 | Line-break pair rules + `break-between` |
| N4.3 | Kihon / vertical stubs |
| N4.4 | Markup helpers (ruby / notes / tategaki) as data constructors |
| N4.5 | Retire `packages/japanese/src/*.rpx`; examples |
| N4.6 | Coverage tip |

### Phase N5 — document surface (after graphics stable)

| ID | Unit | Status |
|----|------|--------|
| N5.1 | Map `reciplexa-std::document` constructors onto native package paths | **done** — `packages/document` + native `document/page` (`doc-*` tags); interim CST untouched |
| N5.1b | Lower `doc-*` package values to scene (text layout stub) | **done** — `document_from_doc_value` + graphics bridge accepts `doc-page`; `pkg_document.rpx` elaborates+evals via bridge |
| N5.2a | Pipeline auto-route `(import document` like graphics | **done** — `wants_package_graphics_path` treats `(import document` as package domain path; `pkg_document.rpx` via `document_from_source`; interim `black_circle` unchanged |
| N5.2b | Package twin for black_circle (keep interim CST golden) | **done** — `examples/pkg_black_circle.rpx`; parity vs interim paper + circle; `black_circle.rpx` untouched |
| N5.2c | GUI read-only layers from scene when CST pages absent | **done** — scene-backed `resolve_preview_layers`; package nudge soft-refuse |
| N5.2d | Markup expand → package native nodes | pending |
| N5.2e | Docs: keyword-table deletion blocked by GUI CST sync (out of N5 delete scope) | pending |
| N5.3 | Coverage tip for document surface / pipeline / markup | pending |

**N5.2 honesty:** Retiring interim `(page)/(circle)` keyword tables requires GUI CST sync v2 (rewrite package AST). N5 finishes the **dual path** (package document + markup native; interim retained for writable goldens).

### Phase N6 — workspace coverage to ~99%

| ID | Unit | Status |
|----|------|--------|
| N6.1 | Remeasure filtered llvm-cov (scoped 4-crate OK if workspace slow) | **done** — 7-crate BEST-ENTRY **99.01%** (25614/25870); 4-crate **98.96%**; **≥99% crossed** |
| N6.2 | Tip `reciplexa-core` elaborate/check residuals | **done** — elaborate live-hash **98.91%**; check **98.54%**; cast **98.98%**; unify **≥99%**; dead-arm deletes |
| N6.3 | Tip `reciplexa-eval` / bind / syntax / cli / pipeline | **done** — `cli` **≥99% (99.04%)**; `pipeline` **≥99% (99.38% or-live)**; `eval` **98.51%**; `load` **97.90%**; `module` **98.27%**; syntax lexer/parse/markup/string_lit **≥99%**; bridge live-hash **100%** |
| N6.4 | Update `coverage-status.md` / holdouts honestly | **done** — BEST-ENTRY + N6q 7-crate **99.01%**; OR-live thr 98.2% |

## Explicit non-goals (this migration)

- Registry distribution (OPEN-PKG-001)
- Full JLReq UCS matrices / math glyph layout engines
- Deleting package **API** (`.rpi`, import paths)
- Measuring “RPX line coverage”

## Commit / gate policy

- One unit ≈ one atomic commit (or two: impl + docs).
- After each unit: `cargo fmt` · `clippy -D warnings` · `test --workspace` · `check -p reciplexa-gui`.
- No push unless asked.
- Do **not** bulk-delete all `packages/*/src/*.rpx` until that package’s native registry entry is live and examples pass.

## First concrete unit after this doc

**Completed through N4:** length, color, graphics (color/page/shapes), math (11 modules), japanese (4 modules) are Rust-synthesized natives; portable `src/*.rpx` retired for those packages.

**N6 done:** 7-crate BEST-ENTRY **≥99%** (N6 q). Soft residuals remain on check/cast/eval/load/document (see `coverage-holdouts.md`).

**N5.1 done:** native `document/page` package scaffold (`doc-*` tagged records mirroring `reciplexa_std::document`). **N5.2a–c done:** pipeline document import, `pkg_black_circle` twin, GUI read-only scene layers. **Next:** N5.2d markup→package native; keyword tables retained.
