param(
    [Parameter(Mandatory = $true)]
    [string[]]$Crates
)

Set-Location $PSScriptRoot\..

cargo fmt --all
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$cargoArgs = @("test")
foreach ($crate in $Crates) {
    $cargoArgs += "-p"
    $cargoArgs += $crate
}
& cargo @cargoArgs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$clippyArgs = @("clippy", "--all-targets", "--", "-D", "warnings")
foreach ($crate in $Crates) {
    $clippyArgs += "-p"
    $clippyArgs += $crate
}
& cargo @clippyArgs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
