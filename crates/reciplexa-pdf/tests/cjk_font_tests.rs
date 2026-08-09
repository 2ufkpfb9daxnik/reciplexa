use reciplexa_pdf::{
    cjk_font_path, subset_tag, system_cjk_font_path, utf16_hex, CjkFontEmbed, PdfError,
};
use std::collections::BTreeSet;
use std::sync::{Mutex, OnceLock};

static CJK_TEST_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn lock_cjk_test_env() -> std::sync::MutexGuard<'static, ()> {
    CJK_TEST_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap()
}

fn with_saved_cjk_font_env<F: FnOnce()>(f: F) {
    let _guard = lock_cjk_test_env();
    let saved = std::env::var_os("RECIPLEXA_CJK_FONT");
    f();
    if let Some(v) = saved {
        std::env::set_var("RECIPLEXA_CJK_FONT", v);
    } else {
        std::env::remove_var("RECIPLEXA_CJK_FONT");
    }
}

#[test]
fn subset_tag_is_six_uppercase_letters() {
    let tag = subset_tag(b"hello");
    assert_eq!(tag.len(), 6);
    assert!(tag.chars().all(|c| c.is_ascii_uppercase()));
    assert_eq!(tag, subset_tag(b"hello"));
    assert_ne!(tag, subset_tag(b"world"));
}

#[test]
fn utf16_hex_bmp_and_supplementary() {
    assert_eq!(utf16_hex(0x0041), "<0041>");
    assert_eq!(utf16_hex(0x1F600), "<D83DDE00>");
}

#[test]
fn build_rejects_empty_char_set() {
    let err = match CjkFontEmbed::build(&BTreeSet::new()) {
        Err(e) => e,
        Ok(_) => panic!("expected error"),
    };
    assert!(matches!(err, PdfError::InvalidShape(_)));
}

#[test]
fn encode_hex_rejects_embedded_newlines() {
    let _guard = lock_cjk_test_env();
    if system_cjk_font_path().is_none() {
        return;
    }
    let mut chars = BTreeSet::new();
    chars.insert('日');
    let embed = CjkFontEmbed::build(&chars).unwrap();
    let err = embed.encode_hex("日\n").unwrap_err();
    assert!(matches!(err, PdfError::InvalidShape(_)));
    let err = embed.encode_hex("日\r").unwrap_err();
    assert!(matches!(err, PdfError::InvalidShape(_)));
}

#[test]
fn missing_glyph_errors_when_font_available() {
    let _guard = lock_cjk_test_env();
    if system_cjk_font_path().is_none() {
        return;
    }
    let mut chars = BTreeSet::new();
    // Private-use character almost never present in system CJK faces.
    chars.insert('\u{F8FF}');
    let err = match CjkFontEmbed::build(&chars) {
        Err(e) => e,
        Ok(_) => panic!("expected missing glyph"),
    };
    assert!(matches!(err, PdfError::InvalidShape(_)));
}

#[test]
fn no_cjk_font_errors_when_env_cleared() {
    let _guard = lock_cjk_test_env();
    let saved_font = std::env::var_os("RECIPLEXA_CJK_FONT");
    let saved_windir = std::env::var_os("WINDIR");
    let empty = std::env::temp_dir().join(format!("reciplexa_empty_fonts_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&empty);
    std::env::remove_var("RECIPLEXA_CJK_FONT");
    std::env::set_var("WINDIR", &empty);
    assert!(system_cjk_font_path().is_none());
    let mut chars = BTreeSet::new();
    chars.insert('日');
    let err = match CjkFontEmbed::build(&chars) {
        Err(e) => e,
        Ok(_) => panic!("expected no-font error"),
    };
    assert!(matches!(err, PdfError::InvalidShape(_)));
    // Also exercise WINDIR default when the variable is unset.
    std::env::remove_var("WINDIR");
    let _ = system_cjk_font_path();
    if let Some(v) = saved_font {
        std::env::set_var("RECIPLEXA_CJK_FONT", v);
    } else {
        std::env::remove_var("RECIPLEXA_CJK_FONT");
    }
    if let Some(v) = saved_windir {
        std::env::set_var("WINDIR", v);
    } else {
        std::env::remove_var("WINDIR");
    }
    let _ = std::fs::remove_dir_all(&empty);
}

#[test]
fn cjk_roundtrip_widths_and_cmap_when_font_available() {
    let _guard = lock_cjk_test_env();
    if system_cjk_font_path().is_none() {
        return;
    }
    let mut chars = BTreeSet::new();
    chars.insert('あ');
    chars.insert('語');
    let embed = CjkFontEmbed::build(&chars).unwrap();
    let hex = embed.encode_hex("あ").unwrap();
    assert!(!hex.is_empty());
    let cmap = embed.to_unicode_cmap();
    assert!(cmap.contains("begincmap"));
    assert!(cmap.contains("endbfchar"));
    let widths = embed.widths_array();
    assert!(widths.starts_with("[ "));
    assert!(widths.ends_with(']'));
    assert!(!embed.base_name.is_empty());
    assert!(embed.font_bbox[2] >= embed.font_bbox[0]);
}

#[test]
fn cjk_font_path_alias_matches_system() {
    assert_eq!(cjk_font_path(), system_cjk_font_path());
}

#[test]
fn encode_hex_unknown_char_errors_when_font_available() {
    let _guard = lock_cjk_test_env();
    if system_cjk_font_path().is_none() {
        return;
    }
    let mut chars = BTreeSet::new();
    chars.insert('あ');
    let embed = CjkFontEmbed::build(&chars).unwrap();
    let err = match embed.encode_hex("あX") {
        Err(e) => e,
        Ok(_) => panic!("expected missing CID error"),
    };
    assert!(matches!(err, PdfError::InvalidShape(_)));
}

#[test]
fn nonexistent_reciplexa_cjk_font_falls_through() {
    with_saved_cjk_font_env(|| {
        std::env::set_var("RECIPLEXA_CJK_FONT", r"D:\__reciplexa_no_such_font__.ttf");
        let path = system_cjk_font_path();
        if let Some(p) = &path {
            assert!(!p.to_string_lossy().contains("__reciplexa_no_such_font__"));
        }
    });
}

#[test]
fn invalid_font_file_errors_on_build() {
    with_saved_cjk_font_env(|| {
        let dir = std::env::temp_dir();
        let bad = dir.join(format!("reciplexa_bad_font_{}.ttf", std::process::id()));
        std::fs::write(&bad, b"not-a-font").unwrap();
        std::env::set_var("RECIPLEXA_CJK_FONT", &bad);
        let mut chars = BTreeSet::new();
        chars.insert('日');
        let err = match CjkFontEmbed::build(&chars) {
            Err(e) => e,
            Ok(_) => panic!("expected invalid font error"),
        };
        assert!(matches!(err, PdfError::InvalidShape(_)));
        let _ = std::fs::remove_file(&bad);
    });
}

#[test]
fn subset_tag_empty_input_still_deterministic() {
    let a = subset_tag(b"");
    let b = subset_tag(b"");
    assert_eq!(a, b);
    assert_eq!(a.len(), 6);
}

#[test]
fn control_chars_skipped_in_build_when_font_available() {
    let _guard = lock_cjk_test_env();
    if system_cjk_font_path().is_none() {
        return;
    }
    let mut chars = BTreeSet::new();
    chars.insert('あ');
    chars.insert('\u{0009}');
    let embed = CjkFontEmbed::build(&chars).unwrap();
    assert!(embed.encode_hex("あ").is_ok());
}
