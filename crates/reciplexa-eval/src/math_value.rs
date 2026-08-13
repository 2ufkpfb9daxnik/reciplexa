//! Light bridge: package `math/*` tagged records → [`MathAtom`] / [`MathBox`].
//!
//! Constructors in `reciplexa-package::domain_bodies` emit
//! `tag: "math-symbol"|"math-row"|"math-fraction"|…` records. This module
//! lowers them for fontless `estimate_box` (document / layout prep).

use std::cell::Cell;

use reciplexa_identity::document::StableNodeId;
use reciplexa_std::math::{MathAtom, MathBox, MathClass};

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
        other => Err(MathValueError::new(format!(
            "unsupported math tag `{other}`"
        ))),
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
                return Err(MathValueError::new(
                    "math children must be a cons list",
                ));
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
