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

## Remaining gaps

- No UCS → class map (only `classify-sample` glyphs).
- `break-between` / `sample-pair-rules` are **subset stubs**, not the normative JLReq appendix C matrix.
- Kihon trim/margin records are not consumed by CST lower / GUI.
- SYN `(markup @heading(…) …)` and `japanese/markup` records are parallel; unification waits on Text IR.
