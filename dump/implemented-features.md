# RPX 実装済み機能サマリ（現状正本）

**Authority:** 現在動くもの・現在のgate・OPEN事項の唯一の正本。実行順は [`active-roadmap.md`](active-roadmap.md)。規範は [`specification.md`](specification.md)。

最終更新の根拠: 同じHEADの実装、`dump/part2-conformance-stats.json`、`dump/part2-deferred.md`、`dump/coverage-status.md`、`dump/package-plan.md`、および `crates/` / `packages/` 構成。

### 製品ラベル（混ぜない）

| ラベル | 意味 |
|--------|------|
| **unit complete** | 対象ユニットのテストが通る |
| **stub complete** | 意図したスタブ境界まで到達（本番エンジンではない） |
| **spec conformant** | 当該見出しが適合台帳上 `ok`（製品完成ではない） |
| **product slice complete** | Vertical Slice の受入条件を満たす |

`gap=0`、`ok`件数、限定coverageを製品完成率として扱わない。

現行標準packageは **Direct Native v2**（package API + DN2 stub + typed Rust callable）。Hybrid v1 合成RPXは差分試験の参照のみ。portable fallback は `OPEN-NATIVE-PKG-001`。詳細は [`direct-native-v2-plan.md`](direct-native-v2-plan.md)。

---

## 1. 目的と見方

この文書は、`dump/specification.md` **第II部**（言語基礎）に対し、**いま何が実装されていて、何が意図的に後回しか**を短く突き合わせるための索引であり、現状の唯一の正本である。

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

関連リンク: [`part2-conformance.md`](part2-conformance.md) · [`part2-deferred.md`](part2-deferred.md) · [`coverage-status.md`](coverage-status.md) · [`package-plan.md`](package-plan.md) · [`direct-native-v2-plan.md`](direct-native-v2-plan.md) · [`active-roadmap.md`](active-roadmap.md) · [`roadmap.md`](roadmap.md)

---

## 2. 現状サマリ

### 2.1 第II部 適合カウンタ

出典: `dump/part2-conformance-stats.json`

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

出典: `dump/part2-deferred.md`

| 分類 | 件数 |
|------|-----:|
| 意図的後回し | 366 |
| 依存待ち | 257 |
| 仕様未決定 | 60 |
| **合計** | **683** |

機能ブロック別の延期件数（多い順）: **EDT 126** · **MOD 110** · **PKG 109** · **ERR 86** · **MEM 70** · **TYP 70** · TST 22 · MAC 21 · DAT 20 · BND 14 · EFF 11 · SYN 9 · ほか少数（IR/EVAL/KER/RSC/LEX/ROW/ASY）。

### 2.3 Rust ワークスペース領域カバレッジ概況

出典: `dump/coverage-status.md`（llvm-cov region、BEST-ENTRY、7-crate impl `src/`）

- **Filtered BEST-ENTRY (7 crates):** **99.01%**（25614/25870; 256 missed）— **≥99% crossed** (N6 q); 目標はおおよそ 99%
- 残ギャップが大きいファイル: `cli` / `document_pipeline` / `load` / `eval` / `check` / `cast`；`pipeline` と `graphics_value` は **≥99%**；`parse` は **98.98%**
- `graphics_bridge` live-hash **100%**；`unify` / `resolve` / `outcome` は ≥99% または 100%

### 2.4 実測gate（HEAD `5d2c5bf`）

Env D / offline。`CARGO_TARGET_DIR=d:\reciplexa\target`、`TEMP`/`TMP=d:\reciplexa\.tmp`。

| Command | 結果 |
|---------|------|
| `cargo fmt --all --check` | green |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | green |
| `cargo test --offline -p reciplexa-lower --features interim-surface` | green |
| `cargo test --offline -p reciplexa-types --features interim-surface` | green |
| `cargo test --offline -p reciplexa-bind --features interim-surface` | green |
| `cargo test --workspace --offline` | green |
| `cargo check --workspace --all-targets --offline` | green |
| `cargo check --offline -p reciplexa-gui` | green |
| `cargo test --offline -p reciplexa-gui --test vertical_slice_e2e` | green |

### 2.4.1 Direct Native v2 完了gate

