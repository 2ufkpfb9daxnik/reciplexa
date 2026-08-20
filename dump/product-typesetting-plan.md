# Product Typesetting (Step 7 item 3)

**Status:** complete (PT-0〜PT-9 Profile v1 on code HEAD `3087565` — remaining OPEN listed in `implemented-features.md`, not implied closed)  
**Current model:** font-backed JLReq Profile v1 + Math Profile v1 on a shared `ttf-parser` substrate; `reciplexa-std` heuristics remain the explicit stub/reference path  
**Execution order:** [`active-roadmap.md`](active-roadmap.md) Step 7 item 3, after Direct Native v2 item 2/2b  
**Normative anchors:** `specification.md` Part IV §16–18, `IR-001` GlyphRun, `OPEN-TEXT-LAYOUT-001`, `OPEN-TEXT-JA-001`, Font Policy §39  
**Long-term milestones:** [`roadmap.md`](roadmap.md) M12 Japanese Document / M13 Mathematics

## Baseline: stub complete, not a product engine

The following described the host **before** PT-0. It remains the stub/reference path
(`document_from_package_entry`, `RECIPLEXA_TYPESET_ENGINE=stub`). Production preview/export
uses the engines in the Implementation record.

- Japanese layout uses `reciplexa_std::japanese::char_em_width`, `break_line`, and
  `layout_wrapped_paragraph_shapes`, then lowers strings directly to `reciplexa_scene::Text`.
- Math layout uses `MathAtom::estimate_box_with_style` and `layout_math_to_shapes`, with
  heuristic em constants and scene `Text` / `Line` output.
- PDF loads real CJK font data only at emission time through `CjkFontEmbed`; GUI installs a
  CJK face only into egui. Neither face drives the upstream line or math layout decision.
- `reciplexa_view::wrap_text_to_width` is a third, independent preview/export heuristic.
- The required Unicode → Analysis → Shaping → Line Layout → Positioned Text pipeline and
  cluster-preserving GlyphRun are not implemented.

Waves 4–29 / HC / LL remain **stub complete**. They are not extended in place and are not
relabeled as a product engine. Existing stub tests remain the differential/reference baseline.

## Target architecture

Product typesetting consists of one shared font/shaping substrate and two separate layout
engines:

```text
japanese/* and math/* package trees (author API unchanged)
                         |
        shared Font / ResourceId / shaping substrate
                    /                         \
       JLReq line-layout engine       OpenType MATH box engine
                    \                         /
          Shaped Text / Positioned Text / GlyphRun
                         |
             preview and export host adapters
```

The separation is normative:

- shared substrate owns face loading, stable font resource identity, glyph mapping, advances,
  shaping runs, Unicode ranges, and cluster mapping;
- the JLReq engine owns Japanese classification, line-break constraints, justification,
  annotation, and glyph orientation;
- the MATH engine owns Math IR measurement, MATH constants, script/limit placement,
  fraction/radical metrics, and glyph variant/assembly selection;
- language-specific rules do not leak into generic Render IR or backend emitters;
- font substitution that changes metrics triggers upstream relayout; a backend must not
  silently replace a font and retain stale positions.

Domain package exports stay package APIs. `import japanese/*`, `import math/*`, `.rpi`
interfaces, DN2 typed callables, and tagged record shapes remain unchanged. No raw FFI or
Rust symbol becomes an author API.

## Fixed design decisions

1. **Two engines, one substrate.** JLReq line layout and OpenType MATH are independent
   engines over common font/shaping data.
2. **Stubs remain references.** Existing `reciplexa-std` heuristics stay available for
   compatibility and differential tests until the production cutover is explicitly complete.
3. **Offline first.** Initial implementation uses the existing pure-Rust `ttf-parser`
   capability. No network dependency is added. A more complete shaping implementation
   (for example rustybuzz or HarfBuzz) requires a later explicit dependency decision.
4. **Deterministic fixtures.** Conformance tests use a redistributable, pinned fixture font.
   System fonts and `RECIPLEXA_CJK_FONT` / a future `RECIPLEXA_MATH_FONT` are runtime
   inputs, not test or ABI identities.
5. **Bounded profiles.** The first shipping targets are declared **JLReq Profile v1** and
   **Math Profile v1**. Full UCS, complete JLReq appendix C, and every MATH assembly are not
   implied by “v1 complete”; remaining bullets stay OPEN.
6. **Explicit host selection.** Product engines are introduced behind explicit parallel
   entry points or an engine selector. GUI/export defaults do not switch before positioned
   output and backend resource handling exist.

## Ordered implementation slices

### PT-0 — shared font and shaping substrate

- Add a dedicated crate (working name `reciplexa-text-layout`).
- Define font resource identity, face load, glyph lookup, units-per-em normalization,
  shaped glyph/cluster records, and structured missing-font/missing-glyph failures.
