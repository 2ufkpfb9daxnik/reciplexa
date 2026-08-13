# OPEN-TEXT-JA-001 / math layout light — deepen plan

Status tracking for deepening `packages/japanese` + `packages/math` **beyond** the N3/N4 synthetic RPX natives, by putting **authoritative Rust tables and typed APIs** in `crates/reciplexa-std`.

## Honest scope

| Deliver | Do **not** claim |
|---------|------------------|
| JLReq-inspired **character class enum** + `classify_char` for a **useful subset** of punctuation / kana / ideographs / western | Full UCS membership tables for cl-01..cl-30 |
| Pair **break opportunity** stub (kinsoku-inspired forbidden starts/ends) | Normative JLReq appendix C pair matrix / streaming linebreak |
| Naive **`break_line` / `char_em_width`** (em budget + break_opportunity + optional hangable stub) | Full justification / CSS `line-break` / normative hanging punctuation |
| **`is_hangable`** (cl-06/07) used optionally by `break_line` | Full JLReq hanging / measure overhang model |
| Kihon-hanmen / writing-mode / ruby / tate-chu-yoko **data types** + **Wave 4 fontless box stubs** | Live line layout, glyph positioning, Document lower consumption |
| Deeper `MathAtom` tree + **fontless** box metric estimates + matrix/align/stack + **stretchy delimiter / accent clearance stubs** | TeX/SATySFi glyph layout, real OpenType MATH stretchy fences |

Package natives (`reciplexa-package::domain_bodies`) keep **synthetic RPX constructors** for the Core/eval import path. Std Rust is what future layout and hosts should call; comments in package sources document that relationship.

