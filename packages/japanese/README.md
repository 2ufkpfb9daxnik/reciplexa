# packages/japanese

JLReq-oriented Japanese typesetting package for Reciplexa (`OPEN-TEXT-JA-001`).

## Scope (honest)

This is a **data + constructor layer**, not a full JLReq implementation or live line breaker.

| Module | Provides | Not yet |
|--------|----------|---------|
| `classes` | Full cl-01..cl-30 English names; JA names for frequent classes; more aliases (`cl-03`…`cl-12`, `cl-17`, `cl-27`…`cl-30`); `is-punctuation-class?` | Full UCS membership tables for every class |
| `linebreak` | Kinsoku predicates, `break-between` (more head/end cases), `sample-pair-rules` §C-inspired subset | Complete §C pair matrices, streaming classifier |
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

## Remaining gaps

- Full UCS → class map still incomplete; **Rust** `reciplexa_std::japanese::classify_char` covers a useful subset (authoritative for hosts; package `classify-sample` stays synthetic).
- `break-between` / `sample-pair-rules` are **subset stubs**; Rust `break_opportunity` mirrors common kinsoku only — not the normative JLReq appendix C matrix.
- Kihon trim/margin records are not consumed by CST lower / GUI (`KihonHanmen` helpers exist in std).
- SYN `(markup @heading(…) …)` and `japanese/markup` records are parallel; std `Ruby` / `TateChuYoko` align with package tags for future Text IR.

See also: [`lang/ja-math-deepen-plan.md`](../../lang/ja-math-deepen-plan.md).
