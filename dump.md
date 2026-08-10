# reciplexa — 実装メモ（dump）

> 作業用メモ。規範は [`lang/specification.md`](lang/specification.md)、実装順は [`lang/roadmap.md`](lang/roadmap.md)。短い使い方は [`README.md`](README.md)。

## いまの位置

コメント `(// …)`、文章 reader `(markup …)`、`@name(…)` / `@name{…}` は SYN-001 に寄せた。図形・`(src)`・`type`/`val` などは未接続またはプロトタイプのまま。

| 項目 | 規範（SYN-001） | いまの実装 / examples |
|---|---|---|
| コメント | `(// …)` | 対応（`;` 行コメントは廃止） |
| 文章 reader | `(markup …)` | 対応（旧 `(doc …)` は廃止） |
| markup command | `@name(…)` / `@name[…]` 等 | `@name(…)` と `@name{…}` の両方を受理。意味はプロトタイプ展開 |
| code mode 宣言 | `(type …)` / `(val …)` | 未接続（typecheck で未知 form） |
| `(src …)` wrapper | 使わない | まだ effect 用に必要（`effects.rpx`） |
| 図形 | package が定義 | 組込み `(page … (circle …))` 等 |
| 単位リテラル | `40mm` 等 | 未接続（裸の数値 mm 前提） |

ビルド成果物は D: の `target/`（`.cargo/config.toml`）。エージェント環境が `CARGO_TARGET_DIR` を C: に上書きする場合は打ち消すこと。

## パイプライン

```text
.rpx → expand → typecheck → lower → scene
         ├─ GUI preview: effects なし
         └─ export: EffectHandler
```

```lisp
(// 図形)
(page a4 (circle 105 148.5 40))

(markup
@heading(Cover)
本文と@strong(強調)。)
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
