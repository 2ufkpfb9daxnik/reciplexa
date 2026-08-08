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
