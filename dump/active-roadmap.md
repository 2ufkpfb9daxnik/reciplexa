# Reciplexa Active Roadmap

**Authority:** 現在の実行順と完了条件の正本。専門AIは最初の未完了Stepから着手する。  
**Disk:** 成果物は D: のみ。C: に Cargo/temp を置かない。[`windows-disk-policy.md`](windows-disk-policy.md) / [`README.md`](README.md) 先頭。  
**Status source:** 実装済み機能と実測gateは [`implemented-features.md`](implemented-features.md) を正とする。  
**Normative source:** 契約を変更するときは [`specification.md`](specification.md) を同時に確認する。

## 状態の意味

- `planned`: 未着手
- `active`: 作業中
- `blocked`: 外部判断または先行Step待ち
- `complete`: 受入条件、実測gate、独立監査を同一HEADで満たした

`unit complete`、`stub complete`、`spec conformant`、`product slice complete` は別の状態である。単に `done` と書かない。

## Step完了の判定

Stepを`complete`とするには、次を**同一code HEAD**で揃える。

1. **実装と直接test** — 受入条件を満たす変更と回帰testが入っている。
2. **実測gate** — 対象crate test、fmt/clippy、必要なhost `cargo run`、Stepで定めたworkspace gateがgreen。
3. **現状正本** — [`implemented-features.md`](implemented-features.md) に実測gate表（HEAD、コマンド、結果）を記録する。
4. **独立監査（subagent）** — 実装担当と**別セッション**の独立 subagent（**Composer 2.5**；Cursor 内蔵 Task、`fast` モード不可、**外部 API 監査モデルは使わない**）が、
   文書・受入条件・test・gate・OPEN/non-goals・doc/code不一致を捜査し、
   `complete` / `partially complete` / `not complete` を報告する。
5. **人間確認** — Stepまたはsliceで要求されているGUI/目視確認を実施し、結果を1行でもよいので記録する。

独立監査が `partially complete` または `not complete` のときは `complete` にしない。
実装は着地していても、workspace gate 未達、gate未記録、人間確認未記録、doc/code不一致が残る場合は
`landed` または `active` のままとする。

監査subagentは実装変更を行わない。model は **Composer 2.5** に固定する（外部 API 監査モデルは使わない）。修正は実装担当が別commitで行い、監査を再実行する。

## Slice完了の判定（macro-Step内）

詳細planでsliceに分解している Step（例: Step 10、Step 11）では、**次のsliceへ進む前に**次を同一 code HEAD で揃える。

1. **実装と直接test** — 当該sliceの受入条件とnon-goalsを満たす。
2. **scoped gate** — 対象crateの test（＋変更があれば fmt/clippy）。
3. **独立監査（subagent）** — 実装担当と**別セッション**の独立 subagent 1体（**Composer 2.5**；`fast` 不可、外部 API 監査モデル不可）が当該sliceのみを捜査し、`complete` / `partially complete` / `not complete` を報告する。
4. **人間確認** — sliceまたはStepで要求されている場合のみ記録。

監査が `complete` でないときは次sliceに進まない。詳細planの audit 表に HEAD・subagent ID・判定を追記する。

macro-Step を `complete` とするには、上記を**全slice**で満たしたうえで、Step完了の判定 2–5（workspace gate、`implemented-features.md` gate 表、Step 全体の独立監査再確認、人間 GUI）を同一 HEAD で揃える。

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

**Status:** complete (item 3 Profile v1 on code HEAD `3087565`; item 1–2b earlier)

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
3. 製品級組版: **complete** ([`product-typesetting-plan.md`](product-typesetting-plan.md) Profile v1; workspace gates on `3087565`)
   - JLReq / MATH Profile v1 エンジンは `reciplexa-text-layout`。ライブラリ `document_from_package_entry` / `layout_doc_page_to_scene` / `layout_math_to_shapes` は stub 参照
   - ホスト `document_from_package_source_host` は `doc-page` / `live-layout-demo` / `math-demo` / graphics `text` が product `GlyphRun`（graphics は authoring 1 ノード = 1 cluster run）
   - ホスト PDF は layout GID を Identity-H で塗り、digest ごとに CID face を subset（JA `host_product_font` + MATH `host_math_font`）。digest 不一致は relayout。Latin-only `Text` は Helvetica
   - GUI preview は同じ digest の egui family で `GlyphRun` を layout advance 位置に置く
   - `RECIPLEXA_TYPESET_ENGINE=stub` で参照ヒューリスティックを強制
   - page 形 example は preview/export。`pkg_japanese_jlreq` / `pkg_math_spacing` は language-demo（eval + engine coverage）
   - Profile v1 完了後も OPEN（実装しない）: `OPEN-TEXT-LAYOUT-001` 残 bullets、ruby/`vert`、完全 MATH assembly、first-class editable math（[`implemented-features.md`](implemented-features.md)）

