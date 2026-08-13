# RPX 実装済み機能サマリ（第II部対比）

最終更新の根拠: `lang/part2-conformance-stats.json`、`lang/part2-deferred.md`、`lang/coverage-status.md`、`lang/language-kernel-plan.md`、`lang/package-plan.md`、および `crates/` / `packages/` 構成。

---

## 1. 目的と見方

この文書は、`lang/specification.md` **第II部**（言語基礎）に対し、**いま何が実装されていて、何が意図的に後回しか**を短く突き合わせるための索引である。

| 読み方 | 意味 |
|--------|------|
| **実装済** | カーネル／ホストとして実働し、適合レビュー上も主に `ok`（または義務のない `meta`） |
| **部分** | 動くが要求の一部のみ（適合 `partial`、または計画上の意図的ギャップ） |
| **延期** | レビュー済みで **`deferred`** — 「未発見の gap」ではなく、順序・依存・OPEN による後回し |

**正直な前提（重要）:**

- **言語は完成していない。** pre-PKG カーネルは実用に足りるが、仕様第II部の全見出しを満たす完全実装ではない。
- 適合 `ok` は「その見出しについて現状実装と矛盾しない／義務を満たす」判定であり、**仕様全体の閉包や製品完成を意味しない**。
- `deferred` が約半数近くある。優先度は `gap`/`partial` 埋めとは別枠（`part2-deferred.md`）。
- Rust 領域カバレッジの高さと、言語仕様の網羅は別物である。

関連リンク: [`part2-conformance.md`](part2-conformance.md) · [`part2-deferred.md`](part2-deferred.md) · [`coverage-status.md`](coverage-status.md) · [`package-plan.md`](package-plan.md) · [`language-kernel-plan.md`](language-kernel-plan.md)

---

## 2. 現状サマリ

### 2.1 第II部 適合カウンタ

出典: `lang/part2-conformance-stats.json`

| ステータス | 件数 | 割合（1589 見出し） |
|------------|-----:|--------------------:|
| **ok** | 767 | 48.3% |
| **partial** | 1 | 0.1% |
| **deferred** | 683 | 43.0% |
| **meta** | 138 | 8.7% |
| **gap** | 0 | 0% |
| **unchecked** | 0 | 0% |
| **total** | 1589 | 100% |

- 唯一の **`partial`**: `PKG-001` パッケージ manifest・依存解決・ワークスペース・リソース（Slice A–C まで到達；レジストリ等は後回し）。
- **`gap` ゼロ**は「未レビュー欠落がない／矛盾が残っていない」という意味であり、**未実装がないことではない**（その多くは `deferred`）。

### 2.2 `deferred` 分類

出典: `lang/part2-deferred.md`

| 分類 | 件数 |
|------|-----:|
| 意図的後回し | 366 |
| 依存待ち | 257 |
| 仕様未決定 | 60 |
| **合計** | **683** |

機能ブロック別の延期件数（多い順）: **EDT 126** · **MOD 110** · **PKG 109** · **ERR 86** · **MEM 70** · **TYP 70** · TST 22 · MAC 21 · DAT 20 · BND 14 · EFF 11 · SYN 9 · ほか少数（IR/EVAL/KER/RSC/LEX/ROW/ASY）。

### 2.3 Rust ワークスペース領域カバレッジ概況

出典: `lang/coverage-status.md`（llvm-cov region、BEST-ENTRY、7-crate impl `src/`）

- **Filtered BEST-ENTRY (7 crates):** **99.01%**（25614/25870; 256 missed）— **≥99% crossed** (N6 q); 目標はおおよそ 99%
- 残ギャップが大きいファイル: `cli` / `document_pipeline` / `load` / `eval` / `check` / `cast`；`pipeline` と `graphics_value` は **≥99%**；`parse` は **98.98%**
- `graphics_bridge` live-hash **100%**；`unify` / `resolve` / `outcome` は ≥99% または 100%

---

## 3. 言語機能マップ

