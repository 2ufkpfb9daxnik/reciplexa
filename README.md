# reciplexa

[Glisp](https://github.com/baku89/glisp) にインスパイアされた、グラフィックス／組版ツールです。  
`.rpx`（Two-Faced LISP + Scribble）から PDF を出し、egui GUI でキャンバスとソースを双方向同期します。

設計の詳細・マイルストーンは [`dump.md`](dump.md) を参照。

## 必要環境

- Rust stable（PATH に `cargo`）
- Windows では GUI の日本語プレビュー用に Noto Sans JP 等があるとよい

## ビルドとテスト

GitHub Actions の CI と同じく、**ワークスペース全体**を毎回見ます（新しいファイルだけではありません）。
コミット前にこの3つを通すのが安全です（全体でもおおよそ数分以内）。

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

日本語を PDF のアウトライン化にするときは Noto Sans JP など（`%WINDIR%\Fonts`）が必要です。  
CI ではフォントをダウンロードし、`RECIPLEXA_CJK_FONT` でパスを渡します。ローカルでも同じ環境変数で上書きできます。

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