**Complete when:** items 1–3 の受入と、item 3 の workspace gate が同じコード HEAD で記録される。 → `3087565`

Step 7はSteps 1–6のgreen HEADを基準線として開始した。

## Step 8 — JLReq follow-ups

**Status:** complete (items 1–3; 実装 `caa2afd`; gate / 文書 `52d088e`; 人間 GUI 2026-08-20)

人間が Vertical Slice の GUI 確認を済ませたあとの次工程。[`jlreq-followup-plan.md`](jlreq-followup-plan.md)

順序:

1. Font-backed simple ruby: **complete** (`layout_simple_ruby`; host `ja-ruby-demo` / `placed` / `samples`)
   - 横組の simple ruby を font advance で測り、annotation を親 em の真上に置く
   - `examples/pkg_ruby.rpx` は見出し＋3見本。`japanese/markup ruby` と `Ruby::estimate_box` stub は変えない
   - jukugo 配分、縦ルビ、overhang、`vert`、縦中横、傍点は当時の非ゴール
2. OpenType `vert` / 縦組: **complete** (`layout_vertical_run`; host `pkg_vert.rpx` on `9768c0d`)
   - 縦組みは上から下、次の列は左（vertical-rl）
   - CJK は upright（GSUB `vert` GID）。ASCII は −90°
   - `lines_to_vertical_text_shapes` stub と `pkg_japanese_vertical` 言語デモは変えない
   - 縦中横、傍点、縦ルビは当時の非ゴール
3. Tate-chu-yoko / bou: **complete** (`layout_tate_chu_yoko`; bou circles; host `ja-tcy-bou-demo` / `examples/pkg_tcy_bou.rpx` on `caa2afd`)
   - 縦組の1emセルに ASCII 1–4 字を正立で横並び（幅が1em超なら縮小）
   - 傍点は横組では文字の上、縦組では横に円を置く。stub `bou-box` / `tate-chu-yoko-width` は変えない
   - CSS `text-emphasis` / sesame GID / 4字超の圧縮は非ゴール
   - 人間 GUI 確認: `令和12年` と傍点が紙面内に見え、点が親 em の外に付く

**Complete when:** items 1–3 の受入を満たし、残 follow-up が OPEN のまま列挙され、§2.4.3 gate が green、
人間 GUI 確認が記録される。 → `52d088e`（実装 `caa2afd` + gate repair + 人間 GUI 2026-08-20）

**人間 GUI 確認（2026-08-20）:** `pkg_ruby` / `pkg_tcy_bou` OK。`pkg_vert`: 縦積み・ABC 回転上→下 OK。`。` `、` `ー` の縦組位置は Step 12 へ（Step 8 非ゴール）。

残 OPEN（この Step では実装しない）: jukugo 配分、縦ルビ、ruby overhang、完全 `vrt2` / `text-orientation` / CSS `text-emphasis`、**縦組句読点・長音の JLReq 位置調整**（`。` `、` `ー` 等。Step 8 item 2 は GSUB `vert` + vmtx 積みのみ）。これらはStep 11–12へ依存順に登録したため、Step 8の暗黙の続きとして着手しない。

## Step 9以降のmacro-Step運用

長期の目的、依存、旧Phase / Milestone、OPEN対応は[`roadmap.md`](roadmap.md) §0.1–0.5を正とする。
Step番号の過剰増殖を防ぐため、以下を固定する。

- top-level Stepは利用者能力または基盤不変条件で区切る。
- bugfix、個別follow-up、fixture、並行作業はStep内sliceとし、新しいStep番号を付けない。
- OPEN IDは自動的に実行項目にしない。該当macro-Stepのscopeへ明示的に取り込む。
- macro-Stepを`active`にする前に、詳細planでslice順、受入条件、non-goals、gateを固定する。
- 同時に`active`とするmacro-Stepは一つだけとする。
- 各Stepの完了時に、本書、`implemented-features.md`、詳細plan、実測gate、独立監査結果を同じHEADへ同期する。
- `complete`判定は上記「Step完了の判定」を満たすまで行わない。限定testだけ、または実装担当の自己申告だけでは足りない。

完成は二段階で判定する。

- **Horizon A:** Step 17 — 実用的な日本語組版・数式編集・配布製品
- **Horizon B:** Step 24 — 仕様第II–VI部を含む完全実装

Horizon AをPart II deferredゼロや形式的証明完了と混同しない。

