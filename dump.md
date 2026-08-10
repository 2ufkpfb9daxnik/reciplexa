# reciplexa — 実装メモ（dump）

> 作業用メモ。規範は [`lang/specification.md`](lang/specification.md)、実装順は [`lang/roadmap.md`](lang/roadmap.md) / [`lang/language-kernel-plan.md`](lang/language-kernel-plan.md)。短い使い方は [`README.md`](README.md)。

## いまの位置

**言語カーネル**と**ドキュメント表面**を分離した。

| 経路 | API | 状態 |
|---|---|---|
| Language | `expand_language` → `elaborate_source` / `elaborate_units` → `typecheck_language_source` / `eval_source` | TYP/EFF/ROW/MOD/DAT/BND/KER/RSC v0 |
| Document | `expand_document_surface` → `reciplexa_types::typecheck_source` → lower → scene | page/circle 等（PKG 待ち） |

### 機能 ID ステータス（言語）

| ID | 状態 | メモ |
|---|---|---|
| LEX/SYN | 既存 | コメント・markup reader 等 |
| MAC-001 | v0 | `expand_language` + gensym |
| RES-001 | v0 | `resolve_language_source` |
| TYP-001 | v0 | Core `infer` + `Dynamic` stub |
| ROW-001 | 断片 | closed record unify + `Lacks` stub |
| EFF-001 | deep one-shot v0 | resume 後に body 継続；handler 再インストール |
| MOD-001 | v0 | `elaborate_units` インメモリ import |
| DAT-001 | v0 | surface `data`/`match` → Variant/Match（単一 binder） |
| BND letrec/var/set | v0 | `LetRec` + `LocalVar`/`Set` cells（alive escape） |
| KER-001 | v0 | `+ - * < =` BuiltinOp |
| RSC-001 | v0 | `MemoryFsHost` + `read-file`/`write-file` |
| EDT-001 | 部分 | BindingId 未スレッド；prototype を DOCUMENT SURFACE に隔離 |
| PKG-001 | 延期 | graphics/math/日本語 |

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

Document:
  .rpx → expand_document_surface → typecheck (types) → lower → scene
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
