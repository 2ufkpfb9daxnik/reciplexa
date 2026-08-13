# Host layout consume plan (HC)

Status tracking for **host consumption** of the JA/math stub layer (Waves 4–29
complete-enough in [`ja-math-deepen-plan.md`](ja-math-deepen-plan.md)).

Goal: thin package/GUI/CLI helpers that **read** break-line line counts, text-shape
previews, and fontless math box estimates — without claiming production JLReq /
OpenType MATH layout.

## Honest scope

| Deliver | Do **not** claim |
|---------|------------------|
| `preview_doc_text_metrics` over package document bridge | Full document pipeline layout / editable text metrics UI |
| Optional inspect-document note with line count | New `reciplexa eval` subcommand surface |
| Read-only GUI layer labels with text prefix | Writable CST sync for markup-generated text |
| `estimate_package_math_main` for package math trees | Math embedded as graphics-page children / live glyph boxes |
| Docs pointing hosts at this plan | Closing full OPEN-TEXT-JA-001 |

## Gate

After units that touch code:

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-package --offline
```

When GUI labels change: also `cargo test -p reciplexa-gui --lib --offline`.
When CLI note changes: also `cargo test -p reciplexa --offline` (narrow if slow).

No push unless asked. Fine-grained commits (one unit ≈ one commit).

---

## Units (HC0–HC5)

### HC0 — Plan doc

- This file: units HC0–HC5, honest scope, gate.
- **Commit.**

### HC1 — `preview_doc_text_metrics`

- Prefer a **library** helper over a new CLI subcommand.
- Add `reciplexa_package::preview_doc_text_metrics(source, index)` (or equivalent)
  that bridges a package document-shaped source and returns **line / text-shape
  counts** plus light **box estimates** (em widths / approx block height) for
  tests and future GUI.
- Uses existing package document → scene bridge + std `char_em_width` /
  `break_line` budget alignment (`DOC_TEXT_MAX_EM` = 40).
- **Commit.**

### HC2 — Wire metrics into tests (+ optional CLI)

- Package (or reciplexa) test asserting long JA `doc-paragraph` metrics:
  `text_shape_count` / line count > 1.
- Optional: one extra diagnostic line from `inspect-document` when package
  document path yields metrics — **skip if tangled**.
- **Commit.**

### HC3 — GUI read-only text label prefix

- When building read-only layer rows from `WorldShape::Text`, include a short
  content prefix in the label (same spirit as package CST `text "…"` labels).
- Commit if clear; otherwise skip and mark skipped here.
- **Commit or skip.**

### HC4 — `estimate_package_math_main`

- Graphics pages embedding math-box records as children are unlikely; do **not**
  force that bridge.
- Export `estimate_package_math_main(entry, index)` that elaborates/evals a
  package math entry `main` and returns `MathBox` via `math_value` (prefer
  `tree` field on math-demo records when present).
- **Commit.**

### HC5 — Docs + implemented-features

- Point `lang/implemented-features.md` next-steps at this plan.
- Mark HC0–HC5 status in the table below.
- **Commit.**

---

## Status

| Unit | Status |
|------|--------|
| HC0 — Plan doc | **done** |
| HC1 — `preview_doc_text_metrics` | **done** |
| HC2 — Tests (+ optional CLI) | **done** |
| HC3 — GUI read-only text prefix | **done** |
| HC4 — `estimate_package_math_main` | **done** |
| HC5 — Docs / implemented-features | **done** |

## Follow-on (HC6–HC8)

### HC6 — Plan: PDF smoke units

- Expand this section with HC6–HC8 gate notes (package bridge → `document_to_pdf` → multiple `Tj`).
- **Commit.**

### HC7 — Long JA paragraph PDF smoke test

- Package (or reciplexa) test: pkg_document-like source with long JA `paragraph` → bridge → PDF.
- Assert ≥2 PDF text showing operators (`Tj`) when a system/env CJK font is available; otherwise skip.
- **Commit.**

### HC8 — Docs status for HC6–HC8

- Mark HC6–HC8 done; refresh `implemented-features.md` next-steps line.
- **Commit.**

Gate (HC7):

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-package --offline --test graphics_bridge_coverage
```

| Unit | Status |
|------|--------|
| HC6 — Plan: PDF smoke units | **done** |
| HC7 — Long JA → PDF multiple `Tj` | **done** |
| HC8 — Docs status | **done** |

## Follow-on (HC9–HC10 + host PKG/math)

### HC9 — Long JA paragraph SVG smoke test

- Mirror HC7: package document long JA `paragraph` → bridge → `document_to_svg`.
- Assert ≥2 SVG `<text` elements (no CJK font gate — SVG embeds content strings).
- **Commit.**

### HC10 — GUI preview metrics in status / debug

- Prefer a status-bar or debug line from `preview_doc_text_metrics` when the
  live preview already has a clear chrome hook.
- **Skip** if the GUI only has page/zoom chrome without a dedicated status/
  debug strip (do not invent a new panel).