## Step 9 — 構造化ページ作者編集とProvenance

**Status:** complete（gate §2.4.4; [`step9-document-authoring-plan.md`](step9-document-authoring-plan.md)）

- `document/page`のheading / paragraph / indent / columnsを作者sourceの構造を保ってGUI編集する
- literal / shared binding / GUI override / 逆変換不能を区別する
- scene nodeからSource span / authoring nodeへのProvenanceを固定する
- markup作者同期は理由付きsoft-refuseのまま維持する

**Landed:** `sync/document.rs`（layer 収集 + text / indent-em / columns 編集）、GUI layer pane + props、doc-page canvas nudge soft-refuse、`document_snapshot_from_source` provenance、E2E（`pkg_document_indent.rpx` / `pkg_columns.rpx` save/reload/PDF/SVG）。

**Complete when:** 構造化ページをGUI編集し、Source保存・再読込・PDF/SVGへ意味を保って往復でき、
transaction failureが部分変更を残さず、逆変換不能を黙って低水準化しない。✓

## Step 10 — Editor persistence v1

**Status:** complete（gate §2.4.5; [`step10-editor-persistence-plan.md`](step10-editor-persistence-plan.md)）

- Undo/Redo、atomic save、crash recovery、snapshot compaction
- schema version、migration graph、unknown extension、partial recovery
- Stable IDを保つsave/loadと、IME / caret / selection rebase

**Landed:** `atomic_write` primary save、`.rpjsrc` edit journal + open recovery prompt、`.rpxsnap` journal/recovery、`AuthoringUndo` + `RevisionUndoLog`、`rebase.rs` caret/selection、`sidecar.rs` migration/partial recovery。

**Complete when:** transaction履歴と復旧が文書正本を壊さず、GUI state、Capability、Secret、
Task、native pointerをsnapshotへ混入させない。✓

## Step 11 — Text shaping / font resource v2

**Status:** complete（gate §2.4.6; [`step11-text-shaping-plan.md`](step11-text-shaping-plan.md)）

- typed layout protocol、決定的font fallback、cluster保持
- metric-changing substitution時のrelayout
- complex shaping、bidi、mixed direction

**Slice 1 landed:** `protocol` + `FontFallbackChain` + per-glyph `FontId`（監査 `895bf444` / `956d253` **complete**）。

**Slice 2 landed:** `bidi.rs` + `from_shaped_run_with_attrs` + mixed JA/RTL gates（監査 `4defb956` / `7b922a0` **complete**；fmt 追従 commit 後）。

**Slice 3 landed:** `rustybuzz` + `complex.rs` + `require_emit_matches_shaped_run`（監査 `5bfa267c` / `3850f59` **complete**）。

**Slice 4 landed:** host product wiring（監査 `b7a50fbd` / `b29ad06` **complete**）。

**Slice 5 landed:** workspace gate + fallback export fixture + §2.4.6。

**Complete when:** fallback / bidi fixtureでUnicodeとcluster対応を失わず、previewとexportが同じ
glyph位置を消費し、`OPEN-TEXT-LAYOUT-001`の製品必須範囲が閉じる。✓

## Step 12 — Japanese Document Profile v2

**Status:** active — [`step12-ja-profile-plan.md`](step12-ja-profile-plan.md) slice 5 landed；macro-Step 監査待ち

- 縦ルビ
- より完全な`vrt2` / 文字方向処理とJLReq rule vector
- stub differentialと残存appendix C / CSS範囲の明示

**Slice 1:** complete — vertical punctuation JLReq position（`。` `、` `ー`、縦組ラテン）。

**Slice 2:** complete — ruby overhang（横組 simple ruby の親文字送り / measure overhang）。

**Slice 3:** complete — jukugo ruby distribution（per-base annotation placement）。

**Slice 4:** complete — vertical ruby（font-backed side annotation in vertical-rl columns）。

**Slice 5:** complete — combined Japanese Profile v2 page（`pkg_ja_profile_v2.rpx` + `step12_gates` + §2.4.11）。macro-Step 独立監査待ち。

**Complete when:** 横組・縦組・ruby・縦中横・傍点を組み合わせた文書がfont-backedで
preview/exportされ、実装規則と適合fixtureが対応する。

## Step 13 — Math Profile v2とGUI編集

**Status:** blocked — Step 12待ち（slice並行可）

- full glyph assembly、MATH kern、font metrics
- alignment / equation numberの実用subset
- first-class math tree GUI編集

**Complete when:** 数式構造をGUIで変更しSourceへ戻せ、unsupported assemblyやASCII代用を
silent fallbackせず、文書内inline/display mathを安定して出力できる。

