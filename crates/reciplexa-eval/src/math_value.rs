//! Light bridge: package `math/*` tagged records → [`MathAtom`] / [`MathBox`].
//!
//! Constructors in `reciplexa-package::domain_bodies` emit
//! `tag: "math-symbol"|"math-row"|"math-fraction"|…` records. This module
//! lowers them for fontless `estimate_box` (document / layout prep).

use std::cell::Cell;

use reciplexa_identity::document::StableNodeId;
use reciplexa_scene::{Document, Page, PaperSize, Shape};
use reciplexa_std::math::{
    layout_math_atom_to_shapes_with_style, scripts_attachment_offsets, EstimateStyle,
    MathAccentKind, MathAtom, MathBox, MathClass, MathMatrixKind, MathStackKind,
};
use reciplexa_text_layout::TypesetEngine;

use crate::value::RuntimeValue;

/// Fail-fast error when a math value cannot become a [`MathAtom`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathValueError {
    pub message: String,
}

impl MathValueError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Lower a package math tagged record to a [`MathAtom`], then estimate a box.
///
/// Optional record field `style` (`"text"` / `"display"`) selects
/// [`EstimateStyle`]; missing / unknown → [`EstimateStyle::Display`].
pub fn estimate_math_box_from_value(v: &RuntimeValue) -> Result<MathBox, MathValueError> {
    estimate_math_box_from_value_with_style(v, estimate_style_from_value(v))
}

/// Estimate with an explicit [`EstimateStyle`] (ignores record `style` field).
///
/// Already-estimated metric records (`math-box` / `math-phantom` / `math-smash`
/// with `width`/`height`/`depth`) are returned as-is; style does not re-scale
/// them.
pub fn estimate_math_box_from_value_with_style(
    v: &RuntimeValue,
    style: EstimateStyle,
) -> Result<MathBox, MathValueError> {
    if let Some(b) = math_box_from_metrics_record(v) {
        return Ok(b);
    }
    Ok(math_atom_from_value(v)?.estimate_box_with_style(style))
}

/// Read optional `style` field (`"text"` / `"display"`) from a math record.
///
/// Accepts [`RuntimeValue::String`] or [`RuntimeValue::ShapeTag`] (kernel surface
/// maps the `"text"` string literal to a shape tag). Unknown / missing → display.
pub fn estimate_style_from_value(v: &RuntimeValue) -> EstimateStyle {
    let Ok(fields) = record_fields(v) else {
        return EstimateStyle::Display;
    };
    let raw = match field(fields, "style") {
        Some(RuntimeValue::String(s)) => Some(s.as_str()),
        Some(RuntimeValue::ShapeTag(s)) => Some(s.as_str()),
        _ => None,
    };
    raw.and_then(EstimateStyle::parse)
        .unwrap_or(EstimateStyle::Display)
}

/// Fontless script attachment offsets from a package `math-scripts` tagged record.
///
/// Lowers via [`math_atom_from_value`], then applies [`scripts_attachment_offsets`]
/// to base / sub / sup estimate boxes. Heuristic only — not OpenType MATH.
pub fn scripts_attachment_offsets_from_value(
    v: &RuntimeValue,
) -> Result<(f64, f64, f64, f64), MathValueError> {
    let atom = math_atom_from_value(v)?;
    match atom {
        MathAtom::Scripts {
            base,
            superscript,
            subscript,
            ..
        } => {
            let base_box = base.estimate_box();
            let sub = subscript.as_ref().map(|s| s.estimate_box());
            let sup = superscript.as_ref().map(|s| s.estimate_box());
            Ok(scripts_attachment_offsets(base_box, sub, sup))
        }
        other => Err(MathValueError::new(format!(
            "scripts_attachment_offsets_from_value expects math-scripts atom, got {:?}",
            std::mem::discriminant(&other)
        ))),
    }
}

/// Lower a package math tagged record to a [`MathAtom`].
pub fn math_atom_from_value(v: &RuntimeValue) -> Result<MathAtom, MathValueError> {
    let mut ids = IdGen::default();
    math_atom_from_value_with_ids(v, &mut ids)
}

/// Very naive: Text glyphs from `linearize`, placed by `estimate_box` width
/// (monospace heuristic) at `origin` (mm). Scripts / BigOp / Fraction / Radical
/// use std attachment heuristics and Line rules (LL7–LL11).
///
/// `layout_style` selects [`EstimateStyle::Display`] vs [`EstimateStyle::Text`]
/// for box estimates (script / limit shrink). `math-phantom` / `math-smash`
/// metric records produce **no ink** (empty shape list). Not TeX `\mathsm@sh`
/// or OpenType MATH — see `lang/implemented-features.md`.
pub fn layout_math_to_shapes(
    math_value: &RuntimeValue,
    origin: (f64, f64),
    layout_style: EstimateStyle,
) -> Result<Vec<Shape>, MathValueError> {
    if let Ok(fields) = record_fields(math_value) {
        if matches!(tag_of(fields), Some("math-phantom" | "math-smash")) {
            return Ok(Vec::new());
        }
    }
    let atom = math_atom_from_value(math_value)?;
    Ok(layout_math_atom_to_shapes_with_style(
        &atom,
        origin,
        layout_style,
    ))
}

