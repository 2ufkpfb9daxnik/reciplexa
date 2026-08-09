# reciplexa — 実装メモ（dump）

> 作業用メモ。規範は [`lang/specification.md`](lang/specification.md)、実装順は [`lang/roadmap.md`](lang/roadmap.md)。短い使い方は [`README.md`](README.md)。

## いまの位置

Phase 1–14 のクレート骨格はある。表面のうち **コメントは SYN-001 の `(// …)`**、**文章 reader 名は `(markup …)`** に寄せた。一方で **図形・`(src)`・組版コマンド意味**などはまだプロトタイプのまま動いている。

| | 規範（SYN-001） | いま動くもの |
|---|---|---|
| コメント | `(// …)` | `(// …)`（`;` 行コメントは廃止） |
| 文章 reader | `(markup …)` | `(markup …)` + `@command`（意味はまだプロトタイプ展開） |
| 既定モード | code mode（`type`/`val` 等） | 未接続（top-level は主に `page` / `markup` / `src`） |
| 図形 | package が定義 | 組込み `(page … (circle …))` 等 |

`examples/` は現行パイプラインで preview / export できるものだけ。

## 現行パイプライン

```text
.rpx → expand（surface macro / markup）→ typecheck → lower → scene
         ├─ GUI preview: effects は走らせない
         └─ export: EffectHandler 経由で perform / handle
```

```lisp
(// 図形ページ（mm、原点は左下）)
(page a4
  (circle 105 148.5 40))

(// エクスポート時だけ動く効果)
(src
  (perform log "hello")
  (handle log (perform log "muted")))

(markup
@title{Cover}
@p{Body with @em{emphasis}.})
```

## コミット前ゲート

```text
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

PowerShell では先に `$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"`。
