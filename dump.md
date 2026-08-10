# reciplexa — 実装メモ（dump）

> 作業用メモ。規範は [`lang/specification.md`](lang/specification.md)、実装順は [`lang/roadmap.md`](lang/roadmap.md)。短い使い方は [`README.md`](README.md)。

## いまの位置

コメント `(// …)`、文章 reader `(markup …)`、トップレベル `perform`/`handle`、最小の `(type …)` / `(val …)` スタブまで SYN-001 に寄せた。図形はまだ組込み `(page …)` プレリュード。

| 項目 | 規範（SYN-001） | いまの実装 / examples |
|---|---|---|
| コメント | `(// …)` | 対応（`;` 行コメントは廃止） |
| 文章 reader | `(markup …)` | 対応（旧 `(doc …)` は廃止） |
| markup command | `@name(…)` / `@name[…]` 等 | `@name(…)` と `@name{…}`（brace）の両方を受理。意味はプロトタイプ展開 |
| code mode 宣言 | `(type …)` / `(val …)` | 最小スタブ（arity・名前宣言のみ；本体は未型検査）。`examples/decls_stub.rpx` |
| `(src …)` wrapper | 使わない | 非推奨だがまだ解釈する。新規はトップレベル `perform`/`handle`（`effects.rpx`） |
| 図形 | package が定義 | 組込み `(page … (circle …))` 等（interim prelude） |
| 単位リテラル | `40mm` 等 | 未接続（裸の数値 mm 前提） |

ビルド成果物は D: の `target/`（`.cargo/config.toml`）。エージェント環境が `CARGO_TARGET_DIR` を C: に上書きする場合は打ち消すこと。

## パイプライン

```text
.rpx → expand → typecheck → lower → scene
         ├─ GUI preview: effects なし
         └─ export: EffectHandler
```

```lisp
(// 図形 — interim)
(page a4 (circle 105 148.5 40))

(markup
@heading(Cover)
本文と@strong(強調)。)

(type title str)
(val title "Hello")
```

## ゲート

```text
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
$env:CARGO_TARGET_DIR = "d:\reciplexa\target"
$env:TEMP = "d:\reciplexa\.tmp"
$env:TMP = "d:\reciplexa\.tmp"
```
