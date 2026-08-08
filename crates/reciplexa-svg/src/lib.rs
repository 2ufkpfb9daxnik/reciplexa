//! SVG export from a scene [`Document`] via the Phase 7 backend pipeline.
//!
//! This crate is a thin host-facing adapter over `reciplexa-backend`. The
//! legacy flatten→SVG path has been replaced so CLI and GUI share planning,
//! emission, and artifact validation.

#![forbid(unsafe_code)]

use std::io::{self, Write};

use reciplexa_backend::{
    export_scene_to_svg_with_hints, BackendCapability, ExportError, OutputProfile,
};
use reciplexa_scene::Document;
use reciplexa_visual_ir::NodeSourceHint;

/// Write one SVG document through the planned backend pipeline.
pub fn write_document(doc: &Document, mut out: impl Write) -> io::Result<()> {
    let svg = document_to_svg(doc).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    out.write_all(svg.as_bytes())
}

/// Write SVG with paint-order source provenance hints.
pub fn write_document_with_hints(
    doc: &Document,
    hints: &[Option<NodeSourceHint>],
    mut out: impl Write,
) -> io::Result<()> {
    let svg = document_to_svg_with_hints(doc, hints)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    out.write_all(svg.as_bytes())
}

/// Render `doc` to an SVG string (millimeter units) via planning + validation.
pub fn document_to_svg(doc: &Document) -> Result<String, String> {
    document_to_svg_with_hints(doc, &[])
}

/// Render with paint-order provenance hints threaded into artifact provenance.
pub fn document_to_svg_with_hints(
    doc: &Document,
    hints: &[Option<NodeSourceHint>],
) -> Result<String, String> {
    let artifact = export_scene_to_svg_with_hints(
        doc,
        &BackendCapability::svg_default(),
        &OutputProfile::svg_default(),
        hints,
    )
    .map_err(fmt_export_error)?;
    Ok(artifact.svg)
}

pub fn fmt_export_error(e: ExportError) -> String {
    match e {
        ExportError::Render(e) => format!("render: {e:?}"),
        ExportError::Planning(e) => format!("planning: {e:?}"),
        ExportError::Emit(e) => format!("emit: {e:?}"),
        ExportError::Artifact(e) => format!("artifact: {e:?}"),
    }
}
