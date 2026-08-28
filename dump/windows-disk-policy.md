# Windows ディスク方針（必ず守る）

Reciplexa の作業木は **D:\reciplexa** にある。C: は容量が逼迫している。開発成果物を C: に書いてはならない。

## 置いてよい場所

| もの | 場所 |
| --- | --- |
| ソース、`target/`、一時ファイル、gate 出力 | **D:\reciplexa**（`target`、`.tmp`） |
| Cargo レジストリ / git / bin（CARGO_HOME） | **D:\dev-cache\cargo**（`%USERPROFILE%\.cargo` はジャンクション） |
| Cursor agent の cargo サンドボックスキャッシュ | **D:\dev-cache\cursor-sandbox-cache** |
| OS、Cursor 本体 | C:（移動しない） |
| Rust 処理系（rustup toolchains） | C: に残してよい。移せるなら **D:\dev-cache\rustup** |

C: の `%LOCALAPPDATA%\Temp` に `cargo-target`、`rustc*`、sandbox キャッシュを置かない。

`CARGO_HOME` は **D:\dev-cache\cargo**。未設定のままだと `%USERPROFILE%\.cargo\registry` が C: に作り直される。

すべての compile 系コマンドの前に:

```powershell
$env:CARGO_HOME = "D:\dev-cache\cargo"
$env:PATH = "D:\dev-cache\cargo\bin;$env:PATH"

## すべての cargo 呼び出し

Cursor の agent シェルは `CARGO_TARGET_DIR` を

`%LOCALAPPDATA%\Temp\cursor-sandbox-cache\<hash>\cargo-target`

へ差し替える。これは **C:** 上であり、workspace の `target/` や `.cargo/config.toml` の相対 `target-dir` を無視する。ディスクフルの原因になる。

必ず次を同時に付ける。

```powershell
$env:CARGO_HOME = "D:\dev-cache\cargo"
$env:PATH = "D:\dev-cache\cargo\bin;$env:PATH"
$env:CARGO_TARGET_DIR = "D:\reciplexa\target"
$env:TEMP = "D:\reciplexa\.tmp"
$env:TMP = "D:\reciplexa\.tmp"
New-Item -ItemType Directory -Force -Path $env:TEMP, $env:CARGO_TARGET_DIR | Out-Null
cargo <subcommand> --target-dir D:\reciplexa\target ...
```

- `--target-dir D:\reciplexa\target` はサンドボックスの `CARGO_TARGET_DIR` より優先する。**省略禁止**（`cargo fmt` を除く）。
- `TEMP` / `TMP` を D: にしないと rustc / `link.exe` が C:\Users\...\Temp\rustc* に中間ファイルを書く。
- ネットワーク依存を増やさない。offline 方針は変えない。

## 移行済みジャンクション

初回移行後、次のパスはディレクトリジャンクションで D: を指す。再実行は `scripts/migrate-c-caches-to-d.ps1`。

- `%LOCALAPPDATA%\Temp\cursor-sandbox-cache` → `D:\dev-cache\cursor-sandbox-cache`（実施済み）
- `%USERPROFILE%\.cargo` 全体のジャンクションは `rustup.exe` が C: でロックされているため未完。レジストリ実体は `D:\dev-cache\cargo`。`%USERPROFILE%\.cargo\registry` と `git` をジャンクションし、`CARGO_HOME=D:\dev-cache\cargo` を使う。ロック解除後に `scripts/migrate-c-caches-to-d.ps1` を再実行する。

ジャンクションを外して実体を C: に戻さない。
