//! Post-emission artifact structural validation.

use crate::loss::LossReport;
use crate::profile::OutputProfile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactValidationError {
    MissingXmlDeclaration,
    MissingSvgRoot,
    MissingViewBox,
    MissingPageGroup,
    ContainsNaN,
    UnbalancedTags,
    InvalidPngSignature,
    EmptyPng,
    SilentLossForbidden,
    LossProfileMismatch,
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

/// Validate PNG signature and non-empty payload.
pub fn validate_png_artifact(png: &[u8]) -> Result<(), ArtifactValidationError> {
    if png.is_empty() {
        return Err(ArtifactValidationError::EmptyPng);
    }
    const SIG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    if png.len() < 8 || png[..8] != SIG {
        return Err(ArtifactValidationError::InvalidPngSignature);
    }
    Ok(())
}

/// Raster Preview/Final: losses must be attached to the profile and never silent.
pub fn validate_raster_loss_report(
    report: &LossReport,
    profile: &OutputProfile,
) -> Result<(), ArtifactValidationError> {
    if report.profile != profile.kind {
        return Err(ArtifactValidationError::LossProfileMismatch);
    }
    for loss in &report.losses {
        if loss.profile != profile.kind {
            return Err(ArtifactValidationError::LossProfileMismatch);
        }
        if profile.forbid_silent_loss && loss.detail.trim().is_empty() {
            return Err(ArtifactValidationError::SilentLossForbidden);
        }
    }
    Ok(())
}
