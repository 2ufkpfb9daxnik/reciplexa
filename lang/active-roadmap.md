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

**Status:** active

- 人間の入口を `lang/README.md` に一本化する
- 専門AI向け実行プロンプトと常時ルールを整備する
- atomic commitとcommit前checkを固定する

**Complete when:** 新しいAIが `lang/README.md` から、追加説明なしで次のStepとgateを特定できる。

## Step 1 — 品質gate復旧

**Status:** planned

- language resolverのgraphics quarantineをS6b surface gateから分離する
- Japanese paragraph配置引数を `ParagraphSceneLayout` にまとめる
- fmt、workspace Clippy、workspace test、all-target check、GUI checkをgreenにする

**Complete when:** 全gateが同じHEADで成功し、実測結果が現状正本に記録される。

## Step 2 — 文書正本化

**Status:** planned

- `implemented-features.md` を唯一の現状正本にする
- 未完契約と有効な受入条件を現行文書へ移す
- 移行済みの古い完了計画を削除し、残存リンクを直す

**Complete when:** 人間は `lang/README.md`、AIは本roadmapと実行プロンプトだけで開始でき、削除済み文書へのリンクがない。

## Step 3 — Hybrid Native v1 / Direct Native v2

**Status:** planned

- 現行をHybrid Native v1として正確に記述する
- Direct Native v2を必須将来マイルストーンとして登録する
- portable fallback契約を勝手に削除せずOPENとして追跡する

**Complete when:** Hybrid v1を完全direct nativeと誤読できず、DN2の機械的な完了条件が仕様・roadmap・適合台帳から辿れる。

## Step 4 — package経路の型安全性

**Status:** planned

- packageソースと共存できるtop-level effect検査を追加する
- package elaborate後、eval前にCore型推論する
- preview、snapshot、exportの各package経路へ接続する
- load=`package`、静的型不一致=`type`、eval/bridge=`package` のstageを保つ

**Complete when:** 正常packageが動き、型不一致とeffect不一致が評価前に構造化エラーとなり、未知importと非page mainも適切なstageで失敗する。

## Step 5 — 編集可能な1ページ

**Status:** planned

- 対象: package形式1ページのcircle、line、text等の基本graphics
- package ASTのinsert、delete、reorder、text更新を実装する
- GUIのInsert/Delete/Paste/Z-order/Propertiesをpackage経路へ接続する
- move、resize、保存、再読込、PDF/SVG出力をE2Eで固定する

**Non-goals:** markup作者同期、製品級JA/math、doc-*作者編集。

**Complete when:** `examples/text_line.rpx` 相当をGUI操作相当APIで編集し、保存・再読込後のsceneとPDF/SVGに編集結果が残り、interim構文が混入しない。

## Step 6 — 完了状態の再計測

**Status:** planned

- 全gateとVertical Slice E2Eを再実行する
- `implemented-features.md` に保証範囲と非対象を記録する
- 長期 `roadmap.md` のPhase 4へ受入条件を対応付ける
- coverageやconformance分類を製品完成率として扱わない

**Complete when:** 同じHEADの実装、テスト結果、現状正本、長期roadmapが一致する。

## Step 7 — 次工程

**Status:** blocked by Steps 1–6

順序:

1. package実運用化: registry、暗号学的content hash、lock再現性、typed resource、自動materialize
2. Direct Native v2: BindingIdから型付きRust callableへ段階移行
3. 製品級組版: font-backed JLReqとOpenType MATHを別エンジンとして本格化

Step 7はSteps 1–6のgreen HEADを基準線として開始する。