/// Font-backed MATH layout (Profile v1). Stub [`layout_math_to_shapes`] is unchanged.
pub fn layout_math_to_shapes_product(
    math_value: &RuntimeValue,
    origin: (f64, f64),
    layout_style: EstimateStyle,
) -> Result<Vec<Shape>, MathValueError> {
    if let Ok(fields) = record_fields(math_value) {
        if matches!(tag_of(fields), Some("math-phantom" | "math-smash")) {
            return Ok(Vec::new());
        }
    }
    let atom = math_atom_from_value(math_value)?;
    let font =
        reciplexa_text_layout::host_math_font().map_err(|e| MathValueError::new(e.to_string()))?;
    let laid = reciplexa_text_layout::layout_math_atom(&font, &atom, layout_style)
        .map_err(|e| MathValueError::new(e.to_string()))?;
    Ok(reciplexa_text_layout::positioned_math_to_shapes(
        &laid,
        origin,
        reciplexa_std::math::MATH_LAYOUT_EM_TO_MM,
        reciplexa_scene::Color::BLACK,
        &font.id,
    ))
}

/// Lower a `math-demo` (or bare math tagged) record to a one-page scene.
///
/// `tree` is laid out with the selected engine. Used by host preview/export so
/// `examples/pkg_math.rpx` consumes the product MATH engine.
pub fn document_from_math_demo_value(
    v: &RuntimeValue,
    engine: TypesetEngine,
) -> Result<Document, MathValueError> {
    let tree = math_demo_tree(v);
    let origin = (20.0, 200.0);
    let style = estimate_style_from_value(tree);
    let shapes = match engine {
        TypesetEngine::Stub => layout_math_to_shapes(tree, origin, style)?,
        TypesetEngine::Product => layout_math_to_shapes_product(tree, origin, style)?,
    };
    Ok(Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes,
    }))
}

fn math_demo_tree(v: &RuntimeValue) -> &RuntimeValue {
    match v {
        RuntimeValue::Record(fields) => fields
            .iter()
            .find(|(k, _)| k == "tree")
            .map(|(_, tree)| tree)
            .unwrap_or(v),
        _ => v,
    }
}

#[derive(Default)]
struct IdGen {
    next: Cell<u64>,
}

impl IdGen {
    fn mint(&self) -> StableNodeId {
        let n = self.next.get();
        self.next.set(n + 1);
        StableNodeId::new(n)
    }
}

