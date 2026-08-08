//! Shebang detection per `specification.md` SYN-001 §1.5.

/// Whether `text` begins with a Unix shebang at byte offset 0 (after optional BOM strip).
pub fn detect_shebang(text: &str) -> Option<&str> {
    let body = text.strip_prefix('\u{feff}').unwrap_or(text);
    body.starts_with("#!")
        .then(|| body.lines().next().unwrap_or(body))
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
