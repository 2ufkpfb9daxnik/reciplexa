# JLReq follow-ups (Step 8)

**Status:** active (item 1 simple ruby complete; item 2 `vert` in progress; tate-chu-yoko / bou remain)  
**Execution order:** [`active-roadmap.md`](active-roadmap.md) Step 8, after Step 7 item 3 Profile v1  
**Normative anchors:** `OPEN-TEXT-JA-001`, JLReq ruby / vertical writing, Font Policy  
**Does not close:** full appendix C, full `vrt2`/script-lang GSUB, CSS `text-orientation`, tate-chu-yoko, bou, jukugo distribution

## Ordered slices

1. **Simple horizontal ruby** — font advances, half-size annotation centered above the base with a `RUBY_PARENT_GAP_EM` gap, combined advance `max(base, annotation)`. Stub `Ruby::estimate_box` stays the reference. Author `japanese/markup ruby` is unchanged.
2. **OpenType `vert` / vertical-rl** — stack CJK upright (GSUB `vert` GID when present), rotate ASCII −90°, columns progress right-to-left. Stub `lines_to_vertical_text_shapes` stays the reference. Tate-chu-yoko / bou / vertical ruby stay out.
3. Tate-chu-yoko / bou (later).

Jukugo per-character distribution, ruby overhang / 親文字送り, and vertical-ruby are out of slice 1.

## Complete slice 1 only when

1. Fixture-font simple ruby widths differ from stub `char_em_width` estimates.
2. Product output is `Shape::GlyphRun` (layout GIDs) for base and annotation.
3. Annotation sits **above the parent em-square** (baseline = parent baseline + 1 em + `RUBY_PARENT_GAP_EM`), half-size at `RUBY_ANNOTATION_SCALE`. The bump is not an overlap into the parent body.
4. `RECIPLEXA_TYPESET_ENGINE=stub` does not switch the `ruby-box` builtin to product metrics.
5. Host preview/export of `ja-ruby-demo` / `ja-ruby` uses the product engine unless stub is forced.

## Complete slice 2 only when

1. Vertical-rl CJK glyphs stack downward (decreasing page Y) with font/`vmtx` advances.
2. GSUB `vert` (then `vrt2`) single substitution is applied when the face has the feature; fixture stays identity.
3. ASCII/western that `needs_tate_rotation` is emitted in a −90° group; CJK stays upright.
4. `RECIPLEXA_TYPESET_ENGINE=stub` keeps `lines_to_vertical_text_shapes` / language-only `pkg_japanese_vertical`.
5. Host preview/export of `examples/pkg_vert.rpx` (`ja-vertical-demo` + `samples`) uses the product engine unless stub is forced.

