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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::Representation;

    #[test]
    fn supports_basic_shapes() {
        let cap = BackendCapability::svg_default();
        assert!(cap.supports(Representation::SvgRect));
        assert!(cap.supports(Representation::SvgCircle));
        assert!(cap.supports(Representation::SvgPolygon));
        assert!(cap.supports(Representation::SvgPath));
    }

    #[test]
    fn supports_text_when_enabled() {
        let mut cap = BackendCapability::svg_default();
        cap.svg_text = true;
        assert!(cap.supports(Representation::SvgText));
        cap.svg_text = false;
        assert!(!cap.supports(Representation::SvgText));
    }

    #[test]
    fn supports_image_when_enabled() {
        let mut cap = BackendCapability::svg_default();
        cap.svg_images = true;
        assert!(cap.supports(Representation::SvgImage));
        cap.svg_images = false;
        assert!(!cap.supports(Representation::SvgImage));
    }

    #[test]
    fn profile_violation_ellipse_sides_too_low() {
        let cap = BackendCapability::svg_default();
        let mut profile = OutputProfile::svg_default();
        profile.ellipse_sides = 4;
        let err = check_capability_profile(&cap, &profile).unwrap_err();
        assert!(matches!(err, PlanningError::ProfileViolation(_)));
    }

    #[test]
    fn profile_violation_ellipse_sides_too_high() {
        let cap = BackendCapability::svg_default();
        let mut profile = OutputProfile::svg_default();
        profile.ellipse_sides = 128;
        let err = check_capability_profile(&cap, &profile).unwrap_err();
        assert!(matches!(err, PlanningError::ProfileViolation(_)));
    }

    #[test]
    fn capability_mismatch_when_text_required_but_unsupported() {
        let mut cap = BackendCapability::svg_default();
        cap.svg_text = false;
        let mut profile = OutputProfile::svg_default();
        profile.requires_text = true;
        let err = check_capability_profile(&cap, &profile).unwrap_err();
        assert!(matches!(err, PlanningError::CapabilityMismatch(_)));
    }

    #[test]
    fn ok_when_profile_within_capability() {
        let cap = BackendCapability::svg_default();
        let profile = OutputProfile::svg_default();
        assert!(check_capability_profile(&cap, &profile).is_ok());
    }
}