Env D / offline。DN2 本番 stub bind・Hybrid 参照分離・型/Failure 差分試験を含む同一作業木。`vertical_slice_e2e` は `cargo test --workspace` 内で green。

| Command | 結果 |
|---------|------|
| `cargo fmt --all --check` | green |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | green |
| `cargo test --workspace --offline` | green |
| `cargo check --workspace --all-targets --offline` | green |
| `cargo check --offline -p reciplexa-gui` | green |
| `cargo run --offline -p reciplexa-gui -- --smoke examples/text_line.rpx` | green (`smoke: ok`) |
| `cargo run --offline -p reciplexa -- examples/text_line.rpx .tmp/dn2-complete-smoke.pdf` | green |

### 2.4.2 製品級組版 Profile v1 完了gate（code HEAD `3087565`）

Env D / offline。JLReq / MATH Profile v1、ホスト GlyphRun / PDF GID、GUI digest face + layout advances、`pkg_live_math` origin は `text_y_mm`（GlyphRun 含む）。`vertical_slice_e2e` は `cargo test --workspace` 内で green。この HEAD のあと docs だけが完了を記録する（DN2 と同じ）。ruby/`vert` / 完全 assembly は閉じない。

| Command | 結果 |
|---------|------|
| `cargo fmt --all --check` | green |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | green |
| `cargo test --workspace --offline` | green |
| `cargo check --workspace --all-targets --offline` | green |
| `cargo check --offline -p reciplexa-gui` | green |
| `cargo run --offline -p reciplexa-gui -- --smoke examples/text_line.rpx` | green (`smoke: ok`) |
| `cargo run --offline -p reciplexa -- examples/text_line.rpx .tmp/pt-item3-gates.pdf` | green |

### 2.4.3 Step 8 JLReq follow-up 完了gate（code HEAD `52d088e`）

Env D / offline。Step 8 items 1–3（simple ruby、`vert` 縦組、縦中横 / 傍点）。jukugo / 縦ルビ / overhang / 完全 appendix C / **縦組句読点の JLReq 位置調整**は閉じない。
`pt9_pkg_live_math_product_keeps_delimiter_and_fraction` は JA `GlyphRun` baseline からの math origin（`ja_min - 12mm`）を検証する。

| Command | 結果 |
|---------|------|
| `cargo fmt --all --check` | green |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | green |
| `cargo test --workspace --offline` | green |
| `cargo check --workspace --all-targets --offline` | green |
| `cargo check --offline -p reciplexa-gui` | green |
| `cargo run --offline -p reciplexa-gui -- --smoke examples/text_line.rpx` | green (`smoke: ok`) |
| `cargo run --offline -p reciplexa -- examples/text_line.rpx .tmp/step8-gate-smoke.pdf` | green |
| `cargo test -p reciplexa-text-layout --test ruby_gates --test vert_gates --test tcy_bou_gates --offline` | 18 passed |
| `cargo test -p reciplexa-package --test pkg_ruby_tests --test pkg_vert_tests --test pkg_tcy_bou_tests --offline` | 6 passed |

**人間 GUI 確認（2026-08-20）:** OK（`pkg_ruby.rpx` simple ruby、`pkg_tcy_bou.rpx` 縦中横・傍点）。`pkg_vert.rpx`: CJK 縦積み・ABC −90° 回転で上→下は **Step 8 想定どおり**。`。` `、` `ー` は GSUB `vert` 字形差替のみで **JLReq の縦組位置（打込み/突き出し等）は未実装** — Step 12 / `OPEN-TEXT-JA-001` へ残す。

### 2.4.4 Step 9 構造化ページ作者編集 完了gate（code HEAD `7ed3389` + slice 4）

Env D / offline。`document/page` の heading / paragraph / indent-em / columns を layer pane + props から Source へ往復。doc-page canvas nudge は soft-refuse。markup は soft-refuse 維持。

