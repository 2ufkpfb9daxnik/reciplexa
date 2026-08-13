# reciplexa

`.rpx` から PDF / SVG / PPTX を出し、egui GUI でキャンバスとソースを双方向同期するツールです。

- 規範仕様: [`lang/specification.md`](lang/specification.md)
- 実装メモ: [`dump.md`](dump.md)

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
$env:CARGO_TARGET_DIR = "d:\reciplexa\target"
$env:TEMP = "d:\reciplexa\.tmp"
$env:TMP = "d:\reciplexa\.tmp"
cargo run -p reciplexa -- examples\black_circle.rpx target\out.pdf
cargo run -p reciplexa-gui -- examples\black_circle.rpx
```

出力拡張子でバックエンドを選びます（`.pdf` / `.svg` / `.pptx`）。サンプルは `examples/`（`(//)` コメント、`(markup …)`、トップレベル `perform`/`handle`、最小 `(type …)`/`(val …)`）。GUI／描画向け `examples/` は package 形（`(import …)(val main (page …))`）；interim 組込み `page`/`circle` の keyword 表はテスト fixture 向けに残存（`crates/reciplexa-lower/tests/fixtures/interim_page.rpx`）。`RECIPLEXA_REQUIRE_PACKAGE=1` でトップレベル裸の `(page …)` を拒否できます。brace の `@form{…}` も受理します。
