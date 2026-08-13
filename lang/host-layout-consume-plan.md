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
| HC8 — Docs status | **pending** |
