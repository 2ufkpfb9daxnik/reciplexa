//! Provenance-aware edits between document model and source (Phase 4 §6.4).

use reciplexa_source::range::TextRange;
use reciplexa_syntax::edit::format_drag_number;
use reciplexa_syntax::{parse_source, SyntaxKind};

use crate::bridge::ApplyEdit;
use crate::property::LayoutBox;
use crate::snapshot::DocumentSnapshot;
use crate::transaction::{TransactionBuilder, TransactionError, TransactionOutcome};

/// Outcome of attempting to mirror a document edit into source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceSyncOutcome {
    /// Document changed; source left unchanged (no provenance or non-literal).
    DocumentOnly(TransactionOutcome),
    /// Document and source both updated.
    SourceUpdated {
        document: TransactionOutcome,
        new_source: String,
    },
    /// Edit rejected before touching document or source.
    Blocked(SourceSyncBlockReason),
}

/// Why a GUI edit cannot be written back to source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceSyncBlockReason {
    NoProvenance,
    MissingLayout,
    NonInvertibleExpression,
    ParseError,
}

/// Apply a GUI edit to the snapshot and, when provenance allows, rewrite source literals.
pub fn apply_provenance_edit(
    snap: &mut DocumentSnapshot,
    source: &str,
    edit: ApplyEdit,
) -> Result<SourceSyncOutcome, TransactionError> {
    let node = match &edit {
        ApplyEdit::Move { node, .. }
        | ApplyEdit::Resize { node, .. }
        | ApplyEdit::SetText { node, .. } => *node,
    };

    let prov = snap
        .provenance
        .get(node)
        .ok_or(TransactionError::UnknownNode(node))?
        .clone();

    let mut tx = TransactionBuilder::new();
    match &edit {
        ApplyEdit::Move { x, y, .. } => {
            let Some(layout) = snap.nodes.get(node).and_then(|n| n.layout()) else {
                return Err(TransactionError::UnknownNode(node));
            };
            tx.set_layout(node, LayoutBox::new(*x, *y, layout.width, layout.height));
        }
        ApplyEdit::Resize { width, height, .. } => {
            let Some(layout) = snap.nodes.get(node).and_then(|n| n.layout()) else {
                return Err(TransactionError::UnknownNode(node));
            };
            tx.set_layout(node, LayoutBox::new(layout.x, layout.y, *width, *height));
        }
        ApplyEdit::SetText { text, .. } => {
            tx.set_text(node, text.clone());
        }
    }

    let doc_outcome = tx.into_transaction().apply(snap)?;

    if prov.text_range == TextRange::EMPTY {
        return Ok(SourceSyncOutcome::DocumentOnly(doc_outcome));
    }

    match sync_literal(source, prov.text_range, &edit) {
        Ok(new_source) => Ok(SourceSyncOutcome::SourceUpdated {
            document: doc_outcome,
            new_source,
        }),
        Err(SourceSyncBlockReason::NonInvertibleExpression) => {
            Ok(SourceSyncOutcome::DocumentOnly(doc_outcome))
        }
        Err(reason) => Ok(SourceSyncOutcome::Blocked(reason)),
    }
}

fn sync_literal(
    source: &str,
    range: TextRange,
    edit: &ApplyEdit,
) -> Result<String, SourceSyncBlockReason> {
    let start: usize = range.start().into();
    let end: usize = range.end().into();
    if start >= end || end > source.len() {
        return Err(SourceSyncBlockReason::NonInvertibleExpression);
    }

    let parse = parse_source(source);
    if !parse.errors.is_empty() {
        return Err(SourceSyncBlockReason::ParseError);
    }

    let new_literal = match edit {
        ApplyEdit::Move { x, y, .. } => {
            format!("{} {}", format_drag_number(*x), format_drag_number(*y))
        }
        ApplyEdit::Resize { width, height, .. } => format!(
            "{} {}",
            format_drag_number(*width),
            format_drag_number(*height)
        ),
        ApplyEdit::SetText { text, .. } => format!("\"{}\"", escape_string(text)),
    };

    let mut out = String::with_capacity(source.len() + new_literal.len());
    out.push_str(&source[..start]);
    out.push_str(&new_literal);
    out.push_str(&source[end..]);

    // Sanity: replacement should re-parse.
    let reparsed = parse_source(&out);
    if !reparsed.errors.is_empty() {
        return Err(SourceSyncBlockReason::NonInvertibleExpression);
    }
    let _ = reparsed
        .root
        .descendants_with_tokens()
        .find_map(|el| el.into_token())
        .filter(|t| t.kind() == SyntaxKind::Number || t.kind() == SyntaxKind::String);
    Ok(out)
}

fn escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(c),
        }
    }
    out
}
