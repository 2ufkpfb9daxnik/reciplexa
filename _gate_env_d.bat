@echo off
set "CARGO_HOME=D:\dev-cache\cargo"
set "PATH=D:\dev-cache\cargo\bin;%USERPROFILE%\.cargo\bin;%PATH%"
set "CARGO_TARGET_DIR=D:\reciplexa\target"
set "TEMP=D:\reciplexa\.tmp"
set "TMP=D:\reciplexa\.tmp"
if not exist "D:\reciplexa\.tmp" mkdir "D:\reciplexa\.tmp"
if not exist "D:\reciplexa\target" mkdir "D:\reciplexa\target"
cd /d D:\reciplexa
set "TD=D:\reciplexa\target"
echo === FMT ===
cargo fmt --all
if errorlevel 1 exit /b 1
echo === CLIPPY ===
cargo clippy --workspace --all-targets --offline --target-dir %TD% -- -D warnings
if errorlevel 1 exit /b 1
echo === TEST (interim-surface fixture crates) ===
cargo test -p reciplexa-lower --features interim-surface --offline --target-dir %TD%
if errorlevel 1 exit /b 1
cargo test -p reciplexa-types --features interim-surface --offline --target-dir %TD%
if errorlevel 1 exit /b 1
cargo test -p reciplexa-bind --features interim-surface --offline --target-dir %TD%
if errorlevel 1 exit /b 1
echo === TEST (workspace production default) ===
cargo test --workspace --offline --exclude reciplexa-lower --exclude reciplexa-types --exclude reciplexa-bind --target-dir %TD%
if errorlevel 1 exit /b 1
echo === GUI CHECK ===
cargo check --offline -p reciplexa-gui --target-dir %TD%
if errorlevel 1 exit /b 1
echo === GATE GREEN ===
exit /b 0