pre-PKG カーネルは計画上 **COMPLETE（意図的 defer 付き）**。各行は「実装状況／一言／代表パス」の対照表。

| ブロック | 実装状況 | 一言 | 代表パス |
|----------|----------|------|----------|
| **LEX** | 実装済（OPEN 1件延期） | NFC・識別子・数値基数／科学記数／`_`、文字列（バックスラッシュなし・`"""`） | `crates/reciplexa-syntax`（lexer） |
| **SYN** | 実装済（葉・markup 等に延期あり） | S 式パーサ、`local`/`rec`、record-update/extend、`/` パス結合 | `crates/reciplexa-syntax` |
| **RES** | 実装済 | 字句束縛・BindingMap・NFC | `crates/reciplexa-core`（resolve） |
| **MAC** | 実装済（typed/外部公開は延期） | 衛生マクロ、`$params ...+`、展開マップ | `crates/reciplexa-macro` |
| **DAT** | 実装済（cyclic/一部 ADT 糖衣は延期） | `data`/`match`、リテラル・タプル・レコードパターン、静的網羅 | `crates/reciplexa-core` + `reciplexa-eval` |
| **TYP** | 実装済（深い拡張は延期多い） | Core 型検査、`Dynamic`/`Union` stub、EffectRow on Fun | `crates/reciplexa-core`（check） |
| **ROW** | 部分 | 閉じたレコード + OpenRecord／Lacks；multi-tail は延期 | `crates/reciplexa-core` |
| **EFF** | 実装済（multi-shot 等は延期） | deep one-shot handler、ambient `log`/`random`、`with`/`handler` | `crates/reciplexa-effect` + `reciplexa-eval` |
| **BND** | 実装済（完全 escape 行列は延期） | `letrec` / `var` / `set`、局所状態・生存 escape 拒却 | `crates/reciplexa-eval` / bind 連携 |
| **EVAL** | 実装済 | Core 評価、ホスト効果、継続の one-shot 経路 | `crates/reciplexa-eval` |
| **MOD** | 実装済（functor/sig は延期） | 外モジュール、`import` path/`as`/`only`/rename、qualified、`.rpi` 境界 | `crates/reciplexa-bind` |
| **KER** | 部分 | 算術比較ビルトイン + EffectHost；正式 ABI hash は OPEN | `crates/reciplexa-eval` / identity |
| **RSC** | 部分 | `MemoryFsHost` の file I/O；言語 `(resource …)` 軽量 deferred record；豊富な資源カタログは延期 | `crates/reciplexa-eval` / `reciplexa-core` |
| **MEM** | 実装済（拡張・一部 OPEN は延期） | Mem IR、record-update/extend 実行、lower | `crates/reciplexa-mem` + `reciplexa-lower` |
| **ERR** | 部分 | 構文診断・主要 abort；深い診断体系・PKG 連動は延期多数 | `crates/reciplexa-diagnostic` |
| **EDT** | 部分 | BindingId／展開ソースマップ、文書パイプライン；協調編集・codec policy は延期 | `crates/reciplexa-document` + macro map |
| **PKG** | **部分**（唯一の適合 `partial`） | ローカル `package.rpxm`、path dep、`rpx.lock`、import 解決；レジストリは延期 | `crates/reciplexa-package` + `packages/*` |
| **TST** | 部分 | 言語ゲート／適合試験の主要通過；版付きフル suite は延期 | `crates/reciplexa-test` + `lang_kernel` 系 |
| **IR / lower / scene** | 実装済（文書面はホスト連動） | Core→Mem→scene／visual-ir；図形は当面 interim + パッケージ二重化 | `crates/reciplexa-lower` / `scene` / `visual-ir` |

### 3.1 パッケージ Slice A（＋ B/C 薄層）— **実装は Rust native へ移行**

仕様上の単位・色・図形は **言語組み込みではない**（パッケージ API 供給）。ホットドメインの **本文は Rust native**（`KER-001` / Compiler-native package）。詳細単位: [`native-domain-plan.md`](native-domain-plan.md)。

