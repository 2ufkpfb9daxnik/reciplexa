# packages/japanese

JLReq-oriented Japanese typesetting package for Reciplexa (`OPEN-TEXT-JA-001`).

## Scope (honest)

This is a **first solid data + constructor layer**, not a full JLReq implementation.

| Module | Provides | Not yet |
|--------|----------|---------|
| `classes` | cl-01..cl-30 id roster, frequent aliases (`cl-01`/`cl-06`/`cl-15`/`cl-16`/`cl-19`…), `class-name` / `advance-em` | Full UCS membership tables for every class |
| `linebreak` | Kinsoku class predicates, sample glyph bunches, `break-between` | Complete §C pair matrices, streaming classifier |
| `kihon` | Kihon-hanmen constructor, line-rate constants, indent/heading band | Trim-size placement, multi-column books |
| `markup` | Heading / paragraph / ruby / tate-chu-yoko records | Live Text IR binding |

Normative reference: [W3C JLReq](https://www.w3.org/TR/jlreq/) (JIS X 4051–based). Spec anchors: `specification.md` language-package extension + `OPEN-TEXT-JA-001`.

## Import

```text
(import japanese/classes only cl-01 cl-19 advance-em)
(import japanese/linebreak only break-between kinsoku-profile)
(import japanese/kihon only kihon-hanmen line-rate-default)
(import japanese/markup only heading paragraph ruby)
```

## Expansion hooks

- Wire `LineBreakProvider` / `TextClassifier` protocols once common Text IR lands.
- Replace `classify-sample` with a real UCS map derived from JLReq appendix A.
- Consume kihon metrics from document lower instead of interim `text` shapes only.
