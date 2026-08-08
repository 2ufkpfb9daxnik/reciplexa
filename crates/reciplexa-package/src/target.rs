//! Build targets and runtime profiles.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildTarget {
    Document,
    Slide,
    Preview,
    Native,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeProfile {
    pub name: String,
    pub effects_allowed: bool,
    pub native_allowed: bool,
}

impl RuntimeProfile {
    pub fn document() -> Self {
        Self {
            name: "document".into(),
            effects_allowed: false,
            native_allowed: false,
        }
    }

    pub fn native_preview() -> Self {
        Self {
            name: "native-preview".into(),
            effects_allowed: true,
            native_allowed: true,
        }
    }
}

impl BuildTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Slide => "slide",
            Self::Preview => "preview",
            Self::Native => "native",
        }
    }

    pub fn validate_entry(&self, entry: &str) -> Result<(), String> {
        if entry.is_empty() {
            return Err("entry module name must not be empty".into());
        }
        match self {
            Self::Document | Self::Slide | Self::Preview => Ok(()),
            Self::Native if entry.contains("native") => Ok(()),
            Self::Native => Err("native target requires native entry binding".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_covers_all_variants() {
        assert_eq!(BuildTarget::Document.as_str(), "document");
        assert_eq!(BuildTarget::Slide.as_str(), "slide");
        assert_eq!(BuildTarget::Preview.as_str(), "preview");
        assert_eq!(BuildTarget::Native.as_str(), "native");
    }

    #[test]
    fn native_entry_containing_native_is_ok() {
        assert!(BuildTarget::Native
            .validate_entry("main_native.rpx")
            .is_ok());
        assert!(BuildTarget::Native.validate_entry("native").is_ok());
    }

    #[test]
    fn empty_entry_fails_all_targets() {
        for target in [
            BuildTarget::Document,
            BuildTarget::Slide,
            BuildTarget::Preview,
            BuildTarget::Native,
        ] {
            assert!(target.validate_entry("").is_err());
        }
    }

    #[test]
    fn native_rejects_non_native_entry() {
        let err = BuildTarget::Native.validate_entry("main.rpx").unwrap_err();
        assert!(err.contains("native entry"));
    }

    #[test]
    fn document_slide_preview_accept_any_nonempty_entry() {
        for target in [
            BuildTarget::Document,
            BuildTarget::Slide,
            BuildTarget::Preview,
        ] {
            assert!(target.validate_entry("main.rpx").is_ok());
        }
    }

    #[test]
    fn runtime_profiles_differ_on_native() {
        let doc = RuntimeProfile::document();
        let native = RuntimeProfile::native_preview();
        assert!(!doc.native_allowed);
        assert!(native.native_allowed);
        assert!(!doc.effects_allowed);
        assert!(native.effects_allowed);
    }
}