- **Commit or skip.**

### OPEN-PKG — deepen registry refusal

- Structured machine-facing code (`OPEN-PKG-001`) on
  [`WorkspaceError::RegistryUnavailable`] plus a checked-in workspace fixture
  that refuses `source registry` without network I/O.
- **Commit.**

### Inspect-document math main

- When `inspect-document` cannot build a document snapshot and the source looks
  math-ish (`import math/…`), call `estimate_package_math_main` and print a
  one-line fontless box estimate.
- **Commit.**

Gate (HC9 / OPEN-PKG):

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-package --offline --test graphics_bridge_coverage --test workspace_tests
```

Gate (inspect math):

```bat
cargo test -p reciplexa --offline --lib inspect_document
```

| Unit | Status |
|------|--------|
| HC9 — Long JA → SVG multiple `<text` | **done** |
| HC10 — GUI metrics status/debug | **skipped** (no dedicated status/debug strip; page/zoom chrome only) |
| OPEN-PKG — structured registry refusal | **done** (`OPEN_PKG_001_CODE` + fixture) |
| Inspect-document math main | **done** |

## Follow-on (HC11–HC15)

### HC11 — Long JA paragraph PPTX smoke test

- Mirror HC7/HC9: package document long JA `paragraph` → bridge → `document_to_pptx`.
- Assert ≥2 OOXML `<a:t>` runs via zip slide XML (no text-extraction API required).
- **Commit.**

### HC12 — Optional ruby / tate counts in preview metrics

- Extend `DocTextPreviewMetrics` with `ruby_count` / `tate_chu_yoko_count`.
- When previewing from package source, walk the eval value tree for
  `ruby-box` / `vertical-ruby-box` / `ja-ruby` and `ja-tate-chu-yoko` /
  `tate-chu-yoko` tags; scene-only metrics stay at zero.
- Host records may wrap a `page` field; preview extracts it for bridging.
- **Commit.**

### HC13 — Manifest diagnose for registry deps

- `diagnose_manifest` emits **PKG005** when a dependency has `source registry`
  (static OPEN-PKG-001 stub; no network). Complements resolve-time
  `RegistryUnavailable`.
- **Commit.**

### HC14 — Docs status for HC11–HC15

- Mark units in this table; refresh `implemented-features.md` next-steps.
- **Commit.**

### HC15 — Export PDF/SVG/PPTX from `pkg_ja_break`

- `examples/pkg_ja_break.rpx` is a language-only `ja-break-demo` record (line
  list), not a document page — export would not apply.
- Long JA multi-backend smoke already covered by HC7 / HC9 / HC11.
- **Skip.**

Gate (HC11–HC13):

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-package --offline --test graphics_bridge_coverage --test build_tests
```

| Unit | Status |
|------|--------|
| HC11 — Long JA → PPTX multiple `<a:t>` | **done** |
| HC12 — ruby/tate optional preview counts | **done** |
| HC13 — PKG005 registry dep diagnose | **done** |
| HC14 — Docs status | **done** |
| HC15 — `pkg_ja_break` export integration | **skipped** (not a document page; redundant with HC7/HC9/HC11) |

## Follow-on (HC16–HC21)

### HC16 — `debug_layout_summary` (inspect + GUI tip)

- Compact tooltip string from `preview_doc_text_metrics` (no status-bar chrome).
- Wire into `inspect-document` for package docs; GUI unit test only (HC10 still skipped).
- **Commit.**

### HC17 — `pkg_document_indent` PDF/SVG smoke

- Bridge indent example → PDF (≥1 `Tj`, CJK gated) / SVG (≥2 `<text`).
- **Commit.**

### HC18 — Workspace-level registry dep (OPEN-PKG fixture)

- Optional `(dependencies …)` on `workspace.rpxm`; resolve refuses `source registry`.
- Fixture moves registry dep to workspace level.
- **Commit.**

### HC19 — Display style default for `estimate_package_math_main`

- Host package main always uses `EstimateStyle::Display` (ignores nested `style`).
- **Commit.**

### HC20 — Tip coverage PKG004 / PKG005

- Dedicated tip tests for missing-resource + registry diagnose codes.
- **Commit.**

### HC21 — Vertical place PDF smoke

- `lines_to_vertical_text_shapes` → PDF ≥2 `Tj` when CJK font available.
- **Commit.**

Gate (HC16–HC21):

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-package --offline --test graphics_bridge_coverage --test workspace_tests --test math_bridge_tests --test pkg_diagnose_tip_tests
cargo test -p reciplexa-gui --offline --test layout_summary_tests
```

| Unit | Status |
|------|--------|
| HC16 — `debug_layout_summary` | **done** |
| HC17 — indent PDF/SVG smoke | **done** |
| HC18 — workspace-level registry dep | **done** |
| HC19 — math Display default | **done** |
| HC20 — PKG004/005 tip | **done** |
| HC21 — vertical place PDF smoke | **done** |
