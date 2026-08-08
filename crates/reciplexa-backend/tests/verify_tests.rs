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
