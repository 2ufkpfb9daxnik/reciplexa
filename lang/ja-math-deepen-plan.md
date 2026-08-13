# OPEN-TEXT-JA-001 / math layout light — deepen plan

Status tracking for deepening `packages/japanese` + `packages/math` **beyond** the N3/N4 synthetic RPX natives, by putting **authoritative Rust tables and typed APIs** in `crates/reciplexa-std`.

## Honest scope

| Deliver | Do **not** claim |
|---------|------------------|
| JLReq-inspired **character class enum** + `classify_char` for a **useful subset** of punctuation / kana / ideographs / western | Full UCS membership tables for cl-01..cl-30 |
| Pair **break opportunity** stub (kinsoku-inspired forbidden starts/ends) | Normative JLReq appendix C pair matrix / streaming linebreak |
| Kihon-hanmen / writing-mode / ruby / tate-chu-yoko **data types** | Live line layout, glyph positioning, Document lower consumption |
| Deeper `MathAtom` tree + **fontless** box metric estimates + matrix/align/stack nodes | TeX/SATySFi glyph layout, stretchy fences, OpenType MATH tables |

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

## Japanese units (J0–J4)

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
| M0 | **done** |
| M1 | **done** |
| M2 | **done** |
| M3 | pending |

## Remains after this deepen (full OPEN-TEXT-JA-001)

- Complete UCS → class membership for all JLReq classes.
- Normative §C break pair matrix + hangable punctuation / justification.
- Font-backed shaping, ruby layout, tate-chu-yoko metrics, vertical glyph orientation.
- Math: OpenType MATH / stretchy fences / real matrix column alignment / linebreak in display math.
- Document pipeline + GUI consuming kihon / ruby / math boxes (not only trees).
