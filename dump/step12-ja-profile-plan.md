# Step 12 — Japanese Document Profile v2

**Status:** complete — all slices landed; gate §2.4.11; macro audit `3058e2a1` at `7c93262`  
**Depends on:** Step 11 complete (`3b38c1e` gate; docs `f7be9f3`)  
**Normative:** `roadmap.md` Step 12, `OPEN-TEXT-JA-001`（製品 subset）, JLReq Profile v1 上の拡張

## 非ゴール

- 完全 UCS / 規範的 appendix C 全体（Step 24）
- CSS `text-orientation` / `text-emphasis`
- 完全 `vrt2` / script-lang GSUB
- stub heuristic の削除（差分参照として維持）
- **階層レイヤー / テキストボックス単位の一括移動**（Step 14。現行 pane は flatten した葉。Step 12 ではやらない）

## Ordered slices

1. **Vertical punctuation JLReq position** — `。` `、` `ー` の縦組打込み/突き出し（Step 8 defer）
2. **Ruby overhang (horizontal simple)** — 親文字送り / measure overhang on `layout_simple_ruby`
3. **Jukugo ruby distribution** — per-base annotation placement（today refused）
4. **Vertical ruby** — font-backed side annotation in vertical-rl columns
5. **Step 12 gate** — combined `pkg_ja_profile_v2.rpx` + `step12_gates.rs` + §2.4.11

## Slice 1 受入（監査用）

- `layout_vertical_run` が reciprocal punctuation に基づき upright 約物を**セル内右上**へオフセットすること（回転はしない）
- `ー`（`ProlongedSoundMark`）は OpenType `vert` で縦用字形に差し替えること（無いときのみ −90° を **CJK と同じ em セル中心**で回転。ペンの ±1em ずらしはしない）
- 縦組ラテン（`ABC` 等、`needs_tate_rotation`）も同じ −90°、同一セル（GUI は egui 時計回り行列。逆行列でピボットを動かすと 1em 対角に飛ぶ）
- advance 幅は変えず ink 位置のみずらすこと（約物）
- `vert_gates` / `step12_gates` に fixture 回帰があること
- **非ゴール（slice 1）:** 行頭/行末コンテキスト依存の完全 JLReq、縦ルビ、jukugo

## Slice 2 受入（監査用）

- `layout_simple_ruby` の行送り `advance_mm` が親 base 幅のみであること（JLReq 親文字送り）
- 注釈は親 base 幅の中央に配置し、幅が親より広いときは `ink_width_mm` で overhang を記録すること
- stub `Ruby::estimate_box` は `max(base, annotation)` のまま（差分参照）
- `ruby_gates` / `step12_gates` に fixture 回帰があること
- **非ゴール（slice 2）:** jukugo 配分、縦ルビ、行頭/行末 overhang との衝突解決

## Slice 3 受入（監査用）

- `RubyKind::Jukugo` が `layout_ruby` で refused されないこと
- `distribute_jukugo_annotation` が親文字数に比例して注釈を分割すること（例: 東京/とうきょう → とう + きょう）
- 各注釈セグメントが対応する親文字セルの中央上に配置されること
- 行送り `advance_mm` は親 base 幅のみ（slice 2 と同じ）
- stub `Ruby::estimate_box` は変更しない
- **非ゴール（slice 3）:** 縦ルビ、形態素解析、行頭/行末 overhang 衝突

## Slice 4 受入（監査用）

- `layout_vertical_ruby` が simple ruby の縦組列に font-backed 側注ルビを置くこと
- 注釈は親列の右側（`VERTICAL_RUBY_SIDE_EM`）に縦積み、親列高さの中央に揃えること
- 縦方向の行送り `advance_mm` は親 base 列の高さのみ
- stub `vertical-ruby-box` / `estimate_vertical_box` は変更しない
- `ruby_gates` / `step12_gates` / `pkg_vert` host 経路に回帰があること
- **非ゴール（slice 4）:** 縦 jukugo 配分、行頭/行末 overhang 衝突

## Slice 5 受入（監査用）

- `examples/pkg_ja_profile_v2.rpx` が横組 ruby（simple + jukugo / overhang）・縦組・縦ルビ・縦中横・傍点を**同一ページ**の font-backed `GlyphRun`（傍点は円）として preview/export すること
- `step12_gates` に組み合わせ回帰があること
- stub `ruby-box` / `vertical-ruby-box` / `bou-box` / `tate-chu-yoko-width` は変えない
- 未実装 appendix C / CSS `text-orientation` / `text-emphasis` / 完全 `vrt2` を OPEN のまま明記すること
- **非ゴール（slice 5）:** 縦 jukugo 配分、行頭/行末 overhang 衝突、完全 appendix C

## Slice 監査（Composer 2.5；`fast` 不可）

| Slice | HEAD | subagent | 判定 | 備考 |
|-------|------|----------|------|------|
| 1 | `cd58f3c` | `215b5ce8` | complete | vertical punctuation position |
| 2 | `d881edc` | `5f95206c` | complete | ruby measure overhang |
| 3 | `dfa0262` | `8d621492` | complete | jukugo ruby distribution |
| 4 | `26282a9` | `6bc64b05` | complete | vertical ruby side annotation |
| 5 | `7b7a45e` | `0e10d3a2` | complete | combined `pkg_ja_profile_v2.rpx`；workspace 再測 `b56aa6a` |

## Complete when

- jukugo、縦ルビ、overhang が font-backed `GlyphRun` として preview/export される
- 横組・縦組・ruby・縦中横・傍点を組み合わせた適合 example が通る
- 実装規則と fixture が対応し、未実装 appendix C / CSS 範囲が明記される
