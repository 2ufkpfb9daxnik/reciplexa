# Step 9 — 構造化ページ作者編集と Provenance

**Status:** active  
**Depends on:** Step 8 complete (`52d088e` / `f9151cd`)  
**Normative:** `OPEN-EDT-001`, `OPEN-EDT-OVERRIDE-001`（製品 subset）, Part V provenance 基礎

## 非ゴール

- markup 作者同期（soft-refuse 維持）
- math / JA markup tree の first-class GUI 編集
- 完全 override policy / collaboration codec
- canvas 上の glyph 位置 nudge（構造編集と矛盾）

## Ordered slices

1. **Document CST layers + text roundtrip** — `collect_layers_document`, heading/paragraph/column 文字列の Source 書換; layer pane + props; canvas nudge soft-refuse
2. **Indent + columns params** — `paragraph-indented` の `indent-em`; `columns` count/gutter/各 column 文字列
3. **Provenance wiring** — val binding span → `NodeProvenance`; `document_snapshot_from_source` doc-page 分支; literal vs shared binding ラベル
4. **Step 9 gate** — `pkg_document_indent.rpx` / `pkg_columns.rpx` E2E（edit → save → reload → PDF/SVG）; markup soft-refuse regression

## Complete when

- heading / paragraph / indent / columns を GUI（layer + props）から Source へ戻し、保存・再読込・PDF/SVG が意味を保つ
- shared `val` binding を壊さず、逆変換不能を黙って graphics 化しない
- scene / export から binding span へ Provenance を辿れる（slice 3）
- markup は引き続き soft-refuse

## Examples

- `examples/pkg_document_indent.rpx` — heading + indented paragraph
- `examples/pkg_columns.rpx` — heading + 2 columns
- `examples/pkg_document.rpx` — richer blocks（spacer は slice 2 以降で検討）
