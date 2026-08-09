//! Backend capability declarations.

use crate::plan::Representation;
use crate::profile::{OutputProfile, ProfileKind};

/// Backend family for capability checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendFamily {
    Svg,
    Raster,
}

/// What a backend can represent natively.
#[derive(Debug, Clone, PartialEq)]
pub struct BackendCapability {
    pub family: BackendFamily,
    pub svg_text: bool,
    pub svg_images: bool,
    pub max_ellipse_polygon_sides: u32,
    pub min_ellipse_polygon_sides: u32,
    /// Raster can emit selectable / shaped text (currently false → planned TextSkipped loss).
    pub raster_text: bool,
    /// Raster can embed source images (currently false → planned ImageSkipped loss).
    pub raster_images: bool,
    pub min_px_per_mm: f64,
    pub max_px_per_mm: f64,
}

impl BackendCapability {
    pub fn svg_default() -> Self {
        Self {
            family: BackendFamily::Svg,
            svg_text: true,
            svg_images: true,
            max_ellipse_polygon_sides: 64,
            min_ellipse_polygon_sides: 8,
            raster_text: false,
            raster_images: false,
            min_px_per_mm: 1.0,
            max_px_per_mm: 48.0,
        }
    }

    pub fn raster_default() -> Self {
        Self {
            family: BackendFamily::Raster,
            svg_text: false,
            svg_images: false,
            max_ellipse_polygon_sides: 64,
            min_ellipse_polygon_sides: 8,
            raster_text: false,
            raster_images: false,
            min_px_per_mm: 1.0,
            max_px_per_mm: 48.0,
        }
    }

    pub fn supports(&self, repr: Representation) -> bool {
        match repr {
            Representation::SvgCircle
            | Representation::SvgRect
            | Representation::SvgPolygon
            | Representation::SvgPath => self.family == BackendFamily::Svg,
            Representation::SvgText => self.family == BackendFamily::Svg && self.svg_text,
            Representation::SvgImage => self.family == BackendFamily::Svg && self.svg_images,
            Representation::RasterCircle
            | Representation::RasterPolygon
            | Representation::RasterPath => self.family == BackendFamily::Raster,
            Representation::RasterTextOmit => self.family == BackendFamily::Raster,
            Representation::RasterImageOmit => self.family == BackendFamily::Raster,
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
    match cap.family {
        BackendFamily::Svg => check_svg_profile(cap, profile),
        BackendFamily::Raster => check_raster_profile(cap, profile),
    }
}

fn check_svg_profile(
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

fn check_raster_profile(
    cap: &BackendCapability,
    profile: &OutputProfile,
) -> Result<(), PlanningError> {
    if !profile.forbid_silent_loss {
        return Err(PlanningError::ProfileViolation(
            "raster profiles must set forbid_silent_loss (Preview vs Final losses never silent)"
                .into(),
        ));
    }
    if !profile.px_per_mm.is_finite()
        || profile.px_per_mm < cap.min_px_per_mm
        || profile.px_per_mm > cap.max_px_per_mm
    {
        return Err(PlanningError::ProfileViolation(format!(
            "px_per_mm {} outside capability range {}..{}",
            profile.px_per_mm, cap.min_px_per_mm, cap.max_px_per_mm
        )));
    }
    // Final requires native text when the profile demands it; Preview may omit with Loss.
    if profile.kind == ProfileKind::Final && profile.requires_text && !cap.raster_text {
        return Err(PlanningError::CapabilityMismatch(vec![
            CapabilityMismatch {
                feature: "raster_text".into(),
                required_by_profile: true,
            },
        ]));
    }
    if profile.kind == ProfileKind::Final && profile.requires_images && !cap.raster_images {
        return Err(PlanningError::CapabilityMismatch(vec![
            CapabilityMismatch {
                feature: "raster_images".into(),
                required_by_profile: true,
            },
        ]));
    }
    if profile.ellipse_sides < cap.min_ellipse_polygon_sides
        || profile.ellipse_sides > cap.max_ellipse_polygon_sides
    {
        return Err(PlanningError::ProfileViolation(format!(
            "ellipse_sides {} outside capability range {}..{}",
            profile.ellipse_sides, cap.min_ellipse_polygon_sides, cap.max_ellipse_polygon_sides
        )));
    }
    Ok(())
}
