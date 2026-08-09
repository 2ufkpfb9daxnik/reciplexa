use reciplexa_backend::capability::*;
use reciplexa_backend::loss::LossDisposition;
use reciplexa_backend::plan::Representation;
use reciplexa_backend::profile::{OutputProfile, ProfileKind};

#[test]
fn supports_basic_shapes() {
    let cap = BackendCapability::svg_default();
    assert_eq!(cap.family, BackendFamily::Svg);
    assert!(cap.supports(Representation::SvgRect));
    assert!(cap.supports(Representation::SvgCircle));
    assert!(cap.supports(Representation::SvgPolygon));
    assert!(cap.supports(Representation::SvgPath));
    assert!(!cap.supports(Representation::RasterCircle));
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

#[test]
fn raster_capability_supports_raster_reprs() {
    let cap = BackendCapability::raster_default();
    assert_eq!(cap.family, BackendFamily::Raster);
    assert!(cap.supports(Representation::RasterCircle));
    assert!(cap.supports(Representation::RasterPolygon));
    assert!(cap.supports(Representation::RasterPath));
    assert!(cap.supports(Representation::RasterTextOmit));
    assert!(cap.supports(Representation::RasterImageOmit));
    assert!(!cap.supports(Representation::SvgRect));
}

#[test]
fn preview_vs_final_raster_profiles_differ_visibly() {
    let preview = OutputProfile::preview_raster();
    let final_p = OutputProfile::final_raster();
    assert_eq!(preview.kind, ProfileKind::Preview);
    assert_eq!(final_p.kind, ProfileKind::Final);
    assert!(preview.px_per_mm < final_p.px_per_mm);
    assert_eq!(preview.text_omit_disposition, LossDisposition::Report);
    assert_eq!(
        final_p.text_omit_disposition,
        LossDisposition::RequireExplicitApproval
    );
    assert!(preview.forbid_silent_loss && final_p.forbid_silent_loss);
}

#[test]
fn raster_preview_profile_accepted() {
    let cap = BackendCapability::raster_default();
    assert!(check_capability_profile(&cap, &OutputProfile::preview_raster()).is_ok());
}

#[test]
fn raster_final_profile_accepted_when_text_not_hard_required() {
    let cap = BackendCapability::raster_default();
    assert!(check_capability_profile(&cap, &OutputProfile::final_raster()).is_ok());
}

#[test]
fn raster_rejects_silent_loss_profile() {
    let cap = BackendCapability::raster_default();
    let mut profile = OutputProfile::preview_raster();
    profile.forbid_silent_loss = false;
    let err = check_capability_profile(&cap, &profile).unwrap_err();
    assert!(matches!(err, PlanningError::ProfileViolation(_)));
}

#[test]
fn raster_final_hard_requires_text_mismatches() {
    let cap = BackendCapability::raster_default();
    let mut profile = OutputProfile::final_raster();
    profile.requires_text = true;
    let err = check_capability_profile(&cap, &profile).unwrap_err();
    assert!(matches!(err, PlanningError::CapabilityMismatch(_)));
}

#[test]
fn raster_rejects_px_per_mm_out_of_range() {
    let cap = BackendCapability::raster_default();
    let mut profile = OutputProfile::preview_raster();
    profile.px_per_mm = 0.0;
    let err = check_capability_profile(&cap, &profile).unwrap_err();
    assert!(matches!(err, PlanningError::ProfileViolation(_)));
}
