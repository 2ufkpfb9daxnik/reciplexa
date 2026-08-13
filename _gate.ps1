$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
$env:CARGO_TARGET_DIR = "d:\reciplexa\target"
$env:TEMP = "d:\reciplexa\.tmp"
$env:TMP = "d:\reciplexa\.tmp"
New-Item -ItemType Directory -Force -Path $env:TEMP, $env:CARGO_TARGET_DIR | Out-Null
Set-Location d:\reciplexa
cargo fmt --all
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo clippy --workspace --all-targets --offline -- -D warnings 2>&1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --workspace --offline 2>&1
exit $LASTEXITCODE
