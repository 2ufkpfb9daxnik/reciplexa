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
    if !svg.contains("id=\"page-1\"") && svg.contains("<g ") {
        // allow empty doc without pages
        if svg.contains("page-") {
            // ok
        } else if svg.matches("<g ").count() > 0 {
            return Err(ArtifactValidationError::MissingPageGroup);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_minimal_svg() {
        let svg = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm" viewBox="0 0 10 10">
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
}
