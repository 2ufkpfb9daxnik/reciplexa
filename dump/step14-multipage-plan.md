# Step 14 — 複数ページ出版レイアウト

**Status:** active — slice 3 complete; slice 4 next  
**Depends on:** Step 13 complete (`f615af5` code; gate §2.4.12; docs `6fa6f9f`; macro `8a1b1ce0`)  
**Normative:** `roadmap.md` Step 14, `implemented-features.md` 本番ページ消費 / 階層レイヤー

## 非ゴール

- markup 作者同期（Step 21 / 22）
- Export / print / accessibility parity（Step 15）
- 完全 CSS ページモデル / named page / 任意柱・ノンブル体系
- 学術文書の完全脚注・参考文献スタイル
- 完全 TeX / 任意 MATH フォントの assembly レシピ（Step 24）
- stub heuristic の削除
- network registry / portable native fallback（Step 16）

## Ordered slices

1. **階層レイヤー / テキストボックス一括移動** — authoring 1 ノードを親行、その `GlyphRun` を子にする。テキストボックスを一塊として選択・移動する。
2. **Page break / 版面** — 明示 page break と版面（マージン / 本文領域）を package ページ列へ落とす。
3. **Flow 要素** — heading / list / note / figure / table の実用 subset。
4. **注・参照** — footnote / reference / index の最小一貫経路。再 flow 後も identity と参照が安定する。
5. **部分 relayout** — 数式・縦組・図表を含む複数ページで部分 relayout が full rebuild と一致する。
6. **Step 14 gate** — `japanese-article.rpx` 相当 + `step14_gates` + §2.4.13。

## Slice 1 受入（監査用）

- Layers pane が authoring ノードを親、そのノードが生む `GlyphRun` を子として階層表示すること
- 親行（テキストボックス）を一塊として選択・移動でき、子 glyph の相対配置を保って Source へ戻ること
- 縦組の per-glyph `GlyphRun` は親テキストノードの子であること（平坦リストにしない）
- Step 13 の math 字下げリストをこの階層へ統合するか、math 木が親ノードとして残ること
- 直接 test（GUI sync または `step14_gates`）に回帰があること
- **非ゴール（slice 1）:** page break、footnote、複数ページ、完全 CSS ページモデル

## Slice 2 受入（監査用）

- 明示 page break が次の描画アイテムを新しい package ページへ送ること
- 版面（本文領域）が preview / export で同じマージンを消費すること
- **非ゴール（slice 2）:** 自動改ページの完全アルゴリズム、柱・ノンブル、footnote

## Slice 3 受入（監査用）

- heading / list / note / figure / table の実用 subset が文書内で安定して preview / export されること
- 固定 example: `examples/pkg_flow_blocks.rpx`
- **非ゴール（slice 3）:** 完全 CSS リスト / キャプション体系、浮動体の全配置規則；list/note/figure/table の first-class GUI 構造編集

## Slice 4 受入（監査用）

- footnote / reference / index の最小一貫経路があること
- 再 flow 後も identity と参照が安定すること
- **非ゴール（slice 4）:** 学術文書の完全書誌スタイル、相互参照 UI の全種

## Slice 5 受入（監査用）

- 数式・縦組・図表を含む複数ページ文書で、部分 relayout の結果が full rebuild と一致すること
- **非ゴール（slice 5）:** Step 15 の backend parity、任意巨大文書の増分キャッシュ

## Slice 6 受入（監査用）

- `japanese-article.rpx` 相当が日本語・数式・図表を含む小冊子として GUI 編集、save / load、preview / export できること
- workspace gate（§2.4.13）が green
- 残る出版 / export 範囲を OPEN のまま明記すること
- **非ゴール（slice 6）:** Step 15 の Output Profile / Loss / a11y 閉包

## Slice 監査（Composer 2.5；`fast` 不可）

| Slice | HEAD | subagent | 判定 | 備考 |
|-------|------|----------|------|------|
| 1 | `2e3953a` | `74995f96` | **complete** | hierarchical layers / text-box move（tree `f158476`、grab `2e3953a`）。Human GUI 2026-09-11. Docs `6f4f02a`. |
| 2 | `a47f376` | `4c273a87` | **complete** | page break / 版面。Human GUI 2026-09-11 (`examples/pkg_pagebreak.rpx`). Docs `a618e33`. |
| 3 | `23f21f2` | `6f46b9d4` | **complete** | heading / list / note / figure / table（feat `2013c25`、grid/caption `23f21f2`）。Human GUI 2026-09-14 (`examples/pkg_flow_blocks.rpx`). Docs `0f41e47`. |

各 slice 完了後、実装担当と別セッションの独立 subagent 1体が当該 slice のみを捜査する。`complete` の HEAD だけ次 slice へ進む。

## Complete when

- page break、版面、heading、list、note、figure、table の実用 subset が動く
- 階層レイヤー（authoring ノードが親、glyph が子）。テキストボックスを一塊として選択・移動できる
- footnote、reference、index の最小一貫経路があり、再 flow 後も identity と参照が安定する
- 数式・縦組・図表を含む複数ページ文書を部分 relayout しても full rebuild と同じ結果になる
- `japanese-article.rpx` 相当を GUI 編集、save / load、preview / export できる