Normative anchors: `OPEN-TEXT-JA-001` in `specification.md`; W3C [JLReq](https://www.w3.org/TR/jlreq/); existing `packages/japanese` / `packages/math` READMEs.

## Gate

After each unit:

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-std --offline
```

If `domain_bodies` / package tests are touched: also `cargo test -p reciplexa-package --offline`.

No push unless asked. Fine-grained commits (one unit ≈ one commit).

---

## Japanese units (J0–J7)

### J0 — `reciplexa-std::japanese` module skeleton

- New `japanese` module exported from `reciplexa-std`.
- `CharClass` enum aligned with package `cl-01`..`cl-30` aliases (id + English code name).
- `classify_char(c: char) -> CharClass` covering a **practical subset** (common brackets/stops/commas, small kana, hiragana/katakana ranges, CJK ideographs, ASCII alnum, spaces) — unknown → ideographic or western fallback documented in code.
- Unit tests for classification samples that mirror `japanese/linebreak` `classify-sample` glyphs where they overlap.
- **Commit.**

### J1 — linebreak pair rules stub

- `BreakOpportunity` (`Allowed` / `Prohibited` / `Inseparable`).
- `break_opportunity(prev, next) -> BreakOpportunity` using class-level kinsoku stubs (line-head / line-end prohibited classes, inseparable cl-08, simple western run).
- Keep package synthetic RPX; add a short source comment that **Rust `reciplexa_std::japanese` is authoritative** for future layout callers.
- Std unit tests for representative pairs (「あ」, `あ。`, `——`, western run, default allow).
- **Commit.**

### J2 — kihon / vertical stubs in std

- `WritingMode` (`HorizontalTb` / `VerticalRl`).
- `KihonHanmen` + size helpers (`hanmen_inline_em`, `hanmen_block_em`, `line_gap_em`) matching package formula stubs.
- Defaults for horizontal / vertical sample frames.
- **Commit.**

### J3 — ruby / tate-chu-yoko data types

- Types aligned with `japanese/markup` tags: `Ruby` (simple / jukugo), `TateChuYoko`, optional `TategakiParagraph` stub.
- Constructors + Display / tag helpers for host bridging later.
- **Commit.**

### J4 — docs + demo + implemented-features

- Module-level docs / README notes pointing at this plan.
- Integration test (or std demo test) exercising `classify_char` + `break_opportunity` on a short Japanese string.
- Update `lang/implemented-features.md` next-steps: first JA/math deepen slice **done**; remaining = full UCS / glyph layout.
- **Commit.**

### J5 — expand `classify_char` coverage

- Broader punctuation / kana / digit / Latin ranges in `reciplexa-std::japanese` (still a subset, not full UCS).
- Unit tests with at least one sample per expanded class / range.
- **Commit.**

### J6 — expand `break_opportunity` matrix

- More prohibited / inseparable pairs aligned with package `kinsoku-profile` sample strings and `sample-pair-rules`.
- Tests that every glyph in package line-head / line-end sample strings classifies into a prohibited class and forbids the matching break.
- **Commit.**

### J7 — package linebreak ↔ std parity bridge

- Public `reciplexa_package::check_linebreak_std_parity` (+ fixed sample table) used from package tests.
- Keep `classify-sample` API; note that authors should prefer a future intrinsic backed by Rust `classify_char`.
- **Commit.**

---

## Math units (M0–M3)

### M0 — deepen `MathAtom` tree

- Keep existing Symbol / Row / Fraction / Radical / Scripts / Delimiter.
- Add Accent + BigOp (limits) variants so std covers the SATySFi-shaped surface already exported by `packages/math`.
- Solidify constructors / `child_count` / `linearize` for scripts, frac, sqrt, delimiters.
- **Commit.**

### M1 — light box metrics stub

- `MathBox { width, height, depth }` in em-ish abstract units (no fonts).
- `MathAtom::estimate_box()` heuristics (symbol ≈ 1×1, frac stacks, scripts shrink factor, delimiter padding).
- Tests for relative sizes (frac taller than symbol; scripts wider than base).
- **Commit.**

### M2 — matrix / align / stack node types

- `MathAtom` variants or companion types: `Matrix`, `Aligned`, `Stack` (and thin helpers like `matrix_row`) aligned with package tags.
- Wire into `estimate_box` / `linearize` lightly.
- **Commit.**

### M3 — docs + tests; update plan

- Extend `math_slide_vector_tests` (or sibling) for new nodes + boxes.
- Update this plan status table + `packages/math` README note (std now has matrix/align/stack).
- Confirm `implemented-features.md` next-steps reflect completed first slice.
- **Commit.**

---

## Status

| Unit | Status |
|------|--------|
| Plan | **done** |
| J0 | **done** |
| J1 | **done** |
| J2 | **done** |
| J3 | **done** |
| J4 | **done** |
| J5 | **done** |
| J6 | **done** |
| J7 | **done** |
| M0 | **done** |
| M1 | **done** |
| M2 | **done** |
| M3 | **done** |

## Remains after this deepen (full OPEN-TEXT-JA-001)

- Complete UCS → class membership for all JLReq classes.
- Normative §C break pair matrix + hangable punctuation / justification.
- Font-backed shaping, ruby layout, tate-chu-yoko metrics, vertical glyph orientation.
- Math: OpenType MATH / stretchy fences / real matrix column alignment / linebreak in display math.
- Document pipeline + GUI consuming kihon / ruby / math boxes (not only trees).
  - Light prep **done**: `reciplexa_eval::math_value` lowers package math tagged records → `MathAtom` → `estimate_box`.
  - **Note:** CLI `inspect-document` is the graphics/document snapshot path; it does **not** print math `estimate_box`. Prefer eval/`math_value` on package math trees (see `pkg_math_main_tree_estimates_box_via_math_value` — **full** `pkg_math` demo tree estimates, including under/over/cases/operatorname).
- Wire `classify_char` / `break_opportunity` as language builtins.
  - **done**: ker builtins `classify-char` (→ Int class id) and `break-between` (→ `allowed`/`prohibited`/`inseparable` tags).
  - **done**: language-only example `examples/pkg_ja_classify.rpx` (+ eval test).

## Follow-on slices (post J*/M*)

| Slice | Status |
|-------|--------|
| Eval `math_value` bridge: symbol/row/frac/scripts/radical/delimiter | **done** |
| Eval `math_value`: accent / matrix / bigop / stack / aligned | **done** |
| Eval `math_value`: under / over / cases / operatorname (+ matrix-env / align-eq / substack) | **done** |
| JA `classify_char` ideographic punctuation + fullwidth expansions | **done** (still subset, not full UCS) |
| JA `classify_char` more CJK symbols / Unicode spaces / wave-dash family | **done** (still subset) |
| Language builtins `classify-char` / `break-between` | **done** |
| Example `pkg_ja_classify.rpx` (builtins demo) | **done** |
| Package `pkg_math` tree → `estimate_box` via `math_value` (not `inspect-document`) | **done** (full demo tree) |
| Package `materialize_package_resource` / `resolve_resource_value` when root known | **done** (helper + tests; full load-time rewrite deferred) |
| Tip tests for math_value / japanese / resource | **done** |
| **Wave 3** — `japanese::break_line` + `char_em_width` (naive em-width wrap via `break_opportunity` / kinsoku) | **done** |
| **Wave 3** — `break_line` demo test (short Japanese phrase + mixed ASCII) | **done** |
| **Wave 3** — `MathAtom::linearize` braces multi-char script/limit bodies | **done** |
| Language builtin `break-line` (ker + eval + light typecheck → cons/nil strings) | **done** |
| `is_hangable` (cl-06/07) + optional hang stub inside `break_line` | **done** (stub; not full hanging punctuation) |
| Full UCS / §C / glyph layout / document consumption | **OPEN** (Wave 4 stubs below) |

---

## Wave 4 — layout stubs + light host consume (X0–X6)

Honest scope: **stubs / heuristics only** — not full JLReq ruby/tate-chu-yoko metrics, not OpenType MATH stretchy fences.

Gate (when packages/eval touched):

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-std -p reciplexa-eval -p reciplexa-package --offline
```

| Unit | Status |
|------|--------|
| X0 — Ruby layout stub (`Ruby::estimate_box` / `ruby_estimate_box`) | **done** |
| X1 — Tate-chu-yoko layout stub (horizontal run width in vertical context) | **done** |
| X2 — Math stretchy delimiter stub (`Delimiter` grows with body; optional `stretch_factor`) | **done** |
| X3 — Math accent clearance stub (clearance above base) | **done** |
| X4 — Pipeline/host light consume (`lines_to_text_shapes` + break demo) | **done** |
| X5 — pkg_math / pkg_ja integration tip tests | **done** |
| X6 — Docs: `implemented-features` next steps | **done** |

### X0 — Ruby layout stub

- Given `Ruby { base, annotation, kind }`: base width = `char_em_width` sum; annotation width scaled ~0.5; advance = max(base, annotation); optional height bump for ruby band.
- API: `Ruby::estimate_box` and/or `ruby_estimate_box`.
- Tests. **Commit.**

### X1 — Tate-chu-yoko layout stub

- Estimate horizontal run width (digits/latin span) inside vertical context via `char_em_width`.
- Tests. **Commit.**

### X2 — Math stretchy delimiter stub

- `Delimiter::estimate_box` (via `MathAtom`) grows with body height (stretchy heuristic).
- Optional `stretch_factor` on the delimiter node / constructor.
- Tests. **Commit.**

### X3 — Math accent clearance stub

- Accent boxes add named clearance above (or below for underline) the base.
- Commit if not already good. **Commit.**

### X4 — Pipeline/host light consume

- Helper `lines_to_text_shapes`: Japanese string → `break_line` → one scene `Text` per line (or tagged records).
- Prefer `reciplexa-std` helper used from package / eval test; example `examples/pkg_ja_break.rpx` if builtins allow.
- **Commit.**

### X5 — pkg_math / pkg_ja integration tips

- More tip tests around math_value / japanese / package demos.
- **Commit.**

### X6 — Docs

- Update `lang/implemented-features.md` next-steps for Wave 4 stubs.
- Mark this plan table done. **Commit.**

## Remains after Wave 4

- Full UCS membership + normative JLReq §C break matrix + real hanging/justification.
- Font-backed ruby / tate-chu-yoko / vertical glyph orientation.
- OpenType MATH stretchy fences, accent attachment, matrix column alignment.
- Document/GUI pipeline consuming boxes as production layout (beyond light Text shapes / estimate_box).

---

## Wave 5 — UCS broaden + vertical / justify / scripts stubs (Y0–Y5)

Honest scope: **still stubs / subset tables** — not full JLReq UCS, not real vertical metrics, not production justification or OpenType MATH script attachment.

Gate:

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-std --offline
```

When package path touched (Y2): also `cargo test -p reciplexa-package --offline`.

| Unit | Status |
|------|--------|
| Y0 — Broader UCS-ish `classify_char` tables | **done** |
| Y1 — Vertical metrics stub | **done** |
| Y2 — Auto-materialize `package-resource` in package document path | **done** |
| Y3 — Justification stub | **done** |
| Y4 — Math scripts position stub | **done** |
| Y5 — Docs + `implemented-features` | **done** |

### Y0 — Broader UCS-ish classify tables

- Expand `classify_char` with more Unicode ranges that map cleanly (CJK symbols block chunks, more fullwidth, complete small-kana set incl. phonetic extensions, general punctuation).
- Still a **subset** — document remaining gaps in module/fn docs.
- Tests. **Commit.**

### Y1 — Vertical metrics stub

- `vertical_advance_em(c)` / line advance for `vertical-rl` using char classes.
- Optional simple `break_line_vertical` stub.
- Tests. **Commit.**

### Y2 — Auto-materialize package-resource in package eval path

- When `document_from_package_entry` / a small helper evaluates and finds `package-resource` records in the value tree, materialize paths (or attach a resolved path field).
- Fail-soft if not listed / no package root.
- Tests. **Commit:** `Auto-materialize package-resource records in package document path.`

### Y3 — Justification stub

- `justify_line(chars, target_em) -> Vec<(char, x_em)>` — naive distribute extra space at `Allowed` breaks only.
- Tests. **Commit.**

### Y4 — Math scripts position stub

- `scripts_attachment_offsets(base_box, sub_box, sup_box) -> (sub_x, sub_y, sup_x, sup_y)` heuristic.
- **Commit.**

### Y5 — Docs + implemented-features

- Mark Wave 5 done in this plan; update `lang/implemented-features.md` next-steps.
- **Commit.**

## Remains after Wave 5

- Full UCS membership + normative JLReq §C break matrix + real hanging/justification.
- Font-backed ruby / tate-chu-yoko / vertical glyph orientation / real vertical metrics.
- OpenType MATH stretchy fences, accent/script attachment, matrix column alignment.
- Document/GUI pipeline consuming boxes as production layout; load-time resource rewrite beyond fail-soft attach.

---

## Wave 6 — language builtins + host consume stubs (Z0–Z5)

Honest scope: **shipping-quality stubs** — wire existing Wave 5 Rust helpers as ker builtins / light host APIs. Not full JLReq justification, ruby-box layout, or OpenType MATH script attachment.

Gate:

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-std -p reciplexa-eval -p reciplexa-core --offline
```

When package path touched: also `cargo test -p reciplexa-package --offline`.

| Unit | Status |
|------|--------|
| Z0 — Ker builtins `break-line-vertical` / `justify-line` | **done** |
| Z1 — Justified line → Text shapes composition helper | **done** |
| Z2 — Vertical break example / builtin test | **done** |
| Z3 — `scripts_attachment_offsets` via math_value / scripts tag | **done** |
| Z4 — Hangable EOL policy stub (`hang_width_em`) | **done** |
| Z5 — Docs | **done** |

### Z0 — Language builtins for new layout helpers

- Prefer `justify-line` (string × target-em → cons/nil of `{char, x}` records) over a complex `ruby-box` builtin.
- Also wire `break-line-vertical` (string × max-em → cons/nil strings), mirroring std.
- Tests. **Commit.**

### Z1 — Scene helper: justified Japanese line → Text shapes

- Compose `justify_line` + positioned glyphs / `lines_to_text_shapes`-style Text shapes.
- Test. **Commit.**

### Z2 — Vertical break example

- `examples/pkg_ja_vertical_break.rpx` and/or Rust eval test using the builtin.
- **Commit.**

### Z3 — Math scripts attachment from math_value

- Use `scripts_attachment_offsets` from `math_value` or a public API test with package `math-scripts` tag.
- **Commit.**

### Z4 — Hangable end-of-line policy stub

- Document + simple API `hang_width_em` (not full JLReq hanging measure).
- **Commit.**

### Z5 — Docs

- Mark Wave 6 done in this plan; update `lang/implemented-features.md` / package READMEs.
- **Commit.**

## Remains after Wave 6

- Full UCS membership + normative JLReq §C break matrix + real hanging/justification.
- Font-backed ruby / tate-chu-yoko / vertical glyph orientation / real vertical metrics.
- OpenType MATH stretchy fences, accent/script attachment, matrix column alignment.
- Document/GUI pipeline consuming boxes as production layout; load-time resource rewrite beyond fail-soft attach.

---

## Wave 7 — denser break matrix + ruby/tate builtins + doc flow (A0–A5)

Honest scope: denser **subset** class×class break table (not full JLReq appendix C); language builtins for existing fontless ruby/tate stubs; light `doc-paragraph` soft-wrap into scene Text. Not production JLReq / font-backed layout.

Gate:

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-std -p reciplexa-eval -p reciplexa-core -p reciplexa-package --offline
```

| Unit | Status |
|------|--------|
| A0 — §C-inspired pair matrix expansion | **done** |
| A1 — Ruby language builtin `ruby-box` | **done** |
| A2 — Tate-chu-yoko builtin `tate-chu-yoko-width` | **done** |
| A3 — Document flow: doc-paragraph → `break_line` → Text shapes | **done** |
| A4 — Tip coverage | **done** |
| A5 — Docs + implemented-features | **done** |

### A0 — §C-inspired pair matrix expansion

- Replace/extend ad-hoc `break_opportunity` with denser class×class [`BREAK_PAIR_MATRIX`] covering inseparable, prohibited head/end, digit-open quirks already in package.
- Tests for matrix corners. **Commit:** `Expand japanese break pair matrix toward JLReq appendix C subset.`

### A1 — Ruby language builtin `ruby-box`

- `(ruby-box base annotation)` → record with width/height estimates using `Ruby::estimate_box`. **Commit.**

### A2 — Tate-chu-yoko builtin `tate-chu-yoko-width`

- `(tate-chu-yoko-width body)` → Number advance width (em). **Commit.**

### A3 — Document flow stub

- When lowering `doc-paragraph`, soft-wrap via `break_line` into multiple scene Text shapes. **Commit.**

### A4 — Tip coverage

- Tip tests for new builtins / matrix / doc wrap. **Commit.**

### A5 — Docs

- Mark Wave 7 done in this plan; update `lang/implemented-features.md` / package READMEs. **Commit.**

## Wave 8 — matrix/bigop stubs + ruby example + wrap helper + lock checksum (B0–B5)

Honest scope: fontless math column/limit offset stubs; language example for `ruby-box`; std Text soft-wrap helper; optional lockfile checksum field **without** verification. Not production matrix alignment / OpenType MATH / registry.

Gate:

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-std -p reciplexa-eval -p reciplexa-core -p reciplexa-package --offline
```

| Unit | Status |
|------|--------|
| B0 — `matrix_column_widths` + cases left-align stub | **done** |
| B1 — `bigop_limit_offsets` | **done** |
| B2 — Example `pkg_ruby.rpx` | **done** |
| B3 — `wrap_text_shape_content` via `break_line` | **done** |
| B4 — Lockfile `LockedPackage.checksum` optional stub | **done** |
| B5 — Docs + implemented-features | **done** |

### B0 — Math cases / matrix column alignment stub

- `matrix_column_widths(rows) -> Vec<f64>`; used by `estimate_box` for Matrix/Aligned.
- Cases left-align columns heuristically (`cases_column_align` / `matrix_cell_x_in_column`). **Commit.**

### B1 — BigOp limits placement stub

- `bigop_limit_offsets(op_box, lower, upper) -> (lower_x, lower_y, upper_x, upper_y)`. **Commit.**

### B2 — Example `pkg_ruby.rpx`

- `ruby-box` builtin + `document/page` surface demo. **Commit.**

### B3 — Soft-wrap Text content helper

- Prefer std helper `wrap_text_shape_content` (uses `break_line`); no GUI text_box hook required. **Commit.**

### B4 — OPEN-PKG lock checksum stub

- Optional `checksum: Option<String>` on `LockedPackage` (serde skip if none); **not verified**. **Commit.**

### B5 — Docs

- Mark Wave 8 done; update `lang/implemented-features.md` / package READMEs. **Commit.**

## Wave 9 — fraction/radical stubs + text wrap wire + hang-width (C0–C5)

Honest scope: fontless fraction rule / radical vinculum placement stubs; graphics_value soft-wrap via `wrap_text_shape_content`; language `hang-width` builtin; example. Not TeX `\fontdimen` / OpenType MATH / full JLReq hanging.

Gate:

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-std -p reciplexa-eval -p reciplexa-core -p reciplexa-package --offline
```

| Unit | Status |
|------|--------|
| C0 — Fraction rule thickness / clearance in `estimate_box` | **done** |
| C1 — Radical vinculum / index placement stub | **done** |
| C2 — `wrap_text_shape_content` in graphics_value text (`wrap-em` / newlines) | **done** |
| C3 — Language builtin `hang-width` | **done** |
| C4 — Example `pkg_ja_hang.rpx` | **done** |
| C5 — Docs + tip tests | **done** |

### C0 — Fraction rule thickness / clearance

- `FRAC_RULE_THICKNESS_EM` / `FRAC_NUM_CLEARANCE_EM` / `FRAC_DEN_CLEARANCE_EM` + `fraction_rule_metrics`; used by `estimate_box` for Fraction. **Commit.**

### C1 — Radical vinculum / index placement

- `RADICAL_VINCULUM_*` + `radical_vinculum_index_offsets`; wire into Radical `estimate_box`. **Commit.**

### C2 — graphics_value text soft-wrap

- Optional `wrap-em` on text records (fail-soft if absent); newlines also trigger wrap path via `wrap_text_shape_content` / hard-break split. **Commit.**

### C3 — `hang-width` builtin

- `(hang-width s)` → Number via `hang_width_em_char` (first char). **Commit.**

### C4 — Example

- `examples/pkg_ja_hang.rpx`: `break-line` + `hang-width` + `justify-line`. **Commit.**

### C5 — Docs + tip tests

- Mark Wave 9 done; update `lang/implemented-features.md` / package READMEs; tip coverage. **Commit.**

## Remains after Wave 9

- Full UCS membership + normative JLReq §C break matrix + real hanging/justification.
- Font-backed ruby / tate-chu-yoko / vertical glyph orientation / real vertical metrics.
- OpenType MATH stretchy fences, accent/script attachment, real matrix column alignment / big-op / fraction/radical metrics.
- Document/GUI pipeline consuming boxes as production layout (beyond soft-wrapped Text shapes / estimate_box); load-time resource rewrite beyond fail-soft attach; registry resolve + checksum verification (OPEN-PKG-001).

---

## Wave 10 — stackrel/aligned snap + math-box + vertical ruby/bou (D0–D5)

Honest scope: fontless stackrel/underbrace spacing + aligned column snap; language `math-box` via `math_value`+`estimate_box`; vertical ruby / bou (傍点) estimate stubs. Not TeX `\stackrel` metrics, OpenType MATH, or JLReq ruby/emphasis placement.

Gate:

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-std -p reciplexa-eval -p reciplexa-core -p reciplexa-package --offline
```

| Unit | Status |
|------|--------|
| D0 — Stackrel / underbrace-style spacing stubs | **done** |
| D1 — `aligned_column_x(rows, col)` | **done** |
| D2 — Builtin `math-box` | **done** |
| D3 — Example `pkg_math_box.rpx` | **done** |
| D4 — Vertical ruby / bou placement stubs | **done** |
| D5 — Docs + tip tests | **done** |

### D0 — Stackrel / underbrace spacing

- `STACKREL_GAP_EM` / `stackrel_spacing_offsets`; `UNDERBRACE_CLEARANCE_EM` / `underbrace_spacing`; wire Stackrel + underline `estimate_box`. **Commit.**

### D1 — Aligned at-column snap

- `aligned_column_x(rows, col) -> f64` via `matrix_column_widths` + `ALIGNED_COLUMN_GUTTER_EM`. **Commit.**

### D2 — `math-box` builtin

- `(math-box record-or-string)` → `{tag, width, height, depth}` via `estimate_math_box_from_value` / ord symbol. **Commit.**

### D3 — Example

- `examples/pkg_math_box.rpx`. **Commit.**

### D4 — Vertical ruby / bou

- `vertical_ruby_estimate_box` / `VerticalRubyBox`; `bou_estimate_box` / `bou_mark_offsets` (傍点). **Commit.**

### D5 — Docs

- Mark Wave 10 done; update `lang/implemented-features.md` / package READMEs; tip coverage. **Commit.**

## Remains after Wave 10

- Full UCS membership + normative JLReq §C break matrix + real hanging/justification.
- Font-backed ruby / tate-chu-yoko / vertical glyph orientation / real vertical metrics / bou placement.
- OpenType MATH stretchy fences, accent/script/stackrel attachment, real matrix column alignment / big-op / fraction/radical metrics.
- Document/GUI pipeline consuming boxes as production layout (beyond soft-wrapped Text shapes / estimate_box / math-box); load-time resource rewrite beyond fail-soft attach; registry resolve + checksum verification (OPEN-PKG-001).

---

## Wave 11 — doc-heading wrap + vertical-ruby/bou builtins + stretchy surface + PKG004 (E0–E5)

Honest scope: soft-wrap long `doc-heading` like paragraphs; thin language wrappers for vertical ruby / bou; package-bridge integration test for long JA paragraph → multiple Text; expose `stretch-factor` / `stretchy-delim`; diagnose missing listed resources (PKG004). Not production JLReq heading layout, OpenType MATH stretchy fences, or lock checksum verification.

Gate:

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-std -p reciplexa-eval -p reciplexa-core -p reciplexa-package --offline
```

| Unit | Status |
|------|--------|
| E0 — `doc-heading` soft-wrap via `break_line` | **done** |
| E1 — Builtins `vertical-ruby-box` / `bou-box` | **done** |
| E2 — Package document long JA paragraph → multiple Text (bridge) | **done** |
| E3 — `stretch-factor` field + `stretchy-delim` builtin | **done** |
| E4 — `diagnose_manifest_with_root` PKG004 missing resource | **done** |
| E5 — Docs + implemented-features | **done** |

### E0 — Document heading soft-wrap

- When lowering `doc-heading`, soft-wrap via `break_line` (shared helper with `doc-paragraph`). **Commit.**

### E1 — Vertical ruby / bou builtins

- `(vertical-ruby-box base annotation)` / `(bou-box body)` → estimate records. **Commit.**

### E2 — Bridge integration

- Package `document/page` with long JA paragraph → multiple scene Text shapes after graphics bridge. **Commit.**

### E3 — Stretchy delimiter language surface

- Optional `stretch-factor` on `math-delimiter` records; `(stretchy-delim left right body-height)` builtin. **Commit.**

### E4 — Resource diagnose

- Lock checksum remains write-None / read-accept stub (no empty-checksum warning). `diagnose_manifest_with_root` emits PKG004 for missing listed resources (same existence rules as `check_resources_exist`). **Commit.**

### E5 — Docs

- Mark Wave 11 done; update `lang/implemented-features.md` / package READMEs. **Commit.**

## Remains after Wave 11

- Full UCS membership + normative JLReq §C break matrix + real hanging/justification.
- Font-backed ruby / tate-chu-yoko / vertical glyph orientation / real vertical metrics / bou placement.
- OpenType MATH stretchy fences, accent/script/stackrel attachment, real matrix column alignment / big-op / fraction/radical metrics.
- Document/GUI pipeline consuming boxes as production layout (beyond soft-wrapped Text shapes / estimate_box / math-box / stretchy-delim); load-time resource rewrite beyond fail-soft attach; registry resolve + checksum verification (OPEN-PKG-001).

---

## Wave 12 — inseparable glue + western soft-wrap + math class spacing (F0–F4)

Honest scope: keep cl-08 runs glued in `break_line`; prefer ASCII-space soft-wrap over mid-latin; TeX-ish `class_spacing_em` in Row `estimate_box`; example + tip docs. Not normative JLReq §C, CSS `word-break`, or OpenType MATH muskips.

Gate:

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-std -p reciplexa-eval -p reciplexa-core -p reciplexa-package --offline
```

| Unit | Status |
|------|--------|
| F0 — cl-08 inseparable run glue in `break_line` | **done** |
| F1 — Prefer ASCII-space soft-wrap (avoid mid-latin when possible) | **done** |
| F2 — `class_spacing_em` + Row `estimate_box` | **done** |
| F3 — Example `pkg_math_spacing.rpx` + test | **done** |
| F4 — Tip + docs | **done** |

### F0 — Inseparable run glue

- Extend measure through contiguous cl-08 runs; snap cuts off cl-08×cl-08 pairs (horizontal + vertical soft-wrap). **Commit.**

### F1 — Western word soft-wrap

- Prefer Allowed break before ASCII space; long latin without spaces may still force mid-run. **Commit.**

### F2 — Math class spacing

- `class_spacing_em(left, right) -> f64` thin/med/thick muskip stubs; `MathAtom::spacing_class`; Row width includes gaps. **Commit.**

### F3 — Example

- `examples/pkg_math_spacing.rpx` + package test. **Commit.**

### F4 — Docs

- Mark Wave 12 done; update `lang/implemented-features.md` / package READMEs. **Commit.**

## Remains after Wave 12

- Full UCS membership + normative JLReq §C break matrix + real hanging/justification / word-break.
- Font-backed ruby / tate-chu-yoko / vertical glyph orientation / real vertical metrics / bou placement.
- OpenType MATH stretchy fences, accent/script/stackrel attachment, real matrix column alignment / big-op / fraction/radical / muskip metrics.
- Document/GUI pipeline consuming boxes as production layout (beyond soft-wrapped Text shapes / estimate_box / math-box / stretchy-delim); load-time resource rewrite beyond fail-soft attach; registry resolve + checksum verification (OPEN-PKG-001).

---

## Wave 13 — vertical glyph orientation stubs (G0–G2)

Honest scope: fontless `needs_tate_rotation` / `VerticalGlyphOrientation` for `vertical-rl` hosts. Not OpenType `vert`/`vrt2`, CSS `text-orientation`, or vertical presentation forms.

Gate:

```bat
set CARGO_TARGET_DIR=d:\reciplexa\target
set TEMP=d:\reciplexa\.tmp
set TMP=d:\reciplexa\.tmp
cargo test -p reciplexa-std -p reciplexa-eval -p reciplexa-core -p reciplexa-package --offline
```

| Unit | Status |
|------|--------|
| G0 — `needs_tate_rotation(c) -> bool` | **done** |
| G1 — `VerticalGlyphOrientation` + `vertical_glyph_orientation` | **done** |
| G2 — Tip + docs | **done** |

### G0 — Rotation flag

- ASCII / Latin-1 western → rotate; CJK / fullwidth latin → upright stub. **Commit.**

### G1 — Orientation enum

- `VerticalGlyphOrientation::{Upright,Rotated}`; `needs_tate_rotation` delegates. **Commit.**

### G2 — Docs

- Mark Wave 13 done; update `lang/implemented-features.md` / package README. **Commit.**

## Remains after Wave 13

- Full UCS membership + normative JLReq §C break matrix + real hanging/justification / word-break.
- Font-backed ruby / tate-chu-yoko / real vertical metrics / bou placement; OpenType `vert`/`vrt2` / CSS `text-orientation`.
- OpenType MATH stretchy fences, accent/script/stackrel attachment, real matrix column alignment / big-op / fraction/radical / muskip metrics.
- Document/GUI pipeline consuming boxes as production layout (beyond soft-wrapped Text shapes / estimate_box / math-box / stretchy-delim); load-time resource rewrite beyond fail-soft attach; registry resolve + checksum verification (OPEN-PKG-001).