| パッケージ | 状況 | 内容の目安 |
|------------|------|------------|
| **`packages/graphics`** | **native**（N1–N2） | `shapes` / `page` / `color`＋`.rpi`；例 `pkg_graphics_*` |
| **`packages/length`** | **native**（N1） | 単位コンストラクタ＋`.rpi`；例 `pkg_length.rpx` |
| **`packages/color`** | **native**（N1） | srgb / named＋`.rpi`；例 `pkg_color.rpx` |
| **`packages/math`** | **native**（N3）＋ **std deepen（M0–M3）** | atoms…stack＋`.rpi`；例 `pkg_math.rpx`；`reciplexa-std::math` に Accent/BigOp/Matrix/Aligned/Stack＋fontless `MathBox`；**glyph layout は後続** |
| **`packages/japanese`** | **native**（N4）＋ **std deepen（J0–J4）** | classes / linebreak / kihon / markup；`reciplexa-std::japanese` に classify / break / kihon / ruby；**完全 JLReq UCS ではない** |
| **`packages/document`** | **native scaffold**（N5.1–N5.2） | `page`＝flow/section/heading 等（`doc-*`）；例 `pkg_document.rpx`；pipeline 自動ルート済；**interim keyword 表は S6b で production から隔離（`interim-surface` / `cfg(test)`）** |
| **`crates/reciplexa-std`** | Rust ファサード（native 本文の主戦場） | `visual` / `text` / `document` / `math` / `japanese` / … |

ローカル import・path-dep・`workspace.rpxm` stub まで到達。**レジストリ等は OPEN-PKG-***。**N5.2 dual-path:** package 文書＋markup→graphics package emit＋GUI writable package sync（v2 S0–S6b）。interim keyword 表は fixture 向け `interim-surface` のみ。

---

## 4. Rust ホスト（CLI / GUI / PDF / SVG / PPTX）

言語クレート（syntax / core / eval / bind / macro / package / mem / effect / …）の上に、ホストが載る。

| ホスト | クレート | できること（現状） |
|--------|----------|-------------------|
| **CLI** | `reciplexa` | `parse` / `format` / `inspect-syntax` / `inspect-document` / `eval`；および `input.rpx → output.{pdf\|svg\|pptx}` エクスポート |
| **GUI** | `reciplexa-gui` (+ `gui-runtime`, `view`) | egui 紙面プレビュー（M6 view 寄り）；PDF/SVG/PPTX 連携、ドラッグ等の編集は仕様 EDT 全体には未到達 |
| **PDF** | `reciplexa-pdf` | scene からの最小 PDF（フォント／画像経路あり；CJK は環境依存） |
| **SVG** | `reciplexa-svg` | scene → SVG（GUI なし） |
| **PPTX** | `reciplexa-pptx` | scene → 最小 OOXML PPTX |
| **補助** | `document` / `scene` / `view` / `backend` / `codec` / `std` / `motion` / `video` / `raster` … | 文書モデル、ビュー投影、型付き std ファサード、実験的モーション／映像など（ドメイン仕様の完全移植ではない） |

ホストは **「言語カーネルで評価・下部変換した文書を見える／書き出す」** 段階であり、仕様第III部以降のランタイム・協調編集・完全パッケージ配布のホストではない。

---

## 5. 仕様にあって未実装・延期の目立つもの

`deferred` の主要塊（件数は機能ブロック集計。詳細タイトルは `part2-deferred.md`）。