- Preserve source Unicode range and cluster mapping for every shaped run.
- Add a pinned, redistributable fixture font and offline tests.
- Do not wire GUI, PDF, package eval, or scene defaults in this slice.

### PT-1 — font-backed Japanese horizontal metrics

- Measure horizontal runs using real glyph advances instead of `char_em_width`.
- Keep the existing stub API and behavior unchanged.
- Prove with fixture strings that font-backed widths are stable and observably different
  from the 0.5em/1.0em heuristic.

### PT-2 — font-width Japanese line breaking

- Add a JLReq line-break provider using font advances and the existing class-level
  prohibition/allowance rules as an initial reference.
- Preserve cl-08 inseparable-run behavior and explicit hanging tolerance.
- Produce line segments with source ranges, natural widths, and break reasons.
- Do not claim normative appendix C completion.

### PT-3 — JLReq Profile v1

- Declare the exact Profile v1 character repertoire and §C rule subset.
- Move Profile v1 UCS classification and pair-rule data into versioned, data-driven tables.
- Implement font-backed punctuation solid/mirror widths, hanging, trimming, and line
  adjustment in the JLReq engine only.
- Add conformance vectors tied to JLReq classes and rules.
- Ruby, vertical writing, OpenType `vert`, tate-chu-yoko, and bou remain ordered follow-ups
  unless explicitly included in the Profile v1 declaration.

### PT-4 — OpenType MATH face and constants

- Add a separate MATH layout crate or module over the shared font substrate.
- Read real glyph advances and OpenType MATH constants with `ttf-parser` MATH support.
- Keep package Math trees, DN2 constructors, and current tagged records unchanged.
- Add fixture-font tests for known constants, symbol advances, and italic correction.

### PT-5 — Math Profile v1 layout

- Replace engine-path heuristic offsets for scripts/limits, fractions, and radicals with
  MATH constants.
- Implement glyph variants and stretchy delimiter assembly for the declared profile.
- Add accents, big operators, matrices/cases, and aligned/stack forms in independently
  testable increments.
- Keep `reciplexa_std::math` heuristic helpers as reference behavior.

### PT-6 — positioned output and scene adapter

- Define the minimum Shaped Text / Positioned Text representation needed by both engines.
- Preserve Unicode, source ranges, cluster mapping, font identity, glyph IDs, advances,
  offsets, direction, language, and writing mode.
- Add an adapter to existing scene output without changing GUI defaults.
- Avoid one scene `Text` node per Unicode scalar as the product representation.

### PT-7 — opt-in host integration

- Add opt-in product-engine paths to `document_value`, `math_value`, and
  `live_layout_bridge`.
- Ensure Japanese preview/export uses the same line layout instead of independently
  rewrapping through `reciplexa_view`.
- Upgrade selected existing examples to opt-in engine gates while preserving stub gates.
- Keep author-visible import and record APIs unchanged.

### PT-8 — GlyphRun, font policy, and backend emission

- Lower positioned output to the specification’s GlyphRun-equivalent Render/Visual IR.
- Reference fonts by stable resource identity, not native handles.
- Embed/subset the chosen font in PDF and define the SVG/PPTX representation policy.
- Reject or explicitly report unsupported substitution; metric-changing substitution
  requires relayout.
- Verify preview and export consume identical glyph positions.

### PT-9 — production cutover and observational hardening

- Run deep examples through stub and product engines with a documented equivalence class:
  exact structure where possible, tolerances for font metrics, and explicit expected
  differences where the stub is knowingly wrong.
- Cover `pkg_columns`, `pkg_document_indent`, `pkg_japanese_jlreq`, `pkg_math`,
  `pkg_math_spacing`, and `pkg_live_math`.
- Make the product engine the production preview/export path only after all cutover gates
  pass.
- Retain or deprecate stubs explicitly; never leave a silent fallback.
- Update OPEN contracts and current implementation truth.

## First implementation increment

The first atomic code increment is **PT-0 only**.

It is complete when:

1. a pinned fixture font is parsed offline;
2. horizontal advances come from the font and differ from at least one stub
   `char_em_width` expectation;
3. shaped clusters cover the complete input Unicode range without loss or overlap;
4. missing fonts and missing glyphs return structured errors;
5. no host, PDF, GUI, package API, or existing stub behavior changes.

Do not begin with full appendix C, vertical writing, stretchy delimiter assembly, or GUI
`doc-*` editing.

## Implementation record

PT-0〜PT-9 landed in `crates/reciplexa-text-layout` plus host adapters:

