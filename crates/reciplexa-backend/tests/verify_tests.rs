use reciplexa_backend::loss::{LossDisposition, LossReport, OutputLoss, OutputLossKind};
use reciplexa_backend::profile::{OutputProfile, ProfileKind};
use reciplexa_backend::verify::*;

#[test]
fn accepts_minimal_svg() {
    let svg = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm" viewBox="0 0 10 10">
</svg>"#;
    assert!(validate_svg_artifact(svg).is_ok());
}

#[test]
fn accepts_page_group_without_page_one() {
    let svg = r#"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm" viewBox="0 0 10 10">
  <g id="page-2"></g>
</svg>"#;
    assert!(validate_svg_artifact(svg).is_ok());
}

#[test]
fn rejects_missing_viewbox() {
    let svg = r#"<?xml version="1.0"?><svg xmlns="http://www.w3.org/2000/svg"></svg>"#;
    assert!(matches!(
        validate_svg_artifact(svg),
        Err(ArtifactValidationError::MissingViewBox)
    ));
}

#[test]
fn rejects_missing_xml_declaration() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"></svg>"#;
    assert!(matches!(
        validate_svg_artifact(svg),
        Err(ArtifactValidationError::MissingXmlDeclaration)
    ));
}

#[test]
fn rejects_missing_svg_root() {
    let svg = r#"<?xml version="1.0"?><g></g>"#;
    assert!(matches!(
        validate_svg_artifact(svg),
        Err(ArtifactValidationError::MissingSvgRoot)
    ));
}

#[test]
fn rejects_missing_page_group() {
    let svg = r#"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm" viewBox="0 0 10 10">
  <g id="content"></g>
</svg>"#;
    assert!(matches!(
        validate_svg_artifact(svg),
        Err(ArtifactValidationError::MissingPageGroup)
    ));
}

#[test]
fn rejects_contains_nan() {
    let svg = r#"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm" viewBox="0 0 10 10">
  <rect x="NaN" y="0" width="1" height="1"/>
</svg>"#;
    assert!(matches!(
        validate_svg_artifact(svg),
        Err(ArtifactValidationError::ContainsNaN)
    ));
}

#[test]
fn rejects_contains_infinity() {
    let svg = r#"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm" viewBox="0 0 10 10">
  <rect x="Infinity" y="0" width="1" height="1"/>
</svg>"#;
    assert!(matches!(
        validate_svg_artifact(svg),
        Err(ArtifactValidationError::ContainsNaN)
    ));
}

#[test]
fn rejects_unbalanced_tags() {
    let svg = r#"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm" viewBox="0 0 10 10">
<svg xmlns="http://www.w3.org/2000/svg" width="5mm" height="5mm" viewBox="0 0 5 5">
</svg>"#;
    assert!(matches!(
        validate_svg_artifact(svg),
        Err(ArtifactValidationError::UnbalancedTags)
    ));
}

#[test]
fn accepts_valid_png_signature() {
    let mut png = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    png.extend_from_slice(&[0, 0, 0, 0]);
    assert!(validate_png_artifact(&png).is_ok());
}

#[test]
fn rejects_empty_png() {
    assert!(matches!(
        validate_png_artifact(&[]),
        Err(ArtifactValidationError::EmptyPng)
    ));
}

#[test]
fn rejects_bad_png_signature() {
    assert!(matches!(
        validate_png_artifact(b"not a png"),
        Err(ArtifactValidationError::InvalidPngSignature)
    ));
}

#[test]
fn raster_loss_report_must_match_profile_kind() {
    let profile = OutputProfile::preview_raster();
    let mut report = LossReport::empty(ProfileKind::Final);
    report.push(OutputLoss {
        kind: OutputLossKind::SemanticText,
        disposition: LossDisposition::Report,
        profile: ProfileKind::Final,
        detail: "x".into(),
    });
    assert!(matches!(
        validate_raster_loss_report(&report, &profile),
        Err(ArtifactValidationError::LossProfileMismatch)
    ));
    let ok = LossReport::empty(ProfileKind::Preview);
    assert!(validate_raster_loss_report(&ok, &profile).is_ok());
}
