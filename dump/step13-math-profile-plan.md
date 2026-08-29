# Step 13 — Math Profile v2 と GUI 編集

**Status:** active — slice 6 next (`db043e7` slice 5 audit complete)  
**Depends on:** Step 12 complete (`7b7a45e` code; gate `b56aa6a`; docs `7c93262`; macro `3058e2a1`)  
**Normative:** `roadmap.md` Step 13, `implemented-features.md` OpenType MATH 残差、Math Profile v1 上の拡張

## 非ゴール

- 完全 TeX / 任意 MATH フォントの全 assembly レシピ網羅（Step 24）
- 黙った ASCII 代用（check/breve 等は Profile v1 どおり Failure / Loss）
- stub `reciplexa-std::math` heuristic の削除（差分参照として維持）
- markup 作者同期
- 階層レイヤー / テキストボックス一括移動（Step 14）
- JLReq appendix C 残り（Step 12 で閉じなかった分は Step 24）

## Ordered slices

1. **Glyph assembly** — stretchy delimiter が最大 variant を超えるとき MATH `GlyphAssembly`（parts + extender + overlap）で組み立てる。製品 path の visual scale をやめる。
2. **MATH kern** — italic correction、script/limit kern を MATH 表から適用。
3. **Font metrics** — 組版が参照する残り MATH constants / `\fontdimen` 相当を font table から取る。
4. **Alignment / equation numbers** — inline/display、alignment、式番号の実用 subset。
5. **First-class math tree GUI** — 分子・分母・script・matrix cell 等を GUI で選び、構造を保って Source へ戻す。
6. **Step 13 gate** — 組み合わせ example + `step13_gates` + §2.4.12。

## Slice 1 受入（監査用）

- `stretchy_delim`（または後継）が、最大 prepared variant でも足りない高さに対し `GlyphConstruction.assembly` を使うこと
- 組立は MATH `GlyphPart` の `full_advance` / connector overlap / extender 繰り返しに従い、**複数 GID** を `PositionedMath.glyphs` に積むこと（1 glyph の visual scale で高さを偽装しない）
- assembly も variant も足りないときは visual scale せず、構造化 `LayoutError`（または明示 Loss）にすること
- `step13_gates`（または既存 math gate への追加）に fixture 回帰があること
- **非ゴール（slice 1）:** MATH kern、式番号、first-class GUI 木編集

## Slice 2 受入（監査用）

- script / limit / 隣接核の italic correction と MATH kern を font table から適用すること
- 既存 fraction / radical / accent の Profile v1 配置を壊さないこと
- **非ゴール（slice 2）:** 式番号、GUI 木編集

## Slice 3 受入（監査用）

- 組版が参照する残り MATH constants を font から読むこと。slice 3 の対象:
  `delimitedSubFormulaMinHeight`、`spaceAfterScript`、`accentBaseHeight`、
  `upperLimitGapMin` / `upperLimitBaselineRiseMin`、
  `lowerLimitGapMin` / `lowerLimitBaselineDropMin`、
  `stackGapMin` / `stackDisplayStyleGapMin`、
  `overbar*` / `underbar*`、
  `radicalVerticalGap` / `radicalDisplayStyleVerticalGap` / `radicalExtraAscender` /
  `radicalKernBeforeDegree` / `radicalKernAfterDegree` / `radicalDegreeBottomRaisePercent`、
  display-style fraction numerator/denominator shifts。
  グリフ ink は glyf bbox（`0.7`/`0.2` em の heuristic を使わない）。
- heuristic em 定数へ silent fallback しないこと
- **非ゴール（slice 3）:** GUI 木編集、alignment 環境

## Slice 4 受入（監査用）

- inline / display の区別が文書内で安定して出力されること
- alignment と equation number の実用 subset。固定 example: `examples/pkg_live_align.rpx`
  （display `math-aligned` 2 行、列揃えの `=`、行ごと `(1)` / `(2)`、同一ページ `inline-math` は Text style 分数）
- **非ゴール（slice 4）:** 完全 amsmath、GUI 木の全ノード種

## Slice 5 受入（監査用）

- 数式の authoring ノード（fraction / scripts / delimiter 等）を GUI で選択し、葉の `GlyphRun` ではなく木として編集できること
- 分子・分母・script・matrix cell の変更が Source に構造を保って戻ること
- 未対応ノードは soft-refuse または明示 Failure であり、ASCII 代用しないこと
- **非ゴール（slice 5）:** Step 14 の階層レイヤー一般化

## Slice 6 受入（監査用）

- 組み合わせ example が assembly・kern・display math を同一ページで preview/export すること
- workspace gate（§2.4.12）が green
- 残る MATH / GUI 範囲を OPEN のまま明記すること
- **非ゴール（slice 6）:** Step 24 の全 MATH 閉包

## Slice 監査（Composer 2.5；`fast` 不可）

| Slice | HEAD | subagent | 判定 | 備考 |
|-------|------|----------|------|------|
| 1 | `79964b9` | `046e3938` | complete | glyph assembly + GUI fixture Unicode preview. Human GUI 2026-08-28 `pkg_live_math.rpx`。 |
| 2 | `95de694` | `f1dfc7a7` | complete | MATH italic correction + corner kern (scripts / limits / adjacent nuclei)。 |
| 3 | `13f54c5` | `c55ae04d` | complete | remaining MATH layout constants + glyf ink (no silent heuristic em)。 |
| 4 | `468ffce` | `a5d11a6a` | complete | alignment / equation numbers + document inline-math（`pkg_live_align.rpx`）。 |
| 5 | `db043e7` | `c39c0685` | complete | first-class math tree GUI（`f9302ee` + source span overlay `147e62c`）。Human GUI 2026-08-29 `pkg_live_math.rpx`。 |

各 slice 完了後、実装担当と別セッションの独立 subagent 1体が当該 slice のみを捜査する。`complete` の HEAD だけ次 slice へ進む。

## Complete when

- delimiter assembly、script/limit kern、fraction/radical/accent が font table で配置される
- 分子・分母・script・matrix cell 等を GUI 編集し Source へ構造を保って戻せる
- inline/display、alignment、equation number の実用 subset を文書内で使える
- ASCII 代用や unsupported assembly を黙って出さず、明示的 Failure または Loss とする