- PT-0: pinned generated fixture TTF, `LoadedFont` (TTC face index), shaped clusters, structured missing font/glyph.
- PT-1/2/3: font advances, versioned class-matrix (`jlreq-profile-v1.0`), hang/trim/justify applied onto `PositionedLine` glyph x.
- PT-4/5: MATH constants drive script/fraction/radical/bigop placement; stretchy delimiters select prepared MATH variant GIDs from the fixture construction (cmap `(` is not the painted GID). Accents: hat/tilde/dot/vec marks and bar/underline rules; check/breve/acute/grave/ring refuse ASCII substitution. Full assembly / MATH kern remain OPEN.
- PT-6: `GlyphRun` / `PositionedLine`; product JA/MATH scene adapters emit one `Shape::GlyphRun` per glyph so trim/justify, intra-row x, and layout GIDs survive. Graphics `text` keeps one cluster `GlyphRun` per authoring node.
- PT-7: `layout_doc_page_to_scene_with_engine` / `layout_math_to_shapes_product` / `document_from_math_demo_value`; host pipeline uses product for `doc-page`, `live-layout-demo`, `math-demo`, and graphics `text`.
- PT-8: production Visual IR lower emits `RenderNode::GlyphRun`; host PDF paints layout GIDs (`encode_gid_hex`) and embeds one CID face per layout digest (`host_product_font` + `host_math_font`). Digest mismatch is relayout. Latin-only `Text` is Helvetica. GUI preview installs those faces by digest and places `GlyphRun` clusters at layout advances. SVG/PPTX keep Unicode clusters at the same positions.
- PT-9: stub remains `document_from_package_entry` (tests/reference); `RECIPLEXA_TYPESET_ENGINE=stub` forces heuristics; cutover tests cover the declared examples.

Library APIs `layout_doc_page_to_scene` and `layout_math_to_shapes` stay stub so existing differential tests remain the reference baseline.

`document_from_graphics_value` stays stub (`Shape::Text`). Host product rewrites those leaves to cluster `GlyphRun` via `productize_document_text`. `pkg_japanese_jlreq` / `pkg_math_spacing` remain language-demo records (eval + engine coverage, not `doc-page`).

## Complete only when all of the following hold

1. Declared JLReq Profile v1 line break and line adjustment are driven by real font metrics.
2. Declared Math Profile v1 box layout is driven by OpenType MATH data and real glyph advances.
3. Stub heuristics are not the production preview/export path; they remain an explicit
   reference or are explicitly deprecated.
4. Unicode text and cluster mapping survive through shaped and positioned output.
5. Metric-changing font substitution causes relayout rather than backend silent replacement.
6. Existing author import APIs and DN2 typed callable contracts are unchanged.
7. Page-shaped declared examples (`pkg_columns`, `pkg_document_indent`, `pkg_math`,
   `pkg_live_math`) use the product engines in preview and export, with backend font
   embedding/resource tests. Language-demo records (`pkg_japanese_jlreq`,
   `pkg_math_spacing`) are eval + engine coverage, not `doc-page`.
8. Remaining `OPEN-TEXT-LAYOUT-001`, `OPEN-TEXT-JA-001`, OpenType MATH, and first-class
   editable-page work are listed precisely in `implemented-features.md`.
9. Workspace fmt, Clippy `-D warnings`, tests, all-target check, GUI check, and GUI/CLI
   host smoke are green on the same HEAD.

Conditions 1–9 hold for Profile v1 on code HEAD `3087565` (workspace gates this HEAD; a later docs commit only records the label). Step 7 item 3 is **complete**. Remaining `OPEN-TEXT-LAYOUT-001` bullets, full MATH assembly, and first-class editable math stay listed OPEN. Step 8 landed ruby / `vert` / tate-chu-yoko / bou; leftover JA is `OPEN-TEXT-JA-001`.

## Non-goals for this milestone

- markup author synchronization;
- `document/page` `doc-*` GUI CST editing;
- first-class editable math nodes on the GUI page;
- full book layout (footnotes, figures, references, and indexes);
- network font registry or downloadable fonts;
- closing `OPEN-NATIVE-PKG-001`;
- treating coverage percentage or conformance-case count as product completion.

These remain explicit OPEN or later-roadmap work. Product engine layout may be consumed by
preview/export before those author-editing features exist.

## Gate

Each slice includes direct unit/integration tests and stub/reference differentials where
meaningful. Before a slice commit, run the relevant scoped tests and Clippy. At milestone
cutover run the full offline gates from
[`implementation-agent-prompt.md`](implementation-agent-prompt.md), including:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
cargo check --workspace --all-targets --offline
cargo check --offline -p reciplexa-gui
cargo run --offline -p reciplexa-gui -- --smoke examples/text_line.rpx
cargo run --offline -p reciplexa -- examples/text_line.rpx .tmp/smoke.pdf
```

No test gate may depend on network access or an unpinned system font.
