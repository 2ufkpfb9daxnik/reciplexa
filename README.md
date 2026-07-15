# reciplexa

[Glisp](https://github.com/baku89/glisp) にインスパイアされた、グラフィックス／組版ツールです。  
`.rpx`（Two-Faced LISP + Scribble）から PDF を出し、egui GUI でキャンバスとソースを双方向同期します。

設計の詳細・マイルストーンは [`dump.md`](dump.md) を参照。

## 必要環境

- Rust stable（PATH に `cargo`）
- Windows では GUI の日本語プレビュー用に Noto Sans JP 等があるとよい

## ビルドとテスト

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## サンプルを PDF にする

```powershell
cargo run -p reciplexa -- examples\doc_title_p.rpx target\out.pdf
cargo run -p reciplexa -- examples\doc_with_figure.rpx target\figure.pdf
```

乱数シードを固定したいときは `RECIPLEXA_SEED`（未設定時は `1`）。

## GUI

```powershell
cargo run -p reciplexa-gui -- examples\doc_title_p.rpx
```

プレビューは `(src …)` 効果を走らせません。エクスポート時だけ `perform` / `handle` が動きます。
