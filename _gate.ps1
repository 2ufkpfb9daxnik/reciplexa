$env:CARGO_HOME = "D:\dev-cache\cargo"
$env:PATH = "D:\dev-cache\cargo\bin;$env:USERPROFILE\.cargo\bin;$env:PATH"
$env:CARGO_TARGET_DIR = "D:\reciplexa\target"
$env:TEMP = "D:\reciplexa\.tmp"
$env:TMP = "D:\reciplexa\.tmp"
New-Item -ItemType Directory -Force -Path $env:TEMP, $env:CARGO_TARGET_DIR | Out-Null
Set-Location D:\reciplexa
$td = "--target-dir"
$target = "D:\reciplexa\target"
cargo fmt --all
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo clippy --workspace --all-targets --offline $td $target -- -D warnings 2>&1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --workspace --offline $td $target 2>&1
exit $LASTEXITCODE
