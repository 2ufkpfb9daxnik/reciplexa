# JLReq follow-ups (Step 8)

**Status:** active (item 1 simple ruby complete; `vert` / tate-chu-yoko / bou remain)  
**Execution order:** [`active-roadmap.md`](active-roadmap.md) Step 8, after Step 7 item 3 Profile v1  
**Normative anchors:** `OPEN-TEXT-JA-001`, JLReq ruby / vertical writing, Font Policy  
**Does not close:** full appendix C, OpenType `vert`, tate-chu-yoko, bou, jukugo distribution

## Ordered slices

1. **Simple horizontal ruby** — font advances, half-size annotation centered above the base, combined advance `max(base, annotation)`. Stub `Ruby::estimate_box` stays the reference. Author `japanese/markup ruby` is unchanged.
2. OpenType `vert` / vertical writing (later).
3. Tate-chu-yoko / bou (later).

Jukugo per-character distribution, ruby overhang / 親文字送り, and vertical-ruby are out of slice 1.

## Complete slice 1 only when

1. Fixture-font simple ruby widths differ from stub `char_em_width` estimates.
2. Product output is `Shape::GlyphRun` (layout GIDs) for base and annotation.
3. Annotation sits above the base in page Y-up at `RUBY_ANNOTATION_SCALE`.
4. `RECIPLEXA_TYPESET_ENGINE=stub` does not switch the `ruby-box` builtin to product metrics.
5. Host preview/export of `ja-ruby-demo` / `ja-ruby` uses the product engine unless stub is forced.
