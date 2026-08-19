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

**Status:** complete (`560f5bd` item 3 workspace gates; item 1–2b earlier)

順序:

1. package実運用化 local/offline slice: **complete**
   - `package.rpxm` SHA-256、PKG005–PKG008、lock再現性
   - typed resource light + 自動materialize
   - offline local registry mirror
   - network registry、full-tree artifact hash、Core resource effectは別OPENとして継続
2. Direct Native v2: **complete** (`57df856` DN2-6, `5b669c9` DN2-7, `27d0946` 本番 stub BindingId bind)
   - [`direct-native-v2-plan.md`](direct-native-v2-plan.md) DN2-0〜DN2-7完了
   - 標準packageは DN2 stub + compilation BindingId（`dn2bid-*` extra env を `op_for` で供給）+ typed Rust callable
   - alias (`g/shapes`) と bare (`import graphics`) も canonical native path で stub elaboration を省略
   - Hybrid 参照本文は `LocalPackageIndex` override（差分試験のみ）
   - portable fallback は `OPEN-NATIVE-PKG-001` で明示追跡（未実装）
2b. DN2 観測同値の硬化: **complete**（製品級組版の前）
   - 既存 example による深い hybrid↔DN2 差分
   - `japanese/linebreak` の動的 export（kind 一致。note / 全matrix は std 正本）
   - 構造化 Failure: DN2 arity を `package/arity` へ。type は `length/units` に加え `color/srgb`・`japanese/*` の契約違反も `package/type`。effect row はまだ pure
   - native stub の body elaboration 省略（typed export から BindingId スロット Core を合成。alias / bare import を含む。Hybrid override は従来の RPX elaborate）
3. 製品級組版: **complete** ([`product-typesetting-plan.md`](product-typesetting-plan.md) Profile v1; gates on `560f5bd`)
   - JLReq / MATH Profile v1 エンジンは `reciplexa-text-layout`。ライブラリ `document_from_package_entry` / `layout_doc_page_to_scene` / `layout_math_to_shapes` は stub 参照
   - ホスト `document_from_package_source_host` は `doc-page` / `live-layout-demo` / `math-demo` / graphics `text` が product `GlyphRun`（graphics は authoring 1 ノード = 1 cluster run）
   - ホスト PDF は layout GID を Identity-H で塗り、digest ごとに CID face を subset（JA `host_product_font` + MATH `host_math_font`）。digest 不一致は relayout。Latin-only `Text` は Helvetica
   - GUI preview は同じ digest の egui family で `GlyphRun` を layout advance 位置に置く
   - `RECIPLEXA_TYPESET_ENGINE=stub` で参照ヒューリスティックを強制
   - Profile v1 完了後も OPEN: ruby/`vert` / 完全 MATH assembly / first-class editable math / `OPEN-NATIVE-PKG-001`（[`implemented-features.md`](implemented-features.md)）

**Complete when:** items 1–3 の受入と、item 3 の workspace gate が同じ HEAD で記録される。 → HEAD `560f5bd`

新しい Step は本文書へ人間が追加するまで着手しない。

Step 7はSteps 1–6のgreen HEADを基準線として開始した。
