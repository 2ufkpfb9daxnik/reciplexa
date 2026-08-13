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
| `layout_math_to_shapes` from `linearize` + `estimate_box` width (monospace heuristic); Scripts / BigOp / Matrix / Aligned / Stack / Accent / Delimiter placement stubs; Fraction / Radical Line rules | Full OpenType MATH / stretchy |
| Scripts / BigOp / Fraction / Radical / Delimiter / Matrix / Accent / Aligned / Stack placement stubs (offsets + Line rules) | Glyph metrics / TeX `\fontdimen` fidelity |
| Package page mixing doc paragraph + math sibling (or math-only graphics text fallback) | Math embedded as first-class `doc-block` kind |
| PDF / SVG / PPTX smoke for the mixed / math page | Guaranteed CJK embedding without system fonts |
| Docs pointing hosts at this plan | Closing full OPEN-TEXT-JA-001 |

## Gate

After units that touch code:

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-eval --offline --lib document_value
cargo test -p reciplexa-package --offline --test graphics_bridge_coverage
cargo test -p reciplexa-std --offline --test math_slide_vector_tests layout_math_
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

## Follow-ons (LL6–LL12)

### LL6 — SVG + PPTX smoke for `pkg_live_layout`

- Mirror HC9 / HC11: live-layout demo → `document_to_svg` / `document_to_pptx`.
- Assert multiple `<text` / `<a:t>` (doc paragraph + math glyphs).
- **Commit.**

### LL7 — Scripts placement in `layout_math_to_shapes`

- For `MathAtom::Scripts`, place base at `origin`; position sub/sup with
  `scripts_attachment_offsets` (math y-up → scene y-down).
- Tip test: sub below / sup above base.
- **Commit.**

### LL8 — BigOp limits placement

- For `MathAtom::BigOp`, place operator at `origin`; lower/upper via
  `bigop_limit_offsets`; body to the right of the op (display stub).
- Tip test: lower below / upper above / body rightward.
- **Commit.**

### LL9 — Fraction rule as Line

- For `MathAtom::Fraction`, stack num/den with `fraction_rule_metrics` clearances;
  draw the vinculum as a scene `Line` between them (not `a/b` linearize).
- Tip test: Line present; num above / den below.
- **Commit.**

### LL10 — Docs + tip tests

- Extend this plan status for LL6–LL9; refresh `implemented-features.md`.
- Keep / tip `layout_math_*` tests in `math_slide_vector_tests` + package smokes.
- **Commit.**

### LL11 — Radical vinculum as Line

- For `MathAtom::Radical`, place radicand (+ optional index via
  `radical_vinculum_index_offsets`); draw over-bar as scene `Line`.
- Tip test: Line above radicand.
- **Commit.**

### LL12 — Docs after radical

- Mark LL11–LL12 done; refresh next-steps if smooth.
- **Commit.**

---

## Follow-ons (LL13–LL20)

### LL13 — Delimiter stretchy fence glyphs

- For `MathAtom::Delimiter`, place body; draw left/right fence chars as
  scene `Text` with taller `size_mm` from the stretch height/depth heuristic
  (`stretch_factor` × body extent).
- Tip test: fence `size_mm` matches heuristic; body between fences.
- **Commit.**

### LL14 — Matrix cell placement

- For `MathAtom::Matrix`, place cells via `matrix_column_widths` /
  `aligned_column_x` / `matrix_cell_x_in_column` (center; cases left-align
  when delimited).
- Tip test: column-1 right of column-0; row-1 below row-0.
- **Commit.**

### LL15 — Accent mark via clearance

- For `MathAtom::Accent`, place base; draw accent mark glyph using
  `accent_clearance_em` / `accent_attachment_offset`.
- Tip test: hat above base; underline below.
- **Commit.**

### LL16 — Example `pkg_live_math.rpx`

- Live-layout demo with nested delimiter + fraction + scripts.
- Package bridge tip: fences, frac Line, subscript present.
- **Commit.**

### LL17 — PDF smoke + docs

