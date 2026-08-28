param(
    [Parameter(Mandatory = $true)]
    [string[]]$Crates
)

Set-Location $PSScriptRoot\..

$env:CARGO_HOME = "D:\dev-cache\cargo"
$env:PATH = "D:\dev-cache\cargo\bin;$env:PATH"
$env:CARGO_TARGET_DIR = "D:\reciplexa\target"
$env:TEMP = "D:\reciplexa\.tmp"
$env:TMP = "D:\reciplexa\.tmp"
New-Item -ItemType Directory -Force -Path $env:TEMP, $env:CARGO_TARGET_DIR | Out-Null
$td = "D:\reciplexa\target"

cargo fmt --all
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$cargoArgs = @("test", "--target-dir", $td)
foreach ($crate in $Crates) {
    $cargoArgs += "-p"
    $cargoArgs += $crate
}
& cargo @cargoArgs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$clippyArgs = @("clippy", "--all-targets", "--target-dir", $td)
foreach ($crate in $Crates) {
    $clippyArgs += "-p"
    $clippyArgs += $crate
}
$clippyArgs += "--"
$clippyArgs += "-D"
$clippyArgs += "warnings"
& cargo @clippyArgs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