| Command | 結果 |
|---------|------|
| `cargo fmt --all --check` | green |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | green |
| `cargo test --workspace --offline` | green |
| `cargo check --workspace --all-targets --offline` | green |
| `cargo run --offline -p reciplexa-gui -- --smoke examples/text_line.rpx` | green (`smoke: ok`) |
| `cargo run --offline -p reciplexa -- examples/text_line.rpx .tmp/step9-gate-smoke.pdf` | green |
| `cargo test -p reciplexa-lower sync::document::tests --offline` | 6 passed |
| `cargo test -p reciplexa-gui --test vertical_slice_e2e --offline` | 7 passed |

**人間 GUI 確認（2026-08-20）:** OK（`pkg_document_indent.rpx` / `pkg_columns.rpx` — layer + props 編集）。

### 2.4.5 Step 10 Editor persistence v1 完了gate（code HEAD `b74b2e6`）

Env D / offline。atomic `.rpx` save、`.rpjsrc` crash journal、`.rpxsnap` journal/recovery、`RevisionUndoLog` undo/redo、caret/selection rebase、sidecar migration + partial extension recovery。

| Command | 結果 |
|---------|------|
| `cargo fmt --all --check` | green |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | green |
| `cargo test --workspace --offline` | green |
| `cargo check --workspace --all-targets --offline` | green |
| `cargo run --offline -p reciplexa-gui -- --smoke examples/text_line.rpx` | green (`smoke: ok`) |
| `cargo run --offline -p reciplexa -- examples/text_line.rpx .tmp/step10-gate-smoke.pdf` | green |
| `cargo test -p reciplexa-gui --test persistence_e2e --offline` | 6 passed |
| `cargo test -p reciplexa-codec --test sidecar_tests --test recovery_tests --test undo_tests --offline` | 16 passed |

### 2.5 製品Vertical Slice（**product slice complete**）

保証する範囲（package形式1ページ、`examples/text_line.rpx` 相当）:

- circle / line / text 等の基本graphics
- move、resize、insert、delete、reorder、text content 変更
- 保存 → 再読込 → PDF/SVG に編集後の文字と図形が残る
- interim `(page …)` を package ソースへ混入させない

明示的非対象（read-only / soft-refuse）:

- markup 作者同期
- 製品級 JLReq / OpenType MATH
- `document/page` の canvas 位置 nudge（構造編集と矛盾するため soft-refuse）

`document/page` の heading / paragraph / indent-em / columns は **Step 9 で layer + props から Source 往復**（[`step9-document-authoring-plan.md`](step9-document-authoring-plan.md)）。

これは長期 `roadmap.md` Phase 4 の Provenance / Shared Style / GUI Override 全体の完了ではない。

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

仕様上の単位・色・図形は **言語組み込みではない**（パッケージ API 供給）。ホットドメインの本文は **Direct Native v2**（DN2 stub + typed Rust callable；`KER-001` / Compiler-native package）。Hybrid v1 合成RPXは差分試験の参照のみ。[`direct-native-v2-plan.md`](direct-native-v2-plan.md)。

