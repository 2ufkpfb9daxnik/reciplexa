@echo off
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
set "CARGO_TARGET_DIR=d:\reciplexa\target"
set "TEMP=d:\reciplexa\.tmp"
set "TMP=d:\reciplexa\.tmp"
if not exist "d:\reciplexa\.tmp" mkdir "d:\reciplexa\.tmp"
cd /d d:\reciplexa
echo === FMT ===
cargo fmt --all
if errorlevel 1 exit /b 1
echo === CLIPPY ===
cargo clippy --workspace --all-targets --offline -- -D warnings
if errorlevel 1 exit /b 1
echo === TEST (interim-surface fixture crates) ===
cargo test -p reciplexa-lower --features interim-surface --offline
if errorlevel 1 exit /b 1
cargo test -p reciplexa-types --features interim-surface --offline
if errorlevel 1 exit /b 1
cargo test -p reciplexa-bind --features interim-surface --offline
if errorlevel 1 exit /b 1
echo === TEST (workspace production default) ===
cargo test --workspace --offline --exclude reciplexa-lower --exclude reciplexa-types --exclude reciplexa-bind
if errorlevel 1 exit /b 1
echo === GUI CHECK ===
cargo check --offline -p reciplexa-gui
if errorlevel 1 exit /b 1
echo === GATE GREEN ===
exit /b 0
