# packages/japanese

JLReq-oriented Japanese typesetting package for Reciplexa (`OPEN-TEXT-JA-001`).

## Scope (honest)

This is a **data + constructor layer**, not a full JLReq implementation or live line breaker.

| Module | Provides | Not yet |
|--------|----------|---------|
| `classes` | Full cl-01..cl-30 English names; JA names for frequent classes; more aliases (`cl-03`…`cl-12`, `cl-17`, `cl-27`…`cl-30`); `is-punctuation-class?` | Full UCS membership tables for every class |
| `linebreak` | Kinsoku predicates, `break-between` (more head/end cases), `sample-pair-rules` §C-inspired subset; package `hangable-class?` | Complete §C pair matrices, streaming classifier, full hanging punctuation |
| — (language / std) | Ker builtins `classify-char` / `break-between` / `break-line` / `break-line-vertical` / `justify-line` / `ruby-box` / `vertical-ruby-box` / `bou-box` / `tate-chu-yoko-width` / `hang-width` / `math-box` / `stretchy-delim`; Rust denser `BREAK_PAIR_MATRIX` + `is_hangable` / `hang_width_em` (cl-06/07) optionally used by `break_line`; `doc-paragraph` / `doc-heading` soft-wrap via `break_line`; `wrap_text_shape_content`; graphics_value text optional `wrap-em`; `vertical_ruby_estimate_box` / `bou_estimate_box` (傍点) | Normative §C / hang / justification / font-backed ruby |
| `kihon` | Kihon-hanmen, line-rate, trim-size (`a5`/`b5`/`a4`), margins, `place-hanmen`, multi-column stub | Real placement in document lower, multi-column books |
| `markup` | Heading / paragraph / ruby / tate-chu-yoko / doc records | Live Text IR binding; SYN `@heading(…)` macros stay separate |

Normative reference: [W3C JLReq](https://www.w3.org/TR/jlreq/) (JIS X 4051–based). Spec anchors: `specification.md` language-package extension + `OPEN-TEXT-JA-001`.

## Import

```text
(import japanese/classes only cl-01 cl-19 advance-em)
(import japanese/linebreak only break-between kinsoku-profile sample-pair-rules)
(import japanese/kihon only kihon-hanmen line-rate-default a5-trim place-hanmen)
(import japanese/markup only heading paragraph ruby doc)
```

## Examples

- `examples/pkg_japanese_jlreq.rpx` — classes / linebreak / kihon smoke
- `examples/pkg_markup_ja.rpx` — package-record mirror of `examples/markup_ja.rpx` `@`-markup (macros remain the GUI/preview path today)
- `examples/pkg_japanese_vertical.rpx` — vertical-text / tategaki / `vertical-flow` graphics-bridge stubs
- `examples/pkg_ja_classify.rpx` / `pkg_ja_break.rpx` / `pkg_ja_vertical_break.rpx` / `pkg_ja_hang.rpx` — language builtins (no package import)
- `examples/pkg_ruby.rpx` — `ruby-box` + `document/page` demo
- `examples/pkg_math_box.rpx` — `math-box` builtin (math tag record / symbol string → box metrics)

## Remaining gaps

- Full UCS → class map still incomplete; **Rust** `reciplexa_std::japanese::classify_char` covers a useful subset (authoritative for hosts; package `classify-sample` stays synthetic).
- Language builtins `classify-char` / `break-between` / `break-line` / `break-line-vertical` / `justify-line` / `ruby-box` / `vertical-ruby-box` / `bou-box` / `tate-chu-yoko-width` / `hang-width` / `math-box` / `stretchy-delim` call the same Rust APIs (no package import required).
- `break-between` / `sample-pair-rules` are **subset stubs**; Rust `BREAK_PAIR_MATRIX` / `break_opportunity` densifies class×class kinsoku + digit-open quirks — still **not** the normative JLReq appendix C matrix.
- `is_hangable` (cl-06/07) mirrors package `hangable-class?`; `break_line` may let those glyphs stick past the em budget. `hang_width_em` / language `hang-width` document a **0.5em** policy stub (not full JLReq hanging / justification).
- Host helpers: `lines_to_text_shapes` / `break_line_to_text_shapes` / `justify_line_to_text_shapes` / `wrap_text_shape_content` (glyph Text placements from `justify_line` / soft-wrap of a scene Text). Document lower soft-wraps long `doc-paragraph` / `doc-heading` text via `break_line` into multiple scene Text shapes (package bridge integration covered). Graphics_value text records may set optional `wrap-em` (or include newlines) to soft-wrap via the same helper.
- Vertical ruby / bou: `vertical_ruby_estimate_box` / `bou_estimate_box` / `bou_mark_offsets` are fontless side-placement stubs (not JLReq / CSS `text-emphasis`); language `vertical-ruby-box` / `bou-box` expose them.
- Kihon trim/margin records are not consumed by CST lower / GUI (`KihonHanmen` helpers exist in std).
- SYN `(markup @heading(…) …)` and `japanese/markup` records are parallel; std `Ruby` / `TateChuYoko` align with package tags for future Text IR.

See also: [`lang/ja-math-deepen-plan.md`](../../lang/ja-math-deepen-plan.md).
