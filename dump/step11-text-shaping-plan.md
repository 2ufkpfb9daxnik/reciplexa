# Step 11 — Text shaping / font resource v2

**Status:** active — slice 4 landed; slice 5 next  
**Depends on:** Step 10 complete (`0abe7bc`)  
**Normative:** `roadmap.md` Step 11, `OPEN-TEXT-LAYOUT-001`（製品必須 subset）, spec Part IV §17–18

## 非ゴール

- JLReq appendix C 完全閉包（`OPEN-TEXT-JA-001` → Step 12）
- 完全 OpenType MATH assembly（Step 13）
- ネットワーク font registry / ダウンロード font
- markup 作者同期
- HarfBuzz ネイティブ FFI（Rust 純粋 shaping ライブラリを優先検討）

## Slice 3 受入（監査用）

- `rustybuzz` を workspace 依存として追加し、`shape_run` / `shape_run_single_font` が `shape_run_complex` 経由になること
- `liga` feature を有効化して cluster byte span を `ShapedGlyph` に保持すること
- pinned `ReciplexaLigaFixture` で `fi` が 1 glyph・cluster `0..text.len()` になること（GSUB 相当は `fixture_liga::coalesce_fixture_ligatures` で fixture 限定）
- `require_emit_matches_shaped_run` で digest 不一致時に `SubstitutionRequiresRelayout` になること
- **非ゴール（slice 3）:** host `productize_shape_text` 配線（slice 4）、完全 UAX#9 bidi、任意 system font の OTL

## Slice 4 受入（監査用）

- `productize_shape_text` / `scene_text_to_glyph_run` が `shape_run_complex` + `visual_glyph_indices` で visual-order の `gids` / `advances_mm` を emit すること
- `require_emit_matches_shaped_run` が host productize 経路で digest 不一致時に `SubstitutionRequiresRelayout` になること（`productize_shape_text_emit`）
- `PositionedLine::from_segment` が `visual_positions_em` を使い、JA paragraph の `layout_wrapped_paragraph_product` と `positioned_line_to_glyph_shapes` の glyph 座標が一致すること
- per-glyph `font_digest` が `PositionedGlyph` から export へ伝播すること
- **非ゴール（slice 4）:** 完全 UAX#9 bidi、任意 system font OTL、justify+bidi 混在の完全整合

## Ordered slices

1. **Typed layout protocol + font fallback** — `ShapingAttributes`（script / language / direction / writing mode）、決定的 `FontFallbackChain`、per-glyph `FontId`、構造化 `MissingGlyph` ✓
2. **Bidi logical→visual** — mixed LTR/RTL + 日本語 fixture、paragraph level reorder、`Direction::Rtl` を `GlyphRun` へ伝播 ✓
3. **Complex shaping** — ligature / GSUB 級 cluster 保持（`rustybuzz` 明示依存）、metric-changing substitution → relayout 配線 ✓
4. **Host product wiring** — `productize_shape_text` / JA paragraph / preview+export が同一 glyph 座標を消費
5. **Step 11 gate** — workspace gate + bidi/fallback fixture tests + §2.4.6

## Slice 監査（Composer 2.5；`fast` 不可）

各 slice 完了後、実装担当と別セッションの独立 subagent 1体が当該 slice のみを捜査する。`complete` の HEAD だけ次 slice へ進む（[`active-roadmap.md`](active-roadmap.md)「Slice完了の判定」）。

| Slice | HEAD | subagent | 判定 | 備考 |
|-------|------|----------|------|------|
| 1 | `956d253` | `895bf444` | **complete** | protocol + fallback + step11_gates(2) |
| 2 | `7b922a0` | `4defb956` | **complete** | bidi + `from_shaped_run_with_attrs`；fmt 修正後 scoped gate green |
| 3 | `3850f59` | `5bfa267c` | **complete** | rustybuzz + `complex.rs` + fixture liga coalescing + `require_emit_matches_shaped_run` |
| 4 | — | — | — | host product wiring（実装中） |

## Complete when

- fallback chain が決定的で、missing glyph は fallback または構造化 Failure
- Unicode range と cluster map を保持（ligature / complex script を含む）
- metric-changing substitution は必ず relayout（[`policy::select_font`]）
- mixed LTR/RTL + 日本語 fixture で preview/export の glyph 位置が一致
- `OPEN-TEXT-LAYOUT-001` の製品必須 bullets（型・fallback・shaping・bidi）が閉じ、line break / justification は Step 12 以降も OPEN 明示

## Landed modules

- `crates/reciplexa-text-layout/src/protocol.rs` — typed shaping run descriptor
- `crates/reciplexa-text-layout/src/fallback.rs` — deterministic font fallback
- `crates/reciplexa-text-layout/src/bidi.rs` — paragraph bidi（slice 2）
- `crates/reciplexa-text-layout/src/complex.rs` — rustybuzz shaping
- `crates/reciplexa-text-layout/src/position.rs` — productize + JA/export parity（slice 4）