fn math_atom_from_value_with_ids(
    v: &RuntimeValue,
    ids: &mut IdGen,
) -> Result<MathAtom, MathValueError> {
    let fields = record_fields(v)?;
    let tag = tag_of(fields).ok_or_else(|| MathValueError::new("math record missing tag"))?;
    match tag {
        "math-absent" => Err(MathValueError::new(
            "math-absent is not a standalone MathAtom",
        )),
        "math-symbol" | "math-textop" => {
            let glyph = string_field(fields, "glyph")?;
            let class = class_field(fields)?;
            Ok(MathAtom::symbol(ids.mint(), glyph, class))
        }
        "math-operatorname" => {
            let name = string_field(fields, "name")?;
            Ok(MathAtom::symbol(ids.mint(), name, MathClass::Operator))
        }
        "math-text" => {
            let body = string_field(fields, "body")?;
            Ok(MathAtom::symbol(ids.mint(), body, MathClass::Ordinary))
        }
        "math-row" => {
            let children_v = field(fields, "children")
                .ok_or_else(|| MathValueError::new("math-row missing children"))?;
            let children = atom_list(children_v, ids)?;
            Ok(MathAtom::row(ids.mint(), children))
        }
        "math-fraction" => {
            let num = required_child(fields, "numerator", ids)?;
            let den = required_child(fields, "denominator", ids)?;
            Ok(MathAtom::fraction(ids.mint(), num, den))
        }
        "math-scripts" => {
            let base = required_child(fields, "base", ids)?;
            let superscript = optional_child(fields, "superscript", ids)?;
            let subscript = optional_child(fields, "subscript", ids)?;
            Ok(MathAtom::scripts(ids.mint(), base, superscript, subscript))
        }
        "math-radical" => {
            let radicand = required_child(fields, "radicand", ids)?;
            match optional_child(fields, "index", ids)? {
                Some(index) => Ok(MathAtom::radical_indexed(ids.mint(), index, radicand)),
                None => Ok(MathAtom::radical(ids.mint(), radicand)),
            }
        }
        "math-delimiter" => {
            let left = string_field(fields, "left")?;
            let right = string_field(fields, "right")?;
            let body = required_child(fields, "body", ids)?;
            let stretch_factor = optional_number_field(fields, "stretch-factor").unwrap_or(1.0);
            Ok(MathAtom::delimiter_with_stretch(
                ids.mint(),
                left,
                right,
                body,
                stretch_factor,
            ))
        }
        "math-accent" => {
            let kind = accent_kind_field(fields)?;
            let base = required_child(fields, "base", ids)?;
            Ok(MathAtom::accent(ids.mint(), kind, base))
        }
        "math-bigop" => {
            let glyph = string_field(fields, "glyph")?;
            let lower = optional_child(fields, "lower", ids)?;
            let upper = optional_child(fields, "upper", ids)?;
            let body = optional_child(fields, "body", ids)?;
            Ok(MathAtom::big_op(ids.mint(), glyph, lower, upper, body))
        }
        "math-bigop-scripts" => {
            // Side-script form of a large op (package `sum-scripts` / `bigop-scripts`).
            let glyph = string_field(fields, "glyph")?;
            let base = MathAtom::symbol(ids.mint(), glyph, MathClass::Operator);
            let superscript = optional_child(fields, "superscript", ids)?;
            let subscript = optional_child(fields, "subscript", ids)?;
            let scripts = MathAtom::scripts(ids.mint(), base, superscript, subscript);
            match optional_child(fields, "body", ids)? {
                Some(body) => Ok(MathAtom::row(ids.mint(), vec![scripts, body])),
                None => Ok(scripts),
            }
        }
        "math-matrix" => {
            let kind = matrix_kind_field(fields)?;
            let rows = matrix_rows_field(fields, ids)?;
            if kind == MathMatrixKind::Delimited {
                let left = string_field(fields, "left").unwrap_or_else(|_| "(".into());
                let right = string_field(fields, "right").unwrap_or_else(|_| ")".into());
                Ok(MathAtom::matrix_delimited(ids.mint(), left, right, rows))
            } else {
                Ok(MathAtom::matrix(ids.mint(), kind, rows))
            }
        }
        "math-aligned" => {
            let rows = matrix_rows_field(fields, ids)?;
            Ok(MathAtom::aligned(ids.mint(), rows))
        }
        "math-stack" => {
            let children_v = field(fields, "children")
                .ok_or_else(|| MathValueError::new("math-stack missing children"))?;
            let children = atom_list(children_v, ids)?;
            let kind = stack_kind_field(fields);
            Ok(MathAtom::stack(ids.mint(), kind, children))
        }
        "math-stackrel" => {
            let relation = required_child(fields, "relation", ids)?;
            let base = required_child(fields, "base", ids)?;
            Ok(MathAtom::stack(
                ids.mint(),
                MathStackKind::Stackrel,
                vec![relation, base],
            ))
        }
        // Package `math/scripts` under / over (brace, set, plain).
        "math-over" => over_under_from_fields(fields, ids, /* over */ true),
        "math-under" => over_under_from_fields(fields, ids, /* over */ false),
        // Metric / tree tags from builtins `math-phantom` / `math-smash`.
        "math-phantom" | "math-smash" => {
            // Metric records are handled by `layout_math_to_shapes` (no ink) and
            // `math_box_from_metrics_record`. Nested tree tags become an empty
            // row so a surrounding row still lays out.
            Ok(MathAtom::row(ids.mint(), vec![]))
        }
        // Package `math/cases` piecewise / delimited cases.
        "math-cases" => {
            let left = string_field(fields, "left").unwrap_or_else(|_| "{".into());
            let right = string_field(fields, "right").unwrap_or_else(|_| "".into());
            let arms_v = field(fields, "arms")
                .ok_or_else(|| MathValueError::new("math-cases missing arms"))?;
            let mut rows = Vec::new();
            for arm in cons_items(arms_v)? {
                rows.push(case_arm_cells(arm, ids)?);
            }
            Ok(MathAtom::matrix_delimited(ids.mint(), left, right, rows))
        }
        "math-case-arm" => {
            let cells = case_arm_cells(v, ids)?;
            Ok(MathAtom::row(ids.mint(), cells))
        }
        // Env / align / substack tags used by `examples/pkg_math.rpx`.
        "math-matrix-env" => {
            let kind = matrix_kind_field(fields).unwrap_or(MathMatrixKind::Plain);
            let rows = matrix_rows_field(fields, ids)?;
            Ok(MathAtom::matrix(ids.mint(), kind, rows))
        }
        "math-align-eq" => {
            let rows = matrix_rows_field(fields, ids)?;
            Ok(MathAtom::aligned(ids.mint(), rows))
        }
        "math-substack" => {
            let rows = matrix_rows_field(fields, ids)?;
            let children: Vec<_> = rows
                .into_iter()
                .map(|cells| MathAtom::row(ids.mint(), cells))
                .collect();
            Ok(MathAtom::stack(
                ids.mint(),
                MathStackKind::Substack,
                children,
            ))
        }
        "math-align" => {
            // Single-body alignment wrapper → just the body.
            required_child(fields, "body", ids)
        }
        "math-align-at" => required_child(fields, "body", ids),
        other => Err(MathValueError::new(format!(
            "unsupported math tag `{other}`"
        ))),
    }
}