| パッケージ | 状況 | 内容の目安 |
|------------|------|------------|
| **`packages/graphics`** | **native**（N1–N2） | `shapes` / `page` / `color`＋`.rpi`；例 `pkg_graphics_*` |
| **`packages/length`** | **native**（N1） | 単位コンストラクタ＋`.rpi`；例 `pkg_length.rpx` |
| **`packages/color`** | **native**（N1） | srgb / named＋`.rpi`；例 `pkg_color.rpx` |
| **`packages/math`** | **native**（N3）＋ **std deepen（M0–M3）** ＋ **Wave 4–28 stubs** | atoms…stack＋`.rpi`；例 `pkg_math.rpx` / `pkg_math_box.rpx` / `pkg_math_phantom.rpx` / `pkg_math_spacing.rpx`；`reciplexa-std::math` に Accent/BigOp/Matrix/Aligned/Stack＋fontless `MathBox`；`EstimateStyle`（Text/Display；`SCRIPT_SCALE_TEXT`）／`phantom_box`／`smash_box`／stretchy delimiter／`accent_clearance_em`（hat/check/…／wide／underline／underbar）／`scripts_attachment_offsets`／`matrix_column_widths`／`aligned_column_x`／cases left-align／`cases_brace_total_height_em`（`nrows × row_height`）／`bigop_limit_offsets`／`fraction_rule_metrics`／`radical_vinculum_index_offsets`／`stackrel_spacing_offsets`／`underbrace_spacing`／`class_spacing_em`（Row muskip stub；Op/Bin/Rel/Punct/Fence pairs）；eval `math_value` 橋＋`scripts_attachment_offsets_from_value`；言語ビルトイン `math-box`（任意 `style`）／`math-phantom`／`math-smash`／`stretchy-delim`；`math-delimiter` 任意 `stretch-factor`；**OpenType MATH / 本格 glyph layout は後続** |
| **`packages/japanese`** | **native**（N4）＋ **std deepen（J0–J4）** ＋ **Wave 4–29 stubs** | classes / linebreak / kihon / markup；`reciplexa-std::japanese` に classify（UCS-ish subset；emoji/color → `Other`）/ denser §C-inspired `BREAK_PAIR_MATRIX` / break / kihon / ruby・tate-chu-yoko **estimate_box**／**vertical_ruby_estimate_box**／**bou_estimate_box**；`break_line`（cl-08 glue＋ASCII-space soft-wrap）/ `break_line_vertical` / `justify_line`（`trimming_width_em` 行頭・行末詰め stub）/ `justify_line_to_text_shapes` / `wrap_text_shape_content` / `vertical_advance_em`（`needs_tate_rotation` taller/wider swap）/ `indent_first_line` / `place_lines_horizontal`／`place_lines_vertical`／`KihonHanmen::line_pitch_em` / `measure_columns` / `reciprocal_punctuation_widths_em` / `hang_width_em`／`trimming_width_em`／`needs_tate_rotation`／`VerticalGlyphOrientation`；`CharClass::as_jlreq_id`／Display／`From`／`FromStr`（`cl-NN`）；`lines_to_text_shapes`（`place_lines_horizontal`）／`lines_to_vertical_text_shapes`（`place_lines_vertical`）；言語ビルトイン `classify-char` / `break-between` / `break-line` / `break-line-vertical` / `justify-line` / `ruby-box` / `vertical-ruby-box` / `bou-box` / `tate-chu-yoko-width` / `hang-width` / `trimming-width` / `vertical-orientation` / `math-box` / `measure-columns`；`doc-paragraph`／`doc-heading` が `break_line` で複数 Text（任意 `indent-em` 字下げ）；`doc-columns` が `measure_columns` で段組 stub；graphics_value text が任意 `wrap-em`／改行で soft-wrap；例 `pkg_ja_classify.rpx` / `pkg_ja_break.rpx` / `pkg_ja_vertical_break.rpx` / `pkg_ja_vertical_place.rpx` / `pkg_ruby.rpx` / `pkg_ja_hang.rpx` / `pkg_ja_hang_trim.rpx` / `pkg_document_indent.rpx` / `pkg_columns.rpx`；**完全 JLReq UCS / 実レイアウトではない** |
| **`packages/document`** | **native scaffold**（N5.1–N5.2）＋ **Wave 15 indent**＋ **Wave 24 columns** | `page`＝flow/section/heading 等（`doc-*`）；`paragraph-indented`；`columns`／`block-columns`（`doc-columns`）；例 `pkg_document.rpx` / `pkg_document_indent.rpx` / `pkg_columns.rpx`；pipeline 自動ルート済；**interim keyword 表は S6b で production から隔離（`interim-surface` / `cfg(test)`）** |
| **`crates/reciplexa-std`** | Rust ファサード（native 本文の主戦場） | `visual` / `text` / `document` / `math` / `japanese` / … |

ローカル import・path-dep・`workspace.rpxm` stub まで到達。**レジストリ等は OPEN-PKG-***。**N5.2 dual-path:** package 文書＋markup→graphics package emit＋GUI writable package sync（v2 S0–S6b）。interim keyword 表は fixture 向け `interim-surface` のみ。

---

## 4. Rust ホスト（CLI / GUI / PDF / SVG / PPTX）

言語クレート（syntax / core / eval / bind / macro / package / mem / effect / …）の上に、ホストが載る。

