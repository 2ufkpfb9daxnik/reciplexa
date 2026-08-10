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

出力拡張子でバックエンドを選びます（`.pdf` / `.svg` / `.pptx`）。サンプルは `examples/`（`(//)` コメントと `(markup …)`）。
