//! Portable fallback contract tracking ([`OPEN_NATIVE_PKG_001_CODE`]).
//!
//! Standard packages ship Direct Native v2 (DN2 stub + typed Rust callables).
//! The normative spec also requires a portable `.rpx` fallback when native
//! implementation is unavailable, plus ABI/version negotiation. That routing is
//! **not** implemented — production hosts use Direct Native only. Non-native
//! packages continue to load checked-in `src/*.rpx` bodies unchanged.

/// Machine-facing code for the open portable-fallback contract.
pub const OPEN_NATIVE_PKG_001_CODE: &str = "OPEN-NATIVE-PKG-001";

/// Human-facing summary: portable fallback for std packages is deferred.
pub const OPEN_NATIVE_PKG_001_FALLBACK: &str = "OPEN-NATIVE-PKG-001: standard-package portable .rpx fallback, ABI/version negotiation, and native-unavailable routing are not implemented; production uses Direct Native v2 only";