1. **EDT（≈126）** — 協調編集、codec override、Provenance 分類、逆編集・エクスポートポリシーなど。多くは **依存待ち（Part III 寄り）** + OPEN。
2. **MOD functor / signature（MOD deferred ≈110 の中核）** — outer + import は実装済；**フル ML ファンクタ・シグネチャ・再帰／一等モジュールは v1 外**。
3. **PKG（≈109 + 適合 `partial`）** — ローカル Slice A–C 以外: **レジストリ（OPEN-PKG-001）**、feature、nested workspace 本番、資源ルート本番、ネイティブ package 差し替え、多数の PKG 適合試験見出し。
4. **ERR（≈86）** — 深い診断・エントリ点・ランタイム失敗分類。多くが **ERR/MEM/CON 依存待ち**。
5. **MEM / TYP（各 ≈70）** — Mem 拡張・OPEN-MEM；型側は厳密 positivity 以外・表形式対応表・深い ModuleEnv 型など **意図的後回しが多い**。
6. **EFF multi-shot / shallow / return 句** — deep one-shot + ambient + `with` まで。**multi-shot・shallow choice・完全 Handler 型付けは意図的後回し**。
7. **MAC OPEN 群** — typed macro、正式 identity、パッケージ外公開、CAP 等（仕様未決定）。
8. **DAT OPEN-GRAPH** — 循環値・共有の非観測・無限構造。
9. **ROW multi-tail / unrestricted tallying**。
10. **単位・色の言語節（SYN 周辺 deferred）** — パッケージ側に移譲済み方針；lexer の Number+Ident は **暫定**。
11. **TST / 適合試験スイート見出し** — 主要通過以外の版付きフルゲート・多数葉。

これらは「バグ一覧」ではなく、**ロードマップ上まだ追わない契約**である。

---

## 6. 次工程

推奨順序:

1. **Native domain 移行** — [`native-domain-plan.md`](native-domain-plan.md) N0→N5.2 dual-path **done**（length/color → graphics → math → japanese → document）。keyword 表 production 隔離は GUI CST sync v2 S6b **done**。
2. **カバレッジ締め（N6）** — **done**（7-crate BEST-ENTRY ≥99%）；N5.3 tip for document surface residuals。
3. **GUI CST sync v2** — [`gui-cst-sync-v2-plan.md`](gui-cst-sync-v2-plan.md) S0–**S6b done**（package writable sync、examples package 化、production keyword 表隔離）。
4. **PKG インフラ（Slice E）** — **done**（workspace lock / resources / OPEN-PKG-001 stub）；package API `resolve_package_resource`（R0）**done**。言語 `(resource "rel")` 軽量面 **done**（elaborate→`package-resource` タグ付き record；package root 無しでは deferred note）。host `materialize_package_resource` / `resolve_resource_value`（R3）**done**。実 `package-resource` 型・load 時自動解決・レジストリはなお **OPEN**。
5. **OPEN-TEXT-JA-001 / math layout** — 第一深化スライス **done**（[`ja-math-deepen-plan.md`](ja-math-deepen-plan.md) J0–J7 / M0–M3）。追加: eval `math_value` 橋（accent/matrix/bigop/stack/aligned 含む）**done**；JA classify 表意約物・全角拡張 **done**（なお subset）。残: 完全 UCS・§C 禁則・ruby/縦中横実レイアウト・OpenType MATH / stretchy / 文書パイプライン本消費・`classify` 言語 intrinsic（BuiltinOp は未配線）。
6. **次の優先候補** — レジストリ（OPEN-PKG-001）／文書パイプラインが math box・JA classify を本番レイアウトに載せる／package load 時 resource 自動 materialize／`classify-char` 言語ビルトイン（任意）。
---

## 付記: クレート一覧の見取り図

**言語パイプライン寄り:** `syntax` · `core` · `macro` · `eval` · `bind` · `package` · `mem` · `effect` · `lower` · `ir` · `diagnostic` · `source` · `identity` · `types` · `outcome` · `test` · `proof` · `harden` · `native` · `opt` · `runtime`

**文書・ホスト寄り:** `document` · `scene` · `view` · `visual-ir` · `backend` · `codec` · `std` · `gui` · `gui-runtime` · `pdf` · `svg` · `pptx` · `raster` · `motion` · `video` · ルート `reciplexa`（CLI）

---

*このファイルは適合台帳の代替ではない。見出し単位の正は常に `part2-conformance.md` / stats JSON を参照すること。*