fn over_under_from_fields(
    fields: &[(String, RuntimeValue)],
    ids: &mut IdGen,
    over: bool,
) -> Result<MathAtom, MathValueError> {
    let kind = match field(fields, "kind") {
        Some(RuntimeValue::String(s)) => s.as_str(),
        _ => {
            if over {
                "over"
            } else {
                "under"
            }
        }
    };
    let body = required_child(fields, "body", ids)?;
    let label = optional_child(fields, "label", ids)?;
    match (over, kind) {
        (true, "overline") => Ok(MathAtom::accent(ids.mint(), MathAccentKind::Overline, body)),
        (false, "underline") => Ok(MathAtom::accent(
            ids.mint(),
            MathAccentKind::Underline,
            body,
        )),
        (true, "overbrace" | "overset" | "over") => match label {
            Some(lab) => Ok(MathAtom::stack(
                ids.mint(),
                MathStackKind::Stackrel,
                vec![lab, body],
            )),
            None => Ok(MathAtom::accent(ids.mint(), MathAccentKind::Overline, body)),
        },
        (false, "underbrace" | "underset" | "under") => match label {
            Some(lab) => Ok(MathAtom::stack(
                ids.mint(),
                MathStackKind::Stackrel,
                vec![body, lab],
            )),
            None => Ok(MathAtom::accent(
                ids.mint(),
                MathAccentKind::Underline,
                body,
            )),
        },
        _ => match label {
            Some(lab) if over => Ok(MathAtom::stack(
                ids.mint(),
                MathStackKind::Stack,
                vec![lab, body],
            )),
            Some(lab) => Ok(MathAtom::stack(
                ids.mint(),
                MathStackKind::Stack,
                vec![body, lab],
            )),
            None if over => Ok(MathAtom::accent(ids.mint(), MathAccentKind::Overline, body)),
            None => Ok(MathAtom::accent(
                ids.mint(),
                MathAccentKind::Underline,
                body,
            )),
        },
    }
}

fn case_arm_cells(v: &RuntimeValue, ids: &mut IdGen) -> Result<Vec<MathAtom>, MathValueError> {
    let fields = record_fields(v)?;
    match tag_of(fields) {
        Some("math-case-arm") => {
            let body = required_child(fields, "body", ids)?;
            let guard = optional_child(fields, "guard", ids)?;
            let mut cells = vec![body];
            if let Some(g) = guard {
                cells.push(g);
            }
            Ok(cells)
        }
        // Bare atom as a single-cell arm.
        _ => Ok(vec![math_atom_from_value_with_ids(v, ids)?]),
    }
}

fn accent_kind_field(fields: &[(String, RuntimeValue)]) -> Result<MathAccentKind, MathValueError> {
    let s = string_field(fields, "kind")?;
    MathAccentKind::from_str_name(s.as_str())
        .ok_or_else(|| MathValueError::new(format!("unknown accent kind `{s}`")))
}

fn matrix_kind_field(fields: &[(String, RuntimeValue)]) -> Result<MathMatrixKind, MathValueError> {
    let s = match field(fields, "kind") {
        Some(RuntimeValue::String(s)) => s.as_str(),
        Some(_) => return Err(MathValueError::new("math matrix kind must be a string")),
        None => "matrix",
    };
    match s {
        "matrix" | "array" => Ok(MathMatrixKind::Plain),
        "bmatrix" | "delimited-bmatrix" => Ok(MathMatrixKind::BMatrix),
        "pmatrix" | "delimited-pmatrix" => Ok(MathMatrixKind::PMatrix),
        "vmatrix" => Ok(MathMatrixKind::VMatrix),
        "smallmatrix" => Ok(MathMatrixKind::Small),
        "delimited" => Ok(MathMatrixKind::Delimited),
        other => Err(MathValueError::new(format!(
            "unknown matrix kind `{other}`"
        ))),
    }
}

fn stack_kind_field(fields: &[(String, RuntimeValue)]) -> MathStackKind {
    match field(fields, "kind") {
        Some(RuntimeValue::String(s)) => match s.as_str() {
            "atop" => MathStackKind::Atop,
            "substack" => MathStackKind::Substack,
            "stackrel" => MathStackKind::Stackrel,
            _ => MathStackKind::Stack,
        },
        _ => MathStackKind::Stack,
    }
}

fn matrix_rows_field(
    fields: &[(String, RuntimeValue)],
    ids: &mut IdGen,
) -> Result<Vec<Vec<MathAtom>>, MathValueError> {
    let rows_v = field(fields, "rows")
        .ok_or_else(|| MathValueError::new("math matrix/aligned missing rows"))?;
    let mut rows = Vec::new();
    for row_v in cons_items(rows_v)? {
        rows.push(matrix_row_atoms(row_v, ids)?);
    }
    Ok(rows)
}

fn matrix_row_atoms(v: &RuntimeValue, ids: &mut IdGen) -> Result<Vec<MathAtom>, MathValueError> {
    let fields = record_fields(v)?;
    match tag_of(fields) {
        Some("math-matrix-row") | Some("math-align-row") => {
            let cells_v = field(fields, "cells")
                .ok_or_else(|| MathValueError::new("matrix/align row missing cells"))?;
            atom_list(cells_v, ids)
        }
        // Bare cons list of cells is also accepted.
        _ => atom_list(v, ids),
    }
}

fn required_child(
    fields: &[(String, RuntimeValue)],
    name: &str,
    ids: &mut IdGen,
) -> Result<MathAtom, MathValueError> {
    let v = field(fields, name)
        .ok_or_else(|| MathValueError::new(format!("math record missing `{name}`")))?;
    math_atom_from_value_with_ids(v, ids)
}

fn optional_child(
    fields: &[(String, RuntimeValue)],
    name: &str,
    ids: &mut IdGen,
) -> Result<Option<MathAtom>, MathValueError> {
    let Some(v) = field(fields, name) else {
        return Ok(None);
    };
    if is_math_absent(v) || is_no_ink_math_tag(v) {
        return Ok(None);
    }
    Ok(Some(math_atom_from_value_with_ids(v, ids)?))
}