- PDF smoke for `pkg_live_math` (CJK-gated `Tj`).
- Extend this plan status for LL13–LL17; refresh `implemented-features.md`.
- **Commit.**

### LL18 — Aligned column snap

- For `MathAtom::Aligned`, place cells snapped to `aligned_column_x` bands.
- Tip test: column x matches helper.
- **Commit.**

### LL19 — Stack / stackrel placement

- For `MathAtom::Stack`, place children via `stackrel_spacing_offsets` (or
  vertical gap stub for plain stack/atop).
- Tip test: upper above / lower below shared baseline.
- **Commit.**

### LL20 — Docs after aligned/stack

- Mark LL18–LL20 done; refresh next-steps.
- **Commit.**

---

## Follow-ons (LL21–LL25)

### LL21 — Cases: draw left brace + row placement

- For delimited / `math-cases` grids, draw the left `{` (and optional right
  fence) as scene `Text` sized by `cases_brace_total_height_em`; place rows
  inside that brace vertical extent (left-align cells).
- Tip test: brace `size_mm` matches heuristic; row-1 below row-0; cells right
  of brace.
- **Commit.**

### LL22 — Underbrace / overset spacing visual

- Wire live layout so underline / underbrace clearance uses
  `underbrace_spacing`, and labeled overset/underbrace (Stackrel) keeps
  `stackrel_spacing_offsets` placement visible in tip tests.
- Tip test when applicable; skip new machinery if already covered.
- **Commit.**

### LL23 — SVG + PPTX smoke for `pkg_live_math`

- Mirror LL6 for `pkg_live_math.rpx` → `document_to_svg` / `document_to_pptx`.
- Assert multiple `<text` / `<a:t>`.
- **Commit.**

### LL24 — GUI: open live-layout demos via package path

- Ensure `pkg_live_layout` / `pkg_live_math` route through
  `wants_package_graphics_path` and produce a scene via the package bridge
  (dispatch `live-layout-demo` from `document_from_package_entry`).
- Tip test `wants_package` + package ingest shapes.
- **Commit.**

### LL25 — Docs: heuristic engine vs OpenType MATH / JLReq

- Summarize live-layout status honestly vs full OpenType MATH / JLReq.
- Mark LL21–LL25 done; refresh `implemented-features.md`.
- **Commit.**

---

## Status

| Unit | Status |
|------|--------|
| LL0 — Plan doc | **done** |
| LL1 — `layout_doc_page_to_scene` | **done** |
| LL2 — `layout_math_to_shapes` | **done** (naive linearize glyphs; monospace `estimate_box` width — not OpenType MATH) |
| LL3 — Integration example + test | **done** (`examples/pkg_live_layout.rpx` + `document_from_live_layout_*`) |
| LL4 — PDF smoke | **done** (CJK-gated `Tj`) |
| LL5 — Docs / implemented-features | **done** |
| LL6 — SVG + PPTX smoke | **done** (multi `<text` / `<a:t>`) |
| LL7 — Scripts placement | **done** (`scripts_attachment_offsets`) |
| LL8 — BigOp limits | **done** (`bigop_limit_offsets`) |
| LL9 — Fraction Line rule | **done** |
| LL10 — Docs + tip tests | **done** |
| LL11 — Radical vinculum Line | **done** |
| LL12 — Docs after radical | **done** |
| LL13 — Delimiter tall fences | **done** |
| LL14 — Matrix column widths | **done** |
| LL15 — Accent clearance glyph | **done** |
| LL16 — `pkg_live_math.rpx` | **done** |
| LL17 — PDF smoke + docs | **done** |
| LL18 — Aligned column snap | **done** |
| LL19 — Stack placement | **done** |
| LL20 — Docs after aligned/stack | **done** |
| LL21 — Cases left brace + rows | **done** |
| LL22 — Underbrace/overset visual | **done** |
| LL23 — `pkg_live_math` SVG/PPTX | pending |
| LL24 — GUI package path open | pending |
| LL25 — Heuristic vs OpenType/JLReq docs | pending |
