# reciplexa — 実装メモ（dump）

> 作業用メモ。規範は [`lang/specification.md`](lang/specification.md)、実装順は [`lang/roadmap.md`](lang/roadmap.md) / [`lang/language-kernel-plan.md`](lang/language-kernel-plan.md)。短い使い方は [`README.md`](README.md)。

## いまの位置

**言語カーネルは pre-PKG 向けに COMPLETE。** ドキュメント表面と分離済み。PKG（graphics/math/日本語）と一部高度な言語機能は意図的に延期。

| 経路 | API | 状態 |
|---|---|---|
| Language | `expand_language` → `elaborate_source` / `elaborate_units` / `load_module_tree` → `typecheck_language_source` / `eval_source` | pre-PKG COMPLETE |
| Document | `expand_document_surface` → `reciplexa_types::typecheck_source` → lower → scene | page/circle 等（PKG 待ち） |

### 機能 ID ステータス（言語）

| ID | 状態 | メモ |
|---|---|---|
| LEX/SYN | 既存 | コメント・markup reader 等 |
| MAC-001 | **done** | `expand_language` + gensym |
| RES-001 | **done** | `resolve_language_source` |
| TYP-001 | **done** | Core `infer` / `infer_with_effects` + `Dynamic` stub |
| ROW-001 | **done** (fragment) | closed + `OpenRecord`; `Lacks` enforced |
| EFF-001 | **done** (deep one-shot) | EffectRow on Fun; multi-shot deferred |
| MOD-001 | **done** (v0) | `elaborate_units` + `load_module_tree` sibling `.rpx`; functors deferred |
| DAT-001 | **done** (v0) | data/match + **static exhaustiveness**; multi-payload deferred |
| BND letrec/var/set | **done** (v0) | LetRec + cells; full escape analysis deferred |
| KER-001 | **done** (v0) | `+ - * < =` BuiltinOp; full foreign ABI deferred |
| RSC-001 | **done** (v0) | MemoryFsHost; richer catalog deferred |
| EDT-001 | **done** (BindingId map) | `BindingMap` use→decl; SyntaxNodeId thread deferred |
| PKG-001 | **延期** | graphics/math/日本語 |

### 意図的な延期（pre-PKG 後）

- multi-shot / shallow effect handlers
- full ML functors / module signatures
- foreign/Rust ABI beyond BuiltinOp + EffectHost
- multi-payload / parameterized ADTs
- unrestricted row tallying
- SyntaxNodeId provenance through elaborate→eval

コメント `(// …)`、文章 reader `(markup …)`、トップレベル `perform`/`handle`（document）、最小の `(type …)` / `(val …)` スタブまで SYN-001 に寄せた。図形はまだ組込み `(page …)` プレリュード。

| 項目 | 規範（SYN-001） | いまの実装 / examples |
|---|---|---|
| コメント | `(// …)` | 対応（`;` 行コメントは廃止） |
| 文章 reader | `(markup …)` | 対応（旧 `(doc …)` は廃止） |
| markup command | `@name(…)` / `@name[…]` 等 | `@name(…)` と `@name{…}`（brace）の両方を受理。意味はプロトタイプ展開 |
| code mode 宣言 | `(type …)` / `(val …)` | 言語: elaborate+Core 型検査。document: arity・名前スタブ |
| `(src …)` wrapper | 使わない | 非推奨だがまだ解釈する。新規はトップレベル `perform`/`handle`（`effects.rpx`） |
| 図形 | package が定義 | 組込み `(page … (circle …))` 等（interim prelude） |
| 単位リテラル | `40mm` 等 | 未接続（裸の数値 mm 前提） |

ビルド成果物は D: の `target/`（`.cargo/config.toml`）。エージェント環境が `CARGO_TARGET_DIR` を C: に上書きする場合は打ち消すこと。

## パイプライン

```text
Language:
  .rpx → expand_language → elaborate → typecheck_language / eval
  multi-unit: load_module_tree → elaborate_units

Document:
  .rpx → expand_document_surface → typecheck (types::document_surface) → lower → scene
         ├─ GUI preview: effects なし
         └─ export: EffectHandler
```

```lisp
(// language)
(val main ((fn (x) x) 1))

(// document — interim)
(page a4 (circle 105 148.5 40))
```

## ゲート

```text
cargo fmt --all
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
cargo check --offline -p reciplexa-gui
```

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
$env:CARGO_TARGET_DIR = "d:\reciplexa\target"
$env:TEMP = "d:\reciplexa\.tmp"
$env:TMP = "d:\reciplexa\.tmp"
```
