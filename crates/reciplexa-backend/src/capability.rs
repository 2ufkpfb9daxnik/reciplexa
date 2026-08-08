//! Backend capability declarations.

use crate::plan::Representation;
use crate::profile::OutputProfile;

/// What a backend can represent natively.
#[derive(Debug, Clone, PartialEq)]
pub struct BackendCapability {
    pub svg_text: bool,
    pub svg_images: bool,
    pub max_ellipse_polygon_sides: u32,
    pub min_ellipse_polygon_sides: u32,
}

impl BackendCapability {
    pub fn svg_default() -> Self {
        Self {
            svg_text: true,
            svg_images: true,
            max_ellipse_polygon_sides: 64,
            min_ellipse_polygon_sides: 8,
        }
    }

    pub fn supports(&self, repr: Representation) -> bool {
        match repr {
            Representation::SvgCircle
            | Representation::SvgRect
            | Representation::SvgPolygon
            | Representation::SvgPath => true,
            Representation::SvgText => self.svg_text,
            Representation::SvgImage => self.svg_images,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityMismatch {
    pub feature: String,
    pub required_by_profile: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanningError {
    CapabilityMismatch(Vec<CapabilityMismatch>),
    ProfileViolation(String),
}

/// Check profile requirements against capability before planning.
pub fn check_capability_profile(
    cap: &BackendCapability,
    profile: &OutputProfile,
) -> Result<(), PlanningError> {
    let mut mismatches = Vec::new();
    if profile.requires_text && !cap.svg_text {
        mismatches.push(CapabilityMismatch {
            feature: "svg_text".into(),
            required_by_profile: true,
        });
    }
    if profile.ellipse_sides < cap.min_ellipse_polygon_sides
        || profile.ellipse_sides > cap.max_ellipse_polygon_sides
    {
        return Err(PlanningError::ProfileViolation(format!(
            "ellipse_sides {} outside capability range {}..{}",
            profile.ellipse_sides, cap.min_ellipse_polygon_sides, cap.max_ellipse_polygon_sides
        )));
    }
    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(PlanningError::CapabilityMismatch(mismatches))
    }
}