| ホスト | クレート | できること（現状） |
|--------|----------|-------------------|
| **CLI** | `reciplexa` | `parse` / `format` / `inspect-syntax` / `inspect-document` / `eval`；および `input.rpx → output.{pdf\|svg\|pptx}` エクスポート |
| **GUI** | `reciplexa-gui` (+ `gui-runtime`, `view`) | package 1ページの基本図形/文字を編集・保存・再読込・PDF/SVG（Vertical Slice **product slice complete**）。markup / `doc-*` は soft-refuse。仕様 EDT 全体には未到達 |
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

実行順の正本は [`active-roadmap.md`](active-roadmap.md)。Step 8 は完了（[`jlreq-followup-plan.md`](jlreq-followup-plan.md); gate HEAD `f9151cd`）。Step 9 complete（[`step9-document-authoring-plan.md`](step9-document-authoring-plan.md); gate §2.4.4）。**Step 10 complete**（[`step10-editor-persistence-plan.md`](step10-editor-persistence-plan.md); gate §2.4.5）— atomic save、crash recovery journal、revision undo/redo、caret/selection rebase、sidecar migration。**Step 11 active** — Text shaping / font resource v2。

1. **package 実運用化 local/offline slice** — [`package-plan.md`](package-plan.md): lock再現性、typed resource light、自動materialize、offline registry mirrorまで完了。network registry、full-tree hash、Core resource effectは別OPEN。
2. **Direct Native v2** — [`direct-native-v2-plan.md`](direct-native-v2-plan.md) **shipping complete**（DN2-0〜DN2-7 + 本番 BindingId dispatch via `op_for`）。標準packageは DN2 stub + compilation BindingId（`dn2bid-*`）+ typed Rust callable。author import 綴り（修飾 / alias / bare）は canonical `package/module` へ対応。Hybrid 参照本文は `LocalPackageIndex` override のみ。portable fallback は `OPEN-NATIVE-PKG-001` で明示追跡（未実装）。
2b. **DN2 観測同値の硬化** — 製品級組版の前。深い example 差分、`japanese/linebreak` 動的 export、構造化 Failure（`package/arity` / `package/type`）、native stub の body elaboration 省略（BindingId スロット、alias / bare を含む）。詳細は plan DN2-8。
3. **製品級組版** — [`product-typesetting-plan.md`](product-typesetting-plan.md) **complete（Profile v1; gates `3087565`）**。`reciplexa-text-layout` が共有 font/shaping 基盤。JLReq Profile v1 と Math Profile v1 は別エンジン。`doc-page` / `live-layout-demo` / `math-demo` / graphics `text` のホスト経路は product `Shape::GlyphRun`（Visual IR / PDF は layout GID。graphics は authoring 1 ノード = 1 cluster run）。GUI preview は digest family + layout advances。`pkg_live_math` の math origin は `text_y_mm`。ライブラリ stub API は従来。author `import japanese/*` / `import math/*` は未変更。
4. **JLReq follow-up** — [`jlreq-followup-plan.md`](jlreq-followup-plan.md) **complete（simple ruby + `vert` + 縦中横 / 傍点; code HEAD `caa2afd`; gate HEAD `52d088e`）**。人間 GUI 確認 2026-08-20。縦組句読点の JLReq 位置調整は Step 12 へ。jukugo / 縦ルビ / overhang は `OPEN-TEXT-JA-001` のまま。

### いま OPEN（次に追うもの）

履歴の「done」列ではなく、**まだ閉じていない契約**だけ:

