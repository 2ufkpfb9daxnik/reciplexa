# JLReq follow-ups (Step 8)

**Status:** complete (items 1–3 on code HEAD `caa2afd`; gate / docs HEAD `52d088e`)  
**Execution order:** [`active-roadmap.md`](active-roadmap.md) Step 8, after Step 7 item 3 Profile v1  
**Normative anchors:** `OPEN-TEXT-JA-001`, JLReq ruby / vertical writing, Font Policy  
**Does not close:** full appendix C, full `vrt2`/script-lang GSUB, CSS `text-orientation` / `text-emphasis`, jukugo distribution, vertical ruby

## Ordered slices

1. **Simple horizontal ruby** — font advances, half-size annotation centered above the base with a `RUBY_PARENT_GAP_EM` gap, combined advance `max(base, annotation)`. Stub `Ruby::estimate_box` stays the reference. Author `japanese/markup ruby` is unchanged.
2. **OpenType `vert` / vertical-rl** — stack CJK upright (GSUB `vert` GID when present), rotate ASCII −90°, columns progress right-to-left. Stub `lines_to_vertical_text_shapes` stays the reference.
3. **Tate-chu-yoko / bou** — ASCII 1–4 alnum upright in a 1 em vertical cell; bou marks as circles above (horizontal) or beside (vertical). Stub `tate-chu-yoko-width` / `bou-box` stay the reference.

Jukugo per-character distribution, ruby overhang / 親文字送り, and vertical-ruby stay out.

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

## Complete slice 3 only when

1. Tate-chu-yoko `"12"` shares one baseline (side-by-side) and occupies one surrounding em of vertical measure, less than stacked rotated digits.
2. Fixture-font tate-chu-yoko widths differ from stub `char_em_width` for latin.
3. Horizontal bou marks sit above the parent em-square; vertical bou marks sit beside the column.
4. `RECIPLEXA_TYPESET_ENGINE=stub` does not switch `tate-chu-yoko-width` / `bou-box` to product metrics.
5. Host preview/export of `examples/pkg_tcy_bou.rpx` uses the product engine unless stub is forced.

Slice 3 conditions 1–5 hold on code HEAD `caa2afd`. Step 8 is **complete** (gate HEAD `52d088e`; human GUI 2026-08-20). Human note: vertical punctuation (`。` `、` `ー`) still uses cell-center stacking without JLReq hang/trim — Step 12. Remaining `OPEN-TEXT-JA-001` bullets (jukugo, vertical ruby, overhang, full `vrt2` / `text-orientation` / CSS `text-emphasis`) stay OPEN and are assigned to Step 12 / Step 24. Step 9 is structured page authoring.
