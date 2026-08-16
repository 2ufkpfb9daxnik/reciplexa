# reciplexa

`.rpx` から PDF / SVG / PPTX を出し、egui GUI でキャンバスとソースを双方向同期するツールです。

開発状況、読むべき文書、実装AIへの引き継ぎは、まず [`dump/README.md`](dump/README.md) を参照してください。

- 現在の実行順: [`dump/active-roadmap.md`](dump/active-roadmap.md)
- 現在動く機能とOPEN事項: [`dump/implemented-features.md`](dump/implemented-features.md)
- 規範仕様: [`dump/specification.md`](dump/specification.md)
- 第II部適合台帳: [`dump/part2-conformance.md`](dump/part2-conformance.md)

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
$env:CARGO_TARGET_DIR = "d:\reciplexa\target"
$env:TEMP = "d:\reciplexa\.tmp"
$env:TMP = "d:\reciplexa\.tmp"
cargo run -p reciplexa -- examples\black_circle.rpx target\out.pdf
cargo run -p reciplexa-gui -- --smoke examples\text_line.rpx
cargo run -p reciplexa-gui -- examples\text_line.rpx
```

出力拡張子でバックエンドを選びます（`.pdf` / `.svg` / `.pptx`）。サンプルは `examples/`（`(//)` コメント、`(markup …)`、トップレベル `perform`/`handle`、最小 `(type …)`/`(val …)`）。GUI／描画向け `examples/` は package 形（`(import …)(val main (page …))`）；interim 組込み `page`/`circle` の keyword 表は fixture 専用（`interim-surface` / `crates/reciplexa-lower/tests/fixtures/interim_page.rpx`）。本番 pipeline はトップレベル裸の `(page …)` を拒否します。brace の `@form{…}` も受理します。
