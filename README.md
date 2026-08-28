# reciplexa

`.rpx` から PDF / SVG / PPTX を出し、egui GUI でキャンバスとソースを双方向同期するツールです。

開発状況、読むべき文書、実装AIへの引き継ぎは、まず [`dump/README.md`](dump/README.md) を参照してください。ディスク方針（成果物は D: のみ、C: に Cargo/temp を置かない）は [`dump/windows-disk-policy.md`](dump/windows-disk-policy.md) と `dump/README.md` 先頭。

- 現在の実行順: [`dump/active-roadmap.md`](dump/active-roadmap.md)
- 現在動く機能とOPEN事項: [`dump/implemented-features.md`](dump/implemented-features.md)
- 規範仕様: [`dump/specification.md`](dump/specification.md)
- 第II部適合台帳: [`dump/part2-conformance.md`](dump/part2-conformance.md)

```powershell
$env:CARGO_HOME = "D:\dev-cache\cargo"
$env:PATH = "D:\dev-cache\cargo\bin;$env:USERPROFILE\.cargo\bin;$env:PATH"
$env:CARGO_TARGET_DIR = "D:\reciplexa\target"
$env:TEMP = "D:\reciplexa\.tmp"
$env:TMP = "D:\reciplexa\.tmp"
New-Item -ItemType Directory -Force -Path $env:TEMP, $env:CARGO_TARGET_DIR | Out-Null
cargo run --target-dir D:\reciplexa\target -p reciplexa -- examples\black_circle.rpx target\out.pdf
cargo run --target-dir D:\reciplexa\target -p reciplexa-gui -- --smoke examples\text_line.rpx
cargo run --target-dir D:\reciplexa\target -p reciplexa-gui -- examples\text_line.rpx
```

出力拡張子でバックエンドを選びます（`.pdf` / `.svg` / `.pptx`）。サンプルは `examples/`（`(//)` コメント、`(markup …)`、トップレベル `perform`/`handle`、最小 `(type …)`/`(val …)`）。GUI／描画向け `examples/` は package 形（`(import …)(val main (page …))`）；interim 組込み `page`/`circle` の keyword 表は fixture 専用（`interim-surface` / `crates/reciplexa-lower/tests/fixtures/interim_page.rpx`）。本番 pipeline はトップレベル裸の `(page …)` を拒否します。brace の `@form{…}` も受理します。
