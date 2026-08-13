# Live layout consume plan (LL)

Status tracking for **production-ish layout consume** of JA break / place / indent /
columns and fontless math linearize → scene shapes.

Builds on [`host-layout-consume-plan.md`](host-layout-consume-plan.md) (HC metrics /
PDF-SVG-PPTX smoke) and the Wave 4–29 stub layer in
[`ja-math-deepen-plan.md`](ja-math-deepen-plan.md).

Goal: shared layout helpers that **place** soft-wrapped document text and naive
math glyphs onto scene pages — still **heuristic fonts/em**, not JLReq /
OpenType MATH.

## Honest scope

| Deliver | Do **not** claim |
|---------|------------------|
| `layout_doc_page_to_scene` over `doc-page` (break_line + place_lines + indent + columns) | Production document pipeline / editable layout engine |
| Refactor `document_from_doc_value` → shared layout module | New markup / CST layout surface |
| `layout_math_to_shapes` from `linearize` + `estimate_box` width (monospace heuristic) | Glyph metrics, stretchy, scripts placement |
| Package page mixing doc paragraph + math sibling (or math-only graphics text fallback) | Math embedded as first-class `doc-block` kind |
| PDF smoke for the mixed / math page | Guaranteed CJK embedding without system fonts |
| Docs pointing hosts at this plan | Closing full OPEN-TEXT-JA-001 |

## Gate

After units that touch code:

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-eval --offline --lib document_value
cargo test -p reciplexa-package --offline --test graphics_bridge_coverage
```

Narrow further when a unit only touches std helpers or a single example.

No push unless asked. Fine-grained commits (one unit ≈ one commit).

---

## Units (LL0–LL5)

### LL0 — Plan doc

- This file: units LL0–LL5, honest scope, gate.
- **Commit.**

### LL1 — `layout_doc_page_to_scene`

- Export `layout_doc_page_to_scene(doc_page_value) -> Document` that lays out
  `doc-page` via std `break_line` + `place_lines_horizontal` + `indent_first_line`
  + `measure_columns` (same budgets as today's document bridge:
  `DOC_TEXT_MAX_EM` = 40).
- Refactor `document_from_doc_value` to call a shared layout module in
  `reciplexa-std` and/or `reciplexa-eval` for clarity (thin wrapper OK).
- Keep existing indent / columns / soft-wrap tip tests green.
- **Commit.**

### LL2 — `layout_math_to_shapes`

- `layout_math_to_shapes(math_value, origin) -> Vec<Shape>`: very naive.
- Text glyph(s) from `MathAtom::linearize`, placed at `origin`, sized / advanced
  by `estimate_box` width (monospace em heuristic).
- If weak, still ship and document honestly (not OpenType MATH).
- **Commit.**

### LL3 — Integration example + test

- Package page that mixes a doc paragraph and an embedded math-box record as
  **siblings** on one scene page (host compose OK).
- If sibling mix is too hard: math-only page via graphics `text` from
  `linearize`.
- Commit example + package (or eval) test asserting both (or math) text shapes.
- **Commit.**

### LL4 — PDF smoke for LL3

- Bridge LL3 scene → `document_to_pdf`.
- Assert ≥1 PDF text showing op (`Tj`) when a system/env CJK font is available;
  otherwise skip (same gate pattern as HC7).
- **Commit.**

### LL5 — Docs + implemented-features

- Mark LL0–LL5 status in the table below.
- Point `lang/implemented-features.md` next-steps at this plan.
- **Commit.**

---

## Status

| Unit | Status |
|------|--------|
| LL0 — Plan doc | **pending** |
| LL1 — `layout_doc_page_to_scene` | pending |
| LL2 — `layout_math_to_shapes` | pending |
| LL3 — Integration example + test | pending |
| LL4 — PDF smoke | pending |
| LL5 — Docs / implemented-features | pending |