## Step 14 — 複数ページ出版レイアウト

**Status:** blocked — Steps 12–13待ち

- page break、版面、heading / list / note、figure / table
- **階層レイヤー / テキストボックス一括移動**（authoring 1 ノードを親行にし、縦組の per-glyph `GlyphRun` を子にする。Step 21 から前倒し。Step 12 非ゴール）
- footnote / reference / indexの最小一貫経路
- 複数ページのidentity、参照、部分relayout

**Complete when:** 日本語・数式・図表を含む記事 / 小冊子をGUI編集、save/load、
preview/exportでき、部分relayoutがfull rebuildと一致する。

## Step 15 — Export / print / accessibility parity

**Status:** blocked — Step 14待ち

- PDF / SVG / PPTXのfont / `GlyphRun`位置とresource policy統一
- Output Profile、LossReport、Artifact Verification
- reading order、heading、math accessibilityとprint宣言

**Complete when:** profile不一致をplanning時に拒否し、全backendがplan外fallbackを行わず、
宣言したaccessibility / print条件を検証できる。

## Step 16 — Package配布とNative portability

**Status:** blocked — Step 15待ち

- network registry、full-tree artifact hash、cache / lock再現性
- Core resource effectとbracket cleanup
- ABI/version negotiation、native-unavailable routing、portable fallback

**Complete when:** package取得とnative fallbackをartifact identityに基づき再現でき、
network機能追加後もoffline build/testが常にgreenである。

## Step 17 — Product beta / release gate

**Status:** blocked — Step 16待ち

- 日本語記事とmath showcaseの統合E2E
- product diagnostic、Loss approval、resource/package UX
- reproducible release、fuzz/decode budget、human GUI check

**Complete when:** GUI編集、save/load、package取得、PDF/SVG/PPTX出力を通常の製品経路で完走し、
full gateと人間確認が同一HEADでgreen。ここを**Horizon A complete**とする。

## Step 18 — Structured runtime / failure boundary

**Status:** blocked — Horizon A待ち

- Effect Lowering、Task scope、Cancellation、bracket cleanup
- deterministic scheduler、Failure / Defect / Diagnostic lifecycle

**Complete when:** 子Taskを残さず、全終了pathでcleanupし、Failure / Cancellation / Defectを
混同せず、test scheduleを再現できる。

## Step 19 — Perceus / ownership / resource safety

**Status:** blocked — Step 18待ち

- RC挿入、reuse analysis、borrow / liveness
- Ownership / Reuse Verifierと参照Evaluator差分

**Complete when:** leak、double release、use-after-move、continuation / cancellation境界の
所有権違反を拒否し、最適化前後が観測同値である。

## Step 20 — Layered IR / incremental pipeline

**Status:** blocked — Step 19待ち

- Domain / Layout / Visual / Compositing / Render / Backend Planning IR
- incremental compile / eval / render、cache invalidation、Provenance Map

**Complete when:** incremental resultがfull rebuildと同値で、stale commitを拒否し、
page / frame / tile単位のbudgeted cacheを検証できる。

## Step 21 — Full GUI runtime / collaboration

**Status:** blocked — Step 20待ち

- Stable Key reconciliation、lifecycle、focus / gesture / accessibility
- codec、collaboration、override policy

**Complete when:** reorder / replaceでstate ownerを誤らず、複数viewと共同transactionを安全に統合し、
Part V / EDTの対応OPENを適合試験で閉じる。

## Step 22 — Language / module / package depth

**Status:** blocked — Step 21待ち

- MOD signature / functor、TYP / ROW、advanced EFF
- typed / package macro、DAT graph、PKG feature / entry

**Complete when:** 各trancheが構文、型、Failure、identity、適合testを揃え、該当`deferred`を
`ok`または承認済み仕様改訂へ移せる。

## Step 23 — Domain expansion

**Status:** blocked — Step 22待ち

- slide / theme / master
- vector / path / gradient / constraint
- motion / timeline / video

**Complete when:** slide deck、vector artwork、motion titleが同じDocument identity / transaction /
Output Profile上でsource・GUI往復、save/load、backend出力を通る。

## Step 24 — Conformance / hardening / proof closure

**Status:** blocked — Step 23待ち

- Part II残存`deferred` / `partial` / OPENの実装または仕様改訂
- Part III–VI conformance、property test、fuzz、security / privacy / capability audit
- 型安全性、Effect、主要lowering、transaction、incremental同値の必要な証明

**Complete when:** 仕様の背景、目的、外部契約、Failure、Lifetime、Diagnostic、testが実装と一致し、
残存項目が暗黙の未実装でなく明示的に閉じられる。ここを**Horizon B complete**とする。
