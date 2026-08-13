//! Light bridge: package `math/*` tagged records → [`MathAtom`] / [`MathBox`].
//!
//! Constructors in `reciplexa-package::domain_bodies` emit
//! `tag: "math-symbol"|"math-row"|"math-fraction"|…` records. This module
//! lowers them for fontless `estimate_box` (document / layout prep).

use std::cell::Cell;

use reciplexa_identity::document::StableNodeId;
use reciplexa_std::math::{
    MathAccentKind, MathAtom, MathBox, MathClass, MathMatrixKind, MathStackKind,
};

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
pub fn estimate_math_box_from_value(v: &RuntimeValue) -> Result<MathBox, MathValueError> {
    Ok(math_atom_from_value(v)?.estimate_box())
}

/// Lower a package math tagged record to a [`MathAtom`].
pub fn math_atom_from_value(v: &RuntimeValue) -> Result<MathAtom, MathValueError> {
    let mut ids = IdGen::default();
    math_atom_from_value_with_ids(v, &mut ids)
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
            Ok(MathAtom::delimiter(ids.mint(), left, right, body))
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
            Ok(MathAtom::stack(ids.mint(), MathStackKind::Substack, children))
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
        (true, "overline") => Ok(MathAtom::accent(
            ids.mint(),
            MathAccentKind::Overline,
            body,
        )),
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
            None => Ok(MathAtom::accent(
                ids.mint(),
                MathAccentKind::Overline,
                body,
            )),
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
            None if over => Ok(MathAtom::accent(
                ids.mint(),
                MathAccentKind::Overline,
                body,
            )),
            None => Ok(MathAtom::accent(
                ids.mint(),
                MathAccentKind::Underline,
                body,
            )),
        },
    }
}

fn case_arm_cells(
    v: &RuntimeValue,
    ids: &mut IdGen,
) -> Result<Vec<MathAtom>, MathValueError> {
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
    match s.as_str() {
        "hat" => Ok(MathAccentKind::Hat),
        "bar" => Ok(MathAccentKind::Bar),
        "vec" => Ok(MathAccentKind::Vec),
        "tilde" => Ok(MathAccentKind::Tilde),
        "dot" => Ok(MathAccentKind::Dot),
        "ddot" => Ok(MathAccentKind::Ddot),
        "overline" => Ok(MathAccentKind::Overline),
        "underline" => Ok(MathAccentKind::Underline),
        "widehat" => Ok(MathAccentKind::WideHat),
        "widetilde" => Ok(MathAccentKind::WideTilde),
        other => Err(MathValueError::new(format!(
            "unknown accent kind `{other}`"
        ))),
    }
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
    if is_math_absent(v) {
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

fn atom_list(v: &RuntimeValue, ids: &mut IdGen) -> Result<Vec<MathAtom>, MathValueError> {
    let mut out = Vec::new();
    for item in cons_items(v)? {
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
        assert!(matches!(atom, MathAtom::Delimiter { stretch_factor, .. } if (stretch_factor - 1.0).abs() < 1e-9));
        let box_ = estimate_math_box_from_value(&delim).expect("box");
        assert!(box_.total_height() > 1.0);

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
}
