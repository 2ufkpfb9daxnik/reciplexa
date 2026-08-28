# Move Reciplexa-related caches off C: onto D:\dev-cache and junction the old paths.
# Safe to re-run. Does not move rustup toolchains (allowed to stay on C:).
$ErrorActionPreference = "Continue"
$dev = "D:\dev-cache"
$tmpD = "D:\reciplexa\.tmp"
$targetD = "D:\reciplexa\target"
New-Item -ItemType Directory -Force -Path $dev, $tmpD, $targetD | Out-Null

function Test-Reparse([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path)) { return $false }
    $item = Get-Item -LiteralPath $Path -Force
    return [bool]($item.Attributes -band [IO.FileAttributes]::ReparsePoint)
}

function Write-LinkOrSkip([string]$Link, [string]$Target) {
    if (Test-Reparse $Link) {
        Write-Output "OK reparse $Link"
        return
    }
    if (Test-Path -LiteralPath $Link) {
        Write-Output "REFUSE junction; real directory still at $Link"
        return
    }
    New-Item -ItemType Directory -Force -Path $Target | Out-Null
    cmd /c "mklink /J `"$Link`" `"$Target`""
}

# 1) Cursor sandbox cargo-target duplicate on C: — delete, then junction to D:.
$sandboxSrc = Join-Path $env:LOCALAPPDATA "Temp\cursor-sandbox-cache"
$sandboxDst = Join-Path $dev "cursor-sandbox-cache"
New-Item -ItemType Directory -Force -Path $sandboxDst | Out-Null
if ((Test-Path -LiteralPath $sandboxSrc) -and -not (Test-Reparse $sandboxSrc)) {
    Write-Output "Removing C: sandbox cache (duplicate cargo-target)..."
    cmd /c "rmdir /s /q `"$sandboxSrc`""
}
Write-LinkOrSkip $sandboxSrc $sandboxDst

# 2) rustc scratch dirs left in the user Temp on C:.
$tempRoot = Join-Path $env:LOCALAPPDATA "Temp"
if (Test-Path -LiteralPath $tempRoot) {
    Get-ChildItem -LiteralPath $tempRoot -Directory -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -like "rustc*" } |
        ForEach-Object {
            Write-Output "Removing $($_.FullName)"
            cmd /c "rmdir /s /q `"$($_.FullName)`""
        }
}

# 3) Cargo home (registry/git/bin) -> D:\dev-cache\cargo
$cargoSrc = Join-Path $env:USERPROFILE ".cargo"
$cargoDst = Join-Path $dev "cargo"
if ((Test-Path -LiteralPath $cargoSrc) -and -not (Test-Reparse $cargoSrc)) {
    Write-Output "Moving CARGO_HOME $cargoSrc -> $cargoDst"
    New-Item -ItemType Directory -Force -Path $cargoDst | Out-Null
    & robocopy $cargoSrc $cargoDst /E /MOVE /R:2 /W:2 /NFL /NDL /NJH /NJS
    if (Test-Path -LiteralPath $cargoSrc) {
        cmd /c "rmdir /s /q `"$cargoSrc`""
    }
}
Write-LinkOrSkip $cargoSrc $cargoDst
if ((Test-Path -LiteralPath $cargoSrc) -and -not (Test-Reparse $cargoSrc)) {
    Write-LinkOrSkip (Join-Path $cargoSrc "registry") (Join-Path $cargoDst "registry")
    Write-LinkOrSkip (Join-Path $cargoSrc "git") (Join-Path $cargoDst "git")
    $binSrc = Join-Path $cargoSrc "bin"
    $binDst = Join-Path $cargoDst "bin"
    if ((Test-Path -LiteralPath $binDst) -and (Test-Path -LiteralPath $binSrc)) {
        Write-Output "Copying cargo/rustup proxies into leftover C bin (PATH)"
        cmd /c "xcopy /y /d `"$binDst\*`" `"$binSrc\`""
    }
}

Write-Output "done"
Get-Item -LiteralPath $sandboxSrc, $cargoSrc -Force -ErrorAction SilentlyContinue |
    Select-Object FullName, Attributes, LinkType, Target