fn is_math_absent(v: &RuntimeValue) -> bool {
    record_fields(v)
        .ok()
        .and_then(|f| tag_of(f))
        .is_some_and(|t| t == "math-absent")
}

fn is_no_ink_math_tag(v: &RuntimeValue) -> bool {
    record_fields(v)
        .ok()
        .and_then(|f| tag_of(f))
        .is_some_and(|t| t == "math-phantom" || t == "math-smash")
}

/// Already-estimated `{tag, width, height, depth}` from `math-box` / phantom / smash.
fn math_box_from_metrics_record(v: &RuntimeValue) -> Option<MathBox> {
    let fields = record_fields(v).ok()?;
    match tag_of(fields)? {
        "math-box" | "math-phantom" | "math-smash" => {}
        _ => return None,
    }
    Some(MathBox::new(
        optional_number_field(fields, "width")?,
        optional_number_field(fields, "height")?,
        optional_number_field(fields, "depth")?,
    ))
}

fn atom_list(v: &RuntimeValue, ids: &mut IdGen) -> Result<Vec<MathAtom>, MathValueError> {
    let mut out = Vec::new();
    for item in cons_items(v)? {
        if is_no_ink_math_tag(item) {
            continue;
        }
        out.push(math_atom_from_value_with_ids(item, ids)?);
    }
    Ok(out)
}

fn class_field(fields: &[(String, RuntimeValue)]) -> Result<MathClass, MathValueError> {
    let s = match field(fields, "class") {
        Some(RuntimeValue::String(s)) => s.as_str(),
        Some(_) => {
            return Err(MathValueError::new("math class must be a string"));
        }
        None => "ord",
    };
    match s {
        "ord" => Ok(MathClass::Ordinary),
        "op" => Ok(MathClass::Operator),
        "bin" => Ok(MathClass::Binary),
        "rel" => Ok(MathClass::Relation),
        "open" => Ok(MathClass::Open),
        "close" => Ok(MathClass::Close),
        "punct" => Ok(MathClass::Punctuation),
        "fence" => Ok(MathClass::Fence),
        other => Err(MathValueError::new(format!("unknown math class `{other}`"))),
    }
}

fn string_field(fields: &[(String, RuntimeValue)], name: &str) -> Result<String, MathValueError> {
    match field(fields, name) {
        Some(RuntimeValue::String(s)) => Ok(s.clone()),
        Some(_) => Err(MathValueError::new(format!("`{name}` must be a string"))),
        None => Err(MathValueError::new(format!("missing `{name}`"))),
    }
}

fn optional_number_field(fields: &[(String, RuntimeValue)], name: &str) -> Option<f64> {
    match field(fields, name)? {
        RuntimeValue::Number(n) => Some(*n),
        RuntimeValue::F64(n) => Some(*n),
        RuntimeValue::Int(n) => Some(*n as f64),
        _ => None,
    }
}

fn cons_items(v: &RuntimeValue) -> Result<Vec<&RuntimeValue>, MathValueError> {
    let mut out = Vec::new();
    let mut cur = v;
    loop {
        match cur {
            RuntimeValue::Variant { tag, .. } if tag == "nil" => break,
            RuntimeValue::Variant { tag, payload } if tag == "cons" => {
                let Some(payload) = payload.as_ref() else {
                    return Err(MathValueError::new("cons variant missing payload"));
                };
                let fields = record_fields(payload)?;
                let head = field(fields, "head")
                    .ok_or_else(|| MathValueError::new("cons missing head"))?;
                let tail = field(fields, "tail")
                    .ok_or_else(|| MathValueError::new("cons missing tail"))?;
                out.push(head);
                cur = tail;
            }
            _ => {
                return Err(MathValueError::new("math children must be a cons list"));
            }
        }
    }
    Ok(out)
}

fn record_fields(v: &RuntimeValue) -> Result<&[(String, RuntimeValue)], MathValueError> {
    match v {
        RuntimeValue::Record(fields) => Ok(fields.as_slice()),
        _ => Err(MathValueError::new("expected math record")),
    }
}

fn field<'a>(fields: &'a [(String, RuntimeValue)], name: &str) -> Option<&'a RuntimeValue> {
    fields.iter().find(|(k, _)| k == name).map(|(_, v)| v)
}

