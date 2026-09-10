//! Direct Native v2 typed exports for `document/page` (DN2-5).

use std::collections::BTreeMap;

use reciplexa_core::ty::{CoreType, EffectRow};
use reciplexa_eval::domain_native::{DocumentPageOp, DomainNativeOp};

use crate::domain_native::{DomainNativeExport, DomainNativeModule};

fn dynamic() -> CoreType {
    CoreType::Dynamic(Box::new(CoreType::Any))
}

fn nullary_dynamic() -> CoreType {
    CoreType::Fun {
        args: vec![],
        ret: Box::new(dynamic()),
        effects: EffectRow::default(),
    }
}

fn fun_n(n: usize) -> CoreType {
    CoreType::Fun {
        args: vec![dynamic(); n],
        ret: Box::new(dynamic()),
        effects: EffectRow::default(),
    }
}

fn insert(
    exports: &mut BTreeMap<String, DomainNativeExport>,
    name: &str,
    ty: CoreType,
    op: DomainNativeOp,
) {
    exports.insert(name.into(), DomainNativeExport::new(name, ty, op));
}

/// Attach typed DN2 exports for `document/page`.
pub fn populate_document_page_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::DocumentPage(sub);
    insert(
        &mut exports,
        "a4",
        nullary_dynamic(),
        op(DocumentPageOp::A4),
    );
    insert(
        &mut exports,
        "letter",
        nullary_dynamic(),
        op(DocumentPageOp::Letter),
    );
    insert(
        &mut exports,
        "a5",
        nullary_dynamic(),
        op(DocumentPageOp::A5),
    );
    insert(
        &mut exports,
        "a3",
        nullary_dynamic(),
        op(DocumentPageOp::A3),
    );
    insert(
        &mut exports,
        "legal",
        nullary_dynamic(),
        op(DocumentPageOp::Legal),
    );
    insert(&mut exports, "page", fun_n(2), op(DocumentPageOp::Page));
    insert(
        &mut exports,
        "page-framed",
        fun_n(3),
        op(DocumentPageOp::PageFramed),
    );
    insert(
        &mut exports,
        "margins",
        fun_n(4),
        op(DocumentPageOp::Margins),
    );
    insert(
        &mut exports,
        "pagebreak",
        nullary_dynamic(),
        op(DocumentPageOp::PageBreak),
    );
    insert(
        &mut exports,
        "block-pagebreak",
        nullary_dynamic(),
        op(DocumentPageOp::BlockPageBreak),
    );
    insert(&mut exports, "flow", fun_n(1), op(DocumentPageOp::Flow));
    insert(
        &mut exports,
        "section",
        fun_n(2),
        op(DocumentPageOp::Section),
    );
    insert(
        &mut exports,
        "heading",
        fun_n(2),
        op(DocumentPageOp::Heading),
    );
    insert(
        &mut exports,
        "paragraph",
        fun_n(1),
        op(DocumentPageOp::Paragraph),
    );
    insert(
        &mut exports,
        "paragraph-indented",
        fun_n(2),
        op(DocumentPageOp::ParagraphIndented),
    );
    insert(
        &mut exports,
        "columns",
        fun_n(4),
        op(DocumentPageOp::Columns),
    );
    insert(
        &mut exports,
        "unordered-list",
        fun_n(1),
        op(DocumentPageOp::UnorderedList),
    );
    insert(
        &mut exports,
        "ordered-list",
        fun_n(1),
        op(DocumentPageOp::OrderedList),
    );
    insert(
        &mut exports,
        "list-item",
        fun_n(1),
        op(DocumentPageOp::ListItem),
    );
    insert(&mut exports, "table", fun_n(2), op(DocumentPageOp::Table));
    insert(&mut exports, "figure", fun_n(2), op(DocumentPageOp::Figure));
    insert(&mut exports, "note", fun_n(1), op(DocumentPageOp::Note));
    insert(&mut exports, "spacer", fun_n(1), op(DocumentPageOp::Spacer));
    insert(
        &mut exports,
        "block-heading",
        fun_n(1),
        op(DocumentPageOp::BlockHeading),
    );
    insert(
        &mut exports,
        "block-paragraph",
        fun_n(1),
        op(DocumentPageOp::BlockParagraph),
    );
    insert(
        &mut exports,
        "block-columns",
        fun_n(1),
        op(DocumentPageOp::BlockColumns),
    );
    insert(
        &mut exports,
        "block-list",
        fun_n(1),
        op(DocumentPageOp::BlockList),
    );
    insert(
        &mut exports,
        "block-table",
        fun_n(1),
        op(DocumentPageOp::BlockTable),
    );
    insert(
        &mut exports,
        "block-figure",
        fun_n(1),
        op(DocumentPageOp::BlockFigure),
    );
    insert(
        &mut exports,
        "block-note",
        fun_n(1),
        op(DocumentPageOp::BlockNote),
    );
    insert(
        &mut exports,
        "block-spacer",
        fun_n(1),
        op(DocumentPageOp::BlockSpacer),
    );
    module.typed_exports = exports;
}
