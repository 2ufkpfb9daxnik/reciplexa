use reciplexa_backend::capability::*;
use reciplexa_backend::plan::Representation;
use reciplexa_backend::profile::OutputProfile;

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