1. **OPEN-PKG-001**（Step 16、残りはStep 22/24）— ネットワーク registry は未実装。ローカルミラー `registry/{name}/{version}/`（または `RPIX_REGISTRY_ROOT`）で `source registry` / `registry:` lock をオフライン解決。ミラーなしは PKG005 / OPEN-PKG-001 拒否。lock checksum は `package.rpxm` SHA-256。path 依存は PKG007 必須。full-tree / registry artifact hash は未着手。
2. **OPEN-NATIVE-PKG-001**（Step 16）— 標準packageの portable `.rpx` fallback、ABI/version negotiation、native unavailable 時のルーティングは未実装。本番は Direct Native v2 のみ。非native package の disk `.rpx` load は維持。定数は `reciplexa_package::OPEN_NATIVE_PKG_001_*`。
3. **package-resource**（Steps 16、18–19）— package load/eval は [`eval_package_entry_main`] で `(resource …)` を自動 materialize（`resource-id` / `content-hash` / `effect: Resource` / `resolved-path`）。replay 不一致は PKG008。Core 型の resource effect / bracket 分離は未着手（RSC-001）。
4. **OPEN-TEXT-LAYOUT-001**（Step 11）— 共通 Text Layout protocol の正確な型、font fallback、HarfBuzz 級 shaping、bidi。Profile v1 の行分割・行調整は font-backed。残りの bullets は閉じない。
5. **OPEN-TEXT-JA-001**（Step 12、完全閉包はStep 24）— 完全 UCS / 規範的 appendix C 全体 / CSS `text-orientation` / `text-emphasis` / jukugo 配分・縦ルビ・overhang / **縦組句読点・長音（`。` `、` `ー` 等）の JLReq 位置調整**（Step 8 は GSUB `vert` 字形差替と vmtx 積みのみ）。Profile v1 はクラス行列・cl-08 glue・hang/trim/justify。Step 8 item 1 は横組 simple ruby。item 2 は vertical-rl + GSUB `vert`。item 3 は tate-chu-yoko（ASCII 1–4）と bou 円（親 em の外、`BOU_PARENT_GAP_EM`）。stub 層（Waves 4–29）は参照のまま閉じない。
6. **OpenType MATH（Profile v1 以降）**（Step 13）— 完全 glyph assembly、MATH kern、`\fontdimen`、GUI 上の first-class editable math。Profile v1 は fixture MATH constants、font advance、scripts/fraction/radical（rule thickness）、stretchy は MATH Variants construction の prepared GID（cmap `(` ではない。scene `GlyphRun` / PDF Identity-H）、hat/tilde/dot/vec / bar-underline rules、display bigop variants、matrix/stack。check/breve 等の ASCII 代用はしない。`reciplexa-std::math` ヒューリスティックは参照。
7. **本番ページ消費（editable）**（Step 9 **complete**、Step 13–14）— `document/page` の heading / paragraph / indent-em / columns は layer + props から Source 往復（`sync/document.rs`）。math / JA markup tree の first-class GUI 編集は Step 13 以降。`live-layout-demo` 等のホスト preview は product engine。graphics `text` は cluster `GlyphRun`（authoring 1 ノード = 1 cluster run）。
8. **Font emission 残差**（Steps 11、15）— ホスト PDF は layout digest ごとに CID face を subset（`host_product_font` と `host_math_font`。同一 digest なら一面）。`Shape::GlyphRun` は layout GID を塗る。cmap 経路の非 ASCII `Text` は product face。digest 不一致は relayout エラー。Latin-only `Text` は Helvetica。GUI preview は同じ digest の egui familyで`GlyphRun`をlayout advance位置に置く（cmapグリフ。construction-only GIDのアウトラインはeguiではUnicode代用）。SVG/PPTXはUnicodeクラスタを同じ座標で出す。
9. **markup 作者同期**（Step 21または22で仕様依存を再評価）— GUI CST sync v2 は package AST のみ。`(markup …)` 展開ページは soft-refuse（意図的）。Step 9のscopeには含めない。
10. **その他意図的 skip** — 分数ルール色付け（既に黒）；`pkg_checksum_demo` 例（crate README で足りる）。

これらはバグ一覧ではなく、仕様 / 計画上まだ追わない（または別エンジンが必要な）項目である。
---

## 付記: クレート一覧の見取り図

**言語パイプライン寄り:** `syntax` · `core` · `macro` · `eval` · `bind` · `package` · `mem` · `effect` · `lower` · `ir` · `diagnostic` · `source` · `identity` · `types` · `outcome` · `test` · `proof` · `harden` · `native` · `opt` · `runtime`

**文書・ホスト寄り:** `document` · `scene` · `view` · `visual-ir` · `backend` · `codec` · `std` · `gui` · `gui-runtime` · `pdf` · `svg` · `pptx` · `raster` · `motion` · `video` · ルート `reciplexa`（CLI）

---

*このファイルは適合台帳の代替ではない。見出し単位の正は常に `part2-conformance.md` / stats JSON を参照すること。*