fn tag_of(fields: &[(String, RuntimeValue)]) -> Option<&str> {
    match field(fields, "tag") {
        Some(RuntimeValue::String(s)) => Some(s.as_str()),
        Some(RuntimeValue::ShapeTag(s)) => Some(s.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tip_tests {
    use super::*;

    fn rec(fields: Vec<(&str, RuntimeValue)>) -> RuntimeValue {
        RuntimeValue::Record(
            fields
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    fn cons(items: Vec<RuntimeValue>) -> RuntimeValue {
        let mut cur = RuntimeValue::Variant {
            tag: "nil".into(),
            payload: None,
        };
        for item in items.into_iter().rev() {
            cur = RuntimeValue::Variant {
                tag: "cons".into(),
                payload: Some(Box::new(RuntimeValue::Record(vec![
                    ("head".into(), item),
                    ("tail".into(), cur),
                ]))),
            };
        }
        cur
    }

    #[test]
    fn estimate_style_from_value_reads_style_field() {
        let base = rec(vec![
            ("tag", RuntimeValue::String("math-symbol".into())),
            ("glyph", RuntimeValue::String("x".into())),
            ("class", RuntimeValue::String("ord".into())),
        ]);
        let scripts = rec(vec![
            ("tag", RuntimeValue::String("math-scripts".into())),
            ("base", base.clone()),
            ("superscript", base),
            ("style", RuntimeValue::String("text".into())),
        ]);
        assert_eq!(estimate_style_from_value(&scripts), EstimateStyle::Text);
        // Kernel surface maps `"text"` string lit → ShapeTag.
        let via_tag = rec(vec![
            ("tag", RuntimeValue::String("math-symbol".into())),
            ("glyph", RuntimeValue::String("x".into())),
            ("style", RuntimeValue::ShapeTag("text".into())),
        ]);
        assert_eq!(estimate_style_from_value(&via_tag), EstimateStyle::Text);
        let display_w = estimate_math_box_from_value_with_style(&scripts, EstimateStyle::Display)
            .unwrap()
            .width;
        let text_w = estimate_math_box_from_value(&scripts).unwrap().width;
        assert!(text_w < display_w);
    }

    #[test]
    fn tip_error_arms_accent_matrix_class_and_shape() {
        // missing tag / non-record
        assert!(math_atom_from_value(&RuntimeValue::Int(1)).is_err());
        assert!(math_atom_from_value(&rec(vec![])).is_err());

        // unknown accent / matrix kinds
        let bad_accent = rec(vec![
            ("tag", RuntimeValue::String("math-accent".into())),
            ("kind", RuntimeValue::String("unknown".into())),
            (
                "base",
                rec(vec![
                    ("tag", RuntimeValue::String("math-symbol".into())),
                    ("glyph", RuntimeValue::String("x".into())),
                ]),
            ),
        ]);
        assert!(math_atom_from_value(&bad_accent)
            .unwrap_err()
            .message
            .contains("accent"));

        let bad_matrix = rec(vec![
            ("tag", RuntimeValue::String("math-matrix".into())),
            ("kind", RuntimeValue::String("weird".into())),
            (
                "rows",
                RuntimeValue::Variant {
                    tag: "nil".into(),
                    payload: None,
                },
            ),
        ]);
        assert!(math_atom_from_value(&bad_matrix)
            .unwrap_err()
            .message
            .contains("matrix"));

        // unknown class + operatorname/text
        let bad_class = rec(vec![
            ("tag", RuntimeValue::String("math-symbol".into())),
            ("glyph", RuntimeValue::String("x".into())),
            ("class", RuntimeValue::String("nope".into())),
        ]);
        assert!(math_atom_from_value(&bad_class).is_err());

        let opname = rec(vec![
            ("tag", RuntimeValue::String("math-operatorname".into())),
            ("name", RuntimeValue::String("sin".into())),
        ]);
        assert!(matches!(
            math_atom_from_value(&opname).unwrap(),
            MathAtom::Symbol { .. }
        ));
        let text = rec(vec![
            ("tag", RuntimeValue::String("math-text".into())),
            ("body", RuntimeValue::String("hi".into())),
        ]);
        assert!(matches!(
            math_atom_from_value(&text).unwrap(),
            MathAtom::Symbol { .. }
        ));

        // delimited matrix defaults
        let delimited = rec(vec![
            ("tag", RuntimeValue::String("math-matrix".into())),
            ("kind", RuntimeValue::String("delimited".into())),
            (
                "rows",
                RuntimeValue::Variant {
                    tag: "nil".into(),
                    payload: None,
                },
            ),
        ]);
        assert!(matches!(
            math_atom_from_value(&delimited).unwrap(),
            MathAtom::Matrix { .. }
        ));

        // non-cons children
        let bad_row = rec(vec![
            ("tag", RuntimeValue::String("math-row".into())),
            ("children", RuntimeValue::Int(0)),
        ]);
        assert!(math_atom_from_value(&bad_row)
            .unwrap_err()
            .message
            .contains("cons"));

        let _ = MathValueError::new("tip");
    }

    #[test]
    fn tip_delimiter_and_accent_estimate_via_bridge() {
        let sym = rec(vec![
            ("tag", RuntimeValue::String("math-symbol".into())),
            ("glyph", RuntimeValue::String("x".into())),
        ]);
        let frac = rec(vec![
            ("tag", RuntimeValue::String("math-fraction".into())),
            ("numerator", sym.clone()),
            ("denominator", sym.clone()),
        ]);
        let delim = rec(vec![
            ("tag", RuntimeValue::String("math-delimiter".into())),
            ("left", RuntimeValue::String("(".into())),
            ("right", RuntimeValue::String(")".into())),
            ("body", frac),
        ]);
        let atom = math_atom_from_value(&delim).expect("delimiter");
        assert!(
            matches!(atom, MathAtom::Delimiter { stretch_factor, .. } if (stretch_factor - 1.0).abs() < 1e-9)
        );
        let box_ = estimate_math_box_from_value(&delim).expect("box");
        assert!(box_.total_height() > 1.0);

        let stretched = rec(vec![
            ("tag", RuntimeValue::String("math-delimiter".into())),
            ("left", RuntimeValue::String("(".into())),
            ("right", RuntimeValue::String(")".into())),
            ("body", sym.clone()),
            ("stretch-factor", RuntimeValue::Number(2.0)),
        ]);
        let atom2 = math_atom_from_value(&stretched).expect("stretched delimiter");
        assert!(
            matches!(atom2, MathAtom::Delimiter { stretch_factor, .. } if (stretch_factor - 2.0).abs() < 1e-9)
        );
        let tall = estimate_math_box_from_value(&stretched).expect("tall");
        let flat = estimate_math_box_from_value(&rec(vec![
            ("tag", RuntimeValue::String("math-delimiter".into())),
            ("left", RuntimeValue::String("(".into())),
            ("right", RuntimeValue::String(")".into())),
            ("body", sym.clone()),
        ]))
        .unwrap();
        assert!(tall.total_height() > flat.total_height());

        let accent = rec(vec![
            ("tag", RuntimeValue::String("math-accent".into())),
            ("kind", RuntimeValue::String("hat".into())),
            ("base", sym),
        ]);
        let ab = estimate_math_box_from_value(&accent).expect("accent box");
        let bare = estimate_math_box_from_value(&rec(vec![
            ("tag", RuntimeValue::String("math-symbol".into())),
            ("glyph", RuntimeValue::String("x".into())),
        ]))
        .unwrap();
        assert!(ab.height > bare.height);
    }

    #[test]
    fn layout_math_to_shapes_places_linearize_glyphs() {
        let a = rec(vec![
            ("tag", RuntimeValue::String("math-symbol".into())),
            ("glyph", RuntimeValue::String("a".into())),
            ("class", RuntimeValue::String("ord".into())),
        ]);
        let b = rec(vec![
            ("tag", RuntimeValue::String("math-symbol".into())),
            ("glyph", RuntimeValue::String("b".into())),
            ("class", RuntimeValue::String("ord".into())),
        ]);
        // Symbol-only (still linearize path).
        let shapes =
            layout_math_to_shapes(&a, (10.0, 200.0), EstimateStyle::Display).expect("math shapes");
        match &shapes[0] {
            Shape::Text(t) => {
                assert_eq!(t.content, "a");
                assert!((t.x_mm - 10.0).abs() < 1e-9);
                assert!((t.y_mm - 200.0).abs() < 1e-9);
            }
            _ => panic!("expected Text shape"),
        }

        // Fraction: num/den Text + rule Line (LL9).
        let frac = rec(vec![
            ("tag", RuntimeValue::String("math-fraction".into())),
            ("numerator", a),
            ("denominator", b),
        ]);
        let frac_shapes = layout_math_to_shapes(&frac, (10.0, 200.0), EstimateStyle::Display)
            .expect("frac shapes");
        let texts: Vec<_> = frac_shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Text(t) => Some(t.content.as_str()),
                _ => None,
            })
            .collect();
        assert!(texts.contains(&"a") && texts.contains(&"b"), "{texts:?}");
        assert!(
            frac_shapes.iter().any(|s| matches!(s, Shape::Line(_))),
            "expected fraction rule Line among {frac_shapes:?}"
        );
        let a_y = frac_shapes
            .iter()
            .find_map(|s| match s {
                Shape::Text(t) if t.content == "a" => Some(t.y_mm),
                _ => None,
            })
            .unwrap();
        let b_y = frac_shapes
            .iter()
            .find_map(|s| match s {
                Shape::Text(t) if t.content == "b" => Some(t.y_mm),
                _ => None,
            })
            .unwrap();
        assert!(a_y < 200.0, "numerator above baseline: {a_y}");
        assert!(b_y > 200.0, "denominator below baseline: {b_y}");
    }

    #[test]
    fn layout_math_phantom_smash_produce_no_ink() {
        let phantom = rec(vec![
            ("tag", RuntimeValue::String("math-phantom".into())),
            ("width", RuntimeValue::Number(0.0)),
            ("height", RuntimeValue::Number(0.7)),
            ("depth", RuntimeValue::Number(0.2)),
        ]);
        let smash = rec(vec![
            ("tag", RuntimeValue::String("math-smash".into())),
            ("width", RuntimeValue::Number(1.0)),
            ("height", RuntimeValue::Number(0.0)),
            ("depth", RuntimeValue::Number(0.0)),
        ]);
        let p = layout_math_to_shapes(&phantom, (10.0, 200.0), EstimateStyle::Display)
            .expect("phantom");
        let s =
            layout_math_to_shapes(&smash, (10.0, 200.0), EstimateStyle::Display).expect("smash");
        assert!(p.is_empty(), "phantom must produce no ink: {p:?}");
        assert!(s.is_empty(), "smash must produce no/zero ink: {s:?}");

        let vis = rec(vec![
            ("tag", RuntimeValue::String("math-symbol".into())),
            ("glyph", RuntimeValue::String("x".into())),
            ("class", RuntimeValue::String("ord".into())),
        ]);
        let ink =
            layout_math_to_shapes(&vis, (10.0, 200.0), EstimateStyle::Display).expect("symbol");
        assert!(
            ink.iter()
                .any(|sh| matches!(sh, Shape::Text(t) if t.content == "x")),
            "visible symbol still inks: {ink:?}"
        );
    }

    #[test]
    fn layout_math_row_skips_nested_phantom_smash_tags() {
        let vis = rec(vec![
            ("tag", RuntimeValue::String("math-symbol".into())),
            ("glyph", RuntimeValue::String("x".into())),
            ("class", RuntimeValue::String("ord".into())),
        ]);
        let phantom = rec(vec![
            ("tag", RuntimeValue::String("math-phantom".into())),
            ("width", RuntimeValue::Number(0.0)),
            ("height", RuntimeValue::Number(0.7)),
            ("depth", RuntimeValue::Number(0.2)),
        ]);
        let smash = rec(vec![
            ("tag", RuntimeValue::String("math-smash".into())),
            ("width", RuntimeValue::Number(1.0)),
            ("height", RuntimeValue::Number(0.0)),
            ("depth", RuntimeValue::Number(0.0)),
        ]);
        let hidden = rec(vec![
            ("tag", RuntimeValue::String("math-symbol".into())),
            ("glyph", RuntimeValue::String("Σ".into())),
            ("class", RuntimeValue::String("op".into())),
        ]);
        let row = rec(vec![
            ("tag", RuntimeValue::String("math-row".into())),
            ("children", cons(vec![vis, phantom, smash, hidden])),
        ]);
        let shapes =
            layout_math_to_shapes(&row, (10.0, 200.0), EstimateStyle::Display).expect("row");
        let texts: Vec<_> = shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Text(t) => Some(t.content.as_str()),
                _ => None,
            })
            .collect();
        assert!(
            texts.contains(&"x") && texts.contains(&"Σ"),
            "visible glyphs remain: {texts:?}"
        );
    }

    #[test]
    fn estimate_math_box_reads_phantom_smash_metrics() {
        let phantom = rec(vec![
            ("tag", RuntimeValue::String("math-phantom".into())),
            ("width", RuntimeValue::Number(0.0)),
            ("height", RuntimeValue::Number(0.7)),
            ("depth", RuntimeValue::Number(0.2)),
        ]);
        let smash = rec(vec![
            ("tag", RuntimeValue::String("math-smash".into())),
            ("width", RuntimeValue::Number(1.25)),
            ("height", RuntimeValue::Number(0.0)),
            ("depth", RuntimeValue::Number(0.0)),
        ]);
        let pb = estimate_math_box_from_value(&phantom).expect("phantom metrics");
        assert!((pb.width - 0.0).abs() < 1e-9);
        assert!((pb.height - 0.7).abs() < 1e-9);
        assert!((pb.depth - 0.2).abs() < 1e-9);
        let sb = estimate_math_box_from_value(&smash).expect("smash metrics");
        assert!((sb.width - 1.25).abs() < 1e-9);
        assert!((sb.height - 0.0).abs() < 1e-9);
        assert!((sb.depth - 0.0).abs() < 1e-9);
    }

    #[test]
    fn layout_math_phantom_tag_as_shape_tag_still_no_ink() {
        let phantom = rec(vec![
            ("tag", RuntimeValue::ShapeTag("math-phantom".into())),
            ("width", RuntimeValue::Number(0.0)),
            ("height", RuntimeValue::Number(0.7)),
            ("depth", RuntimeValue::Number(0.2)),
        ]);
        let p = layout_math_to_shapes(&phantom, (10.0, 200.0), EstimateStyle::Display)
            .expect("shape-tag phantom");
        assert!(
            p.is_empty(),
            "ShapeTag math-phantom must produce no ink: {p:?}"
        );
    }

    #[test]
    fn layout_math_to_shapes_honors_layout_style() {
        // Row hits the linearize path; a superscripted scripts child's
        // estimate_box height uses script_scale, so Text vs Display changes
        // glyph height_mm.
        let x = rec(vec![
            ("tag", RuntimeValue::String("math-symbol".into())),
            ("glyph", RuntimeValue::String("x".into())),
            ("class", RuntimeValue::String("ord".into())),
        ]);
        let n = rec(vec![
            ("tag", RuntimeValue::String("math-symbol".into())),
            ("glyph", RuntimeValue::String("n".into())),
            ("class", RuntimeValue::String("ord".into())),
        ]);
        let scripts = rec(vec![
            ("tag", RuntimeValue::String("math-scripts".into())),
            ("base", x),
            ("superscript", n),
        ]);
        let row = rec(vec![
            ("tag", RuntimeValue::String("math-row".into())),
            ("children", cons(vec![scripts])),
        ]);
        let d =
            layout_math_to_shapes(&row, (10.0, 200.0), EstimateStyle::Display).expect("display");
        let t = layout_math_to_shapes(&row, (10.0, 200.0), EstimateStyle::Text).expect("text");
        let h = |shapes: &[Shape]| {
            shapes.iter().find_map(|s| match s {
                Shape::Text(tx) => tx.height_mm,
                _ => None,
            })
        };
        let dh = h(&d).expect("display height");
        let th = h(&t).expect("text height");
        assert!(
            (dh - th).abs() > 1e-9,
            "layout_style Text vs Display should change row glyph height_mm, {dh} vs {th}"
        );
    }
}
