# Reciplexa Active Roadmap

**Authority:** 現在の実行順と完了条件の正本。専門AIは最初の未完了Stepから着手する。  
**Status source:** 実装済み機能と実測gateは [`implemented-features.md`](implemented-features.md) を正とする。  
**Normative source:** 契約を変更するときは [`specification.md`](specification.md) を同時に確認する。

## 状態の意味

- `planned`: 未着手
- `active`: 作業中
- `blocked`: 外部判断または先行Step待ち
- `complete`: 受入条件とgateを満たした

`unit complete`、`stub complete`、`spec conformant`、`product slice complete` は別の状態である。単に `done` と書かない。

## Step 0 — 引き継ぎ基盤

**Status:** complete (`4859e4c`)

- 人間の入口を `dump/README.md` に一本化した
- 専門AI向け実行プロンプトと常時ルールを整備した
- atomic commitとcommit前checkを固定した

**Complete when:** 新しいAIが `dump/README.md` から、追加説明なしで次のStepとgateを特定できる。

## Step 1 — 品質gate復旧

**Status:** complete (`56fbec5`, `a8bf9d9`, `aa8800e`, `18e0571`)

- language resolverのgraphics quarantineをS6b surface gateから分離した
- Japanese paragraph配置引数を `ParagraphSceneLayout` にまとめた
- interim-surface fixtureをCIで先に実行し、workspace testをgreenにした

**Complete when:** 全gateが同じHEADで成功し、実測結果が現状正本に記録される。 → HEAD `5d2c5bf` で記録済み。

## Step 2 — 文書正本化

**Status:** complete (`fc2a5b0`)

- `implemented-features.md` を唯一の現状正本にした
- 未完契約と有効な受入条件を現行文書へ移した
- 移行済みの古い完了計画を削除し、残存リンクを直した

**Complete when:** 人間は `dump/README.md`、AIは本roadmapと実行プロンプトだけで開始でき、削除済み文書へのリンクがない。

## Step 3 — Hybrid Native v1 / Direct Native v2

**Status:** complete (`2fe1d04`)

- 現行をHybrid Native v1として正確に記述した
- Direct Native v2を必須将来マイルストーンとして登録した
- portable fallback契約をOPENとして追跡する

**Complete when:** Hybrid v1を完全direct nativeと誤読できず、DN2の機械的な完了条件が仕様・roadmap・適合台帳から辿れる。

## Step 4 — package経路の型安全性

**Status:** complete (`e798243`, `00fb457`, `6301513`, `6ef9ebe`)

- packageソースと共存できるtop-level effect検査を追加した
- package elaborate後、eval前にCore型推論する
- preview、snapshot、exportの各package経路へ接続した
- load=`package`、静的型不一致=`type`、eval/bridge=`package` のstageを保つ

**Complete when:** 正常packageが動き、型不一致とeffect不一致が評価前に構造化エラーとなり、未知importと非page mainも適切なstageで失敗する。

## Step 5 — 編集可能な1ページ

**Status:** complete (`537aa33`, `5972f54`, `047d38f`, `b452113`, `5d2c5bf`) — **product slice complete**

- 対象: package形式1ページのcircle、line、text等の基本graphics
- package ASTのinsert、delete、reorder、text更新を実装した
- GUIのInsert/Delete/Paste/Z-order/Propertiesをpackage経路へ接続した
- move、resize、保存、再読込、PDF/SVG出力をE2Eで固定した

**Non-goals:** markup作者同期、製品級JA/math、doc-*作者編集。

**Complete when:** `examples/text_line.rpx` 相当をGUI操作相当APIで編集し、保存・再読込後のsceneとPDF/SVGに編集結果が残り、interim構文が混入しない。

人間確認: GUIで基本図形と文字を編集し、保存後に同じ表示へ戻ることを一度確認する。

## Step 6 — 完了状態の再計測

**Status:** complete (this document + [`implemented-features.md`](implemented-features.md) §2.4–2.5, HEAD `5d2c5bf`)

- 全gateとVertical Slice E2Eを再実行した
- `implemented-features.md` に保証範囲と非対象を記録した
- 長期 `roadmap.md` のPhase 4へ受入条件を対応付けた
- coverageやconformance分類を製品完成率として扱わない

**Complete when:** 同じHEADの実装、テスト結果、現状正本、長期roadmapが一致する。

## Step 7 — 次工程

**Status:** active — item 1 local/offline slice complete; item 2 Direct Native v2 next

順序:

1. package実運用化 local/offline slice: **complete**
   - `package.rpxm` SHA-256、PKG005–PKG008、lock再現性
   - typed resource light + 自動materialize
   - offline local registry mirror
   - network registry、full-tree artifact hash、Core resource effectは別OPENとして継続
2. Direct Native v2: **next / planned**
   - [`direct-native-v2-plan.md`](direct-native-v2-plan.md) のDN2-0から順に実行
   - 最初のincrementはtyped callable基盤 + `length/units` dual-path
   - `package/module/export` をregistry keyとし、bind時に`BindingId`へ対応付ける
3. 製品級組版: font-backed JLReqとOpenType MATHを別エンジンとして本格化

Step 7はSteps 1–6のgreen HEADを基準線として開始する。
