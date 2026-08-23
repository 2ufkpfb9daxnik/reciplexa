# Step 12 — Japanese Document Profile v2

**Status:** active — slice 1 next  
**Depends on:** Step 11 complete (`3b38c1e` gate; docs `f7be9f3`)  
**Normative:** `roadmap.md` Step 12, `OPEN-TEXT-JA-001`（製品 subset）, JLReq Profile v1 上の拡張

## 非ゴール

- 完全 UCS / 規範的 appendix C 全体（Step 24）
- CSS `text-orientation` / `text-emphasis`
- 完全 `vrt2` / script-lang GSUB
- stub heuristic の削除（差分参照として維持）

## Ordered slices

1. **Vertical punctuation JLReq position** — `。` `、` `ー` の縦組打込み/突き出し（Step 8 defer）
2. **Ruby overhang (horizontal simple)** — 親文字送り / measure overhang on `layout_simple_ruby`
3. **Jukugo ruby distribution** — per-base annotation placement（today refused）
4. **Vertical ruby** — font-backed side annotation in vertical-rl columns
5. **Step 12 gate** — combined `pkg_ja_profile_v2.rpx` + `step12_gates.rs` + §2.4.7

## Slice 1 受入（監査用）

- `layout_vertical_run` が reciprocal punctuation に基づき upright 約物を**セル内左上**（top-start）へオフセットすること
- `ー`（`ProlongedSoundMark`）が縦組向けに **−90°** 回転すること
- advance 幅は変えず ink 位置のみずらすこと
- `vert_gates` / `step12_gates` に fixture 回帰があること
- **非ゴール（slice 1）:** 行頭/行末コンテキスト依存の完全 JLReq、縦ルビ、jukugo

## Slice 監査（Composer 2.5；`fast` 不可）

| Slice | HEAD | subagent | 判定 | 備考 |
|-------|------|----------|------|------|
| 1 | — | — | — | vertical punctuation position |

## Complete when

- jukugo、縦ルビ、overhang が font-backed `GlyphRun` として preview/export される
- 横組・縦組・ruby・縦中横・傍点を組み合わせた適合 example が通る
- 実装規則と fixture が対応し、未実装 appendix C / CSS 範囲が明記される
