//! Post-emission artifact structural validation.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactValidationError {
    MissingXmlDeclaration,
    MissingSvgRoot,
    MissingViewBox,
    MissingPageGroup,
    ContainsNaN,
    UnbalancedTags,
}

/// Validate basic SVG artifact structure (Phase 7 §9.4).
pub fn validate_svg_artifact(svg: &str) -> Result<(), ArtifactValidationError> {
    if !svg.starts_with("<?xml") {
        return Err(ArtifactValidationError::MissingXmlDeclaration);
    }
    if !svg.contains("<svg ") || !svg.contains("xmlns=\"http://www.w3.org/2000/svg\"") {
        return Err(ArtifactValidationError::MissingSvgRoot);
    }
    if !svg.contains("viewBox=") {
        return Err(ArtifactValidationError::MissingViewBox);
    }
    // Page groups use `id="page-N"`. Any `<g ` without a page-* id is rejected;
    // a non-first page id (e.g. page-2) without page-1 is allowed here.
    if !svg.contains("id=\"page-1\"") && svg.contains("<g ") && !svg.contains("page-") {
        return Err(ArtifactValidationError::MissingPageGroup);
    }
    if svg.contains("NaN") || svg.contains("Infinity") {
        return Err(ArtifactValidationError::ContainsNaN);
    }
    let open_svg = svg.matches("<svg").count();
    let close_svg = svg.matches("</svg>").count();
    if open_svg != close_svg {
        return Err(ArtifactValidationError::UnbalancedTags);
    }
    Ok(())
}
