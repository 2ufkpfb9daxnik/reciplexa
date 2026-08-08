//! Output profile snapshot for a planned export.

/// User-selected output constraints recorded at planning time.
#[derive(Debug, Clone, PartialEq)]
pub struct OutputProfile {
    pub font_family: String,
    pub ellipse_sides: u32,
    pub requires_text: bool,
    pub requires_images: bool,
}

impl OutputProfile {
    pub fn svg_default() -> Self {
        Self {
            font_family: "sans-serif".into(),
            ellipse_sides: 32,
            requires_text: true,
            requires_images: true,
        }
    }
}
