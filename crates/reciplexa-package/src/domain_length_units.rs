//! Direct Native v2 typed exports for `length/units` (DN2-1).

use std::collections::BTreeMap;

use reciplexa_core::ty::{CoreType, EffectRow};
use reciplexa_eval::DomainNativeOp;

use crate::domain_native::{DomainNativeExport, DomainNativeModule};

fn length_record_ty() -> CoreType {
    CoreType::Record {
        fields: vec![
            ("unit".into(), CoreType::String),
            ("value".into(), CoreType::Number),
        ],
    }
}

fn unary_number_to_record() -> CoreType {
    CoreType::Fun {
        args: vec![CoreType::Number],
        ret: Box::new(length_record_ty()),
        effects: EffectRow::default(),
    }
}

fn unary_record_to_number() -> CoreType {
    CoreType::Fun {
        args: vec![length_record_ty()],
        ret: Box::new(CoreType::Number),
        effects: EffectRow::default(),
    }
}

fn binary_record_record_to_record() -> CoreType {
    CoreType::Fun {
        args: vec![length_record_ty(), length_record_ty()],
        ret: Box::new(length_record_ty()),
        effects: EffectRow::default(),
    }
}

fn binary_record_number_to_record() -> CoreType {
    CoreType::Fun {
        args: vec![length_record_ty(), CoreType::Number],
        ret: Box::new(length_record_ty()),
        effects: EffectRow::default(),
    }
}

fn nullary_record() -> CoreType {
    CoreType::Fun {
        args: vec![],
        ret: Box::new(length_record_ty()),
        effects: EffectRow::default(),
    }
}

/// Attach typed DN2 exports for `length/units` (Hybrid body unchanged).
pub fn populate_length_units_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let insert = |exports: &mut BTreeMap<String, DomainNativeExport>,
                  name: &str,
                  ty: CoreType,
                  op: DomainNativeOp| {
        exports.insert(name.into(), DomainNativeExport::new(name, ty, op));
    };
    insert(
        &mut exports,
        "mm",
        unary_number_to_record(),
        DomainNativeOp::LengthUnitsMm,
    );
    insert(
        &mut exports,
        "cm",
        unary_number_to_record(),
        DomainNativeOp::LengthUnitsCm,
    );
    insert(
        &mut exports,
        "pt",
        unary_number_to_record(),
        DomainNativeOp::LengthUnitsPt,
    );
    insert(
        &mut exports,
        "bp",
        unary_number_to_record(),
        DomainNativeOp::LengthUnitsBp,
    );
    insert(
        &mut exports,
        "inch",
        unary_number_to_record(),
        DomainNativeOp::LengthUnitsInch,
    );
    insert(
        &mut exports,
        "q",
        unary_number_to_record(),
        DomainNativeOp::LengthUnitsQ,
    );
    insert(
        &mut exports,
        "px",
        unary_number_to_record(),
        DomainNativeOp::LengthUnitsPx,
    );
    insert(
        &mut exports,
        "em",
        unary_number_to_record(),
        DomainNativeOp::LengthUnitsEm,
    );
    insert(
        &mut exports,
        "zero",
        nullary_record(),
        DomainNativeOp::LengthUnitsZero,
    );
    insert(
        &mut exports,
        "to-mm",
        unary_record_to_number(),
        DomainNativeOp::LengthUnitsToMm,
    );
    insert(
        &mut exports,
        "from-mm",
        unary_number_to_record(),
        DomainNativeOp::LengthUnitsFromMm,
    );
    insert(
        &mut exports,
        "add-mm",
        binary_record_record_to_record(),
        DomainNativeOp::LengthUnitsAddMm,
    );
    insert(
        &mut exports,
        "scale-length",
        binary_record_number_to_record(),
        DomainNativeOp::LengthUnitsScaleLength,
    );
    module.typed_exports = exports;
}
