//! Integration tests moved from src/shebang.rs for region coverage.

use reciplexa_source::shebang::*;

#[test]
fn detects_shebang_at_start() {
    assert_eq!(
        detect_shebang("#!/usr/bin/env rpx\n(page a4)"),
        Some("#!/usr/bin/env rpx")
    );
}

#[test]
fn shebang_after_bom_is_detected_for_conflict_check() {
    assert!(detect_shebang("\u{feff}#!/usr/bin/env rpx").is_some());
}

#[test]
fn no_shebang_returns_none() {
    assert!(detect_shebang("(page a4)").is_none());
}
