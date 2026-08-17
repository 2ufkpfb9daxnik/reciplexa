//! Direct Native v2 typed exports for math constructors (DN2-3).

use std::collections::BTreeMap;

use reciplexa_core::ty::{CoreType, EffectRow};
use reciplexa_eval::domain_native::{
    DomainNativeOp, MathAccentsOp, MathAlignOp, MathAtomsOp, MathBigopsOp, MathCasesOp,
    MathDelimitersOp, MathFracOp, MathMatrixOp, MathScriptsOp, MathSqrtOp, MathStackOp,
};

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

/// Attach typed DN2 exports for `math/atoms`.
pub fn populate_math_atoms_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::MathAtoms(sub);
    insert(
        &mut exports,
        "class-ord",
        nullary_dynamic(),
        op(MathAtomsOp::ClassOrd),
    );
    insert(
        &mut exports,
        "class-op",
        nullary_dynamic(),
        op(MathAtomsOp::ClassOp),
    );
    insert(
        &mut exports,
        "class-bin",
        nullary_dynamic(),
        op(MathAtomsOp::ClassBin),
    );
    insert(
        &mut exports,
        "class-rel",
        nullary_dynamic(),
        op(MathAtomsOp::ClassRel),
    );
    insert(
        &mut exports,
        "class-open",
        nullary_dynamic(),
        op(MathAtomsOp::ClassOpen),
    );
    insert(
        &mut exports,
        "class-close",
        nullary_dynamic(),
        op(MathAtomsOp::ClassClose),
    );
    insert(
        &mut exports,
        "class-punct",
        nullary_dynamic(),
        op(MathAtomsOp::ClassPunct),
    );
    insert(
        &mut exports,
        "class-fence",
        nullary_dynamic(),
        op(MathAtomsOp::ClassFence),
    );
    insert(&mut exports, "symbol", fun_n(2), op(MathAtomsOp::Symbol));
    insert(&mut exports, "ord", fun_n(1), op(MathAtomsOp::Ord));
    insert(&mut exports, "op", fun_n(1), op(MathAtomsOp::Op));
    insert(&mut exports, "bin", fun_n(1), op(MathAtomsOp::Bin));
    insert(&mut exports, "rel", fun_n(1), op(MathAtomsOp::Rel));
    insert(&mut exports, "open", fun_n(1), op(MathAtomsOp::Open));
    insert(&mut exports, "close", fun_n(1), op(MathAtomsOp::Close));
    insert(&mut exports, "punct", fun_n(1), op(MathAtomsOp::Punct));
    insert(&mut exports, "fence", fun_n(1), op(MathAtomsOp::Fence));
    insert(&mut exports, "row", fun_n(1), op(MathAtomsOp::Row));
    insert(
        &mut exports,
        "operatorname",
        fun_n(1),
        op(MathAtomsOp::Operatorname),
    );
    insert(&mut exports, "mathrm", fun_n(1), op(MathAtomsOp::Mathrm));
    insert(&mut exports, "mathbf", fun_n(1), op(MathAtomsOp::Mathbf));
    insert(&mut exports, "textop", fun_n(1), op(MathAtomsOp::Textop));
    insert(
        &mut exports,
        "fraction",
        fun_n(2),
        op(MathAtomsOp::Fraction),
    );
    insert(
        &mut exports,
        "superscript",
        fun_n(2),
        op(MathAtomsOp::Superscript),
    );
    insert(
        &mut exports,
        "subscript",
        fun_n(2),
        op(MathAtomsOp::Subscript),
    );
    module.typed_exports = exports;
}

/// Attach typed DN2 exports for `math/scripts`.
pub fn populate_math_scripts_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::MathScripts(sub);
    insert(
        &mut exports,
        "absent",
        nullary_dynamic(),
        op(MathScriptsOp::Absent),
    );
    insert(
        &mut exports,
        "superscript",
        fun_n(2),
        op(MathScriptsOp::Superscript),
    );
    insert(
        &mut exports,
        "subscript",
        fun_n(2),
        op(MathScriptsOp::Subscript),
    );
    insert(
        &mut exports,
        "scripts",
        fun_n(3),
        op(MathScriptsOp::Scripts),
    );
    insert(&mut exports, "super", fun_n(2), op(MathScriptsOp::Super));
    insert(&mut exports, "sub", fun_n(2), op(MathScriptsOp::Sub));
    insert(
        &mut exports,
        "overline",
        fun_n(1),
        op(MathScriptsOp::Overline),
    );
    insert(
        &mut exports,
        "underline",
        fun_n(1),
        op(MathScriptsOp::Underline),
    );
    insert(
        &mut exports,
        "overbrace",
        fun_n(1),
        op(MathScriptsOp::Overbrace),
    );
    insert(
        &mut exports,
        "underbrace",
        fun_n(1),
        op(MathScriptsOp::Underbrace),
    );
    insert(
        &mut exports,
        "overset",
        fun_n(2),
        op(MathScriptsOp::Overset),
    );
    insert(
        &mut exports,
        "underset",
        fun_n(2),
        op(MathScriptsOp::Underset),
    );
    insert(&mut exports, "over", fun_n(2), op(MathScriptsOp::Over));
    insert(&mut exports, "under", fun_n(2), op(MathScriptsOp::Under));
    module.typed_exports = exports;
}

/// Attach typed DN2 exports for `math/frac`.
pub fn populate_math_frac_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::MathFrac(sub);
    insert(&mut exports, "fraction", fun_n(2), op(MathFracOp::Fraction));
    insert(&mut exports, "over", fun_n(2), op(MathFracOp::Over));
    module.typed_exports = exports;
}

/// Attach typed DN2 exports for `math/sqrt`.
pub fn populate_math_sqrt_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::MathSqrt(sub);
    insert(
        &mut exports,
        "absent-index",
        nullary_dynamic(),
        op(MathSqrtOp::AbsentIndex),
    );
    insert(&mut exports, "sqrt", fun_n(1), op(MathSqrtOp::Sqrt));
    insert(&mut exports, "radical", fun_n(1), op(MathSqrtOp::Radical));
    insert(
        &mut exports,
        "radical-indexed",
        fun_n(2),
        op(MathSqrtOp::RadicalIndexed),
    );
    insert(&mut exports, "root", fun_n(2), op(MathSqrtOp::Root));
    module.typed_exports = exports;
}

/// Attach typed DN2 exports for `math/delimiters`.
pub fn populate_math_delimiters_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::MathDelimiters(sub);
    insert(
        &mut exports,
        "delimiter",
        fun_n(3),
        op(MathDelimitersOp::Delimiter),
    );
    insert(&mut exports, "paren", fun_n(1), op(MathDelimitersOp::Paren));
    insert(
        &mut exports,
        "brackets",
        fun_n(1),
        op(MathDelimitersOp::Brackets),
    );
    insert(
        &mut exports,
        "braces",
        fun_n(1),
        op(MathDelimitersOp::Braces),
    );
    insert(
        &mut exports,
        "angles",
        fun_n(1),
        op(MathDelimitersOp::Angles),
    );
    insert(&mut exports, "abs", fun_n(1), op(MathDelimitersOp::Abs));
    insert(&mut exports, "floor", fun_n(1), op(MathDelimitersOp::Floor));
    insert(&mut exports, "ceil", fun_n(1), op(MathDelimitersOp::Ceil));
    module.typed_exports = exports;
}

/// Attach typed DN2 exports for `math/matrix`.
pub fn populate_math_matrix_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::MathMatrix(sub);
    insert(&mut exports, "matrix", fun_n(1), op(MathMatrixOp::Matrix));
    insert(&mut exports, "bmatrix", fun_n(1), op(MathMatrixOp::Bmatrix));
    insert(&mut exports, "pmatrix", fun_n(1), op(MathMatrixOp::Pmatrix));
    insert(&mut exports, "vmatrix", fun_n(1), op(MathMatrixOp::Vmatrix));
    insert(
        &mut exports,
        "matrix-row",
        fun_n(1),
        op(MathMatrixOp::MatrixRow),
    );
    insert(
        &mut exports,
        "smallmatrix",
        fun_n(1),
        op(MathMatrixOp::Smallmatrix),
    );
    insert(
        &mut exports,
        "matrix-env",
        fun_n(1),
        op(MathMatrixOp::MatrixEnv),
    );
    insert(
        &mut exports,
        "array-env",
        fun_n(2),
        op(MathMatrixOp::ArrayEnv),
    );
    insert(
        &mut exports,
        "matrix-delim",
        fun_n(3),
        op(MathMatrixOp::MatrixDelim),
    );
    insert(
        &mut exports,
        "bmatrix-env",
        fun_n(1),
        op(MathMatrixOp::BmatrixEnv),
    );
    insert(
        &mut exports,
        "pmatrix-env",
        fun_n(1),
        op(MathMatrixOp::PmatrixEnv),
    );
    module.typed_exports = exports;
}

/// Attach typed DN2 exports for `math/accents`.
pub fn populate_math_accents_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::MathAccents(sub);
    insert(&mut exports, "accent", fun_n(2), op(MathAccentsOp::Accent));
    insert(&mut exports, "hat", fun_n(1), op(MathAccentsOp::Hat));
    insert(&mut exports, "bar", fun_n(1), op(MathAccentsOp::Bar));
    insert(&mut exports, "vec", fun_n(1), op(MathAccentsOp::Vec));
    insert(&mut exports, "tilde", fun_n(1), op(MathAccentsOp::Tilde));
    insert(&mut exports, "dot", fun_n(1), op(MathAccentsOp::Dot));
    insert(&mut exports, "ddot", fun_n(1), op(MathAccentsOp::Ddot));
    insert(&mut exports, "check", fun_n(1), op(MathAccentsOp::Check));
    insert(&mut exports, "breve", fun_n(1), op(MathAccentsOp::Breve));
    insert(&mut exports, "acute", fun_n(1), op(MathAccentsOp::Acute));
    insert(&mut exports, "grave", fun_n(1), op(MathAccentsOp::Grave));
    insert(&mut exports, "ring", fun_n(1), op(MathAccentsOp::Ring));
    insert(
        &mut exports,
        "overline",
        fun_n(1),
        op(MathAccentsOp::Overline),
    );
    insert(
        &mut exports,
        "underline",
        fun_n(1),
        op(MathAccentsOp::Underline),
    );
    insert(
        &mut exports,
        "underbar",
        fun_n(1),
        op(MathAccentsOp::Underbar),
    );
    insert(
        &mut exports,
        "widehat",
        fun_n(1),
        op(MathAccentsOp::Widehat),
    );
    insert(
        &mut exports,
        "widetilde",
        fun_n(1),
        op(MathAccentsOp::Widetilde),
    );
    module.typed_exports = exports;
}

/// Attach typed DN2 exports for `math/bigops`.
pub fn populate_math_bigops_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::MathBigops(sub);
    insert(&mut exports, "bigop", fun_n(3), op(MathBigopsOp::Bigop));
    insert(&mut exports, "sum", fun_n(3), op(MathBigopsOp::Sum));
    insert(&mut exports, "prod", fun_n(3), op(MathBigopsOp::Prod));
    insert(
        &mut exports,
        "integral",
        fun_n(3),
        op(MathBigopsOp::Integral),
    );
    insert(&mut exports, "oint", fun_n(3), op(MathBigopsOp::Oint));
    insert(&mut exports, "lim", fun_n(2), op(MathBigopsOp::Lim));
    insert(&mut exports, "limsup", fun_n(2), op(MathBigopsOp::Limsup));
    insert(&mut exports, "liminf", fun_n(2), op(MathBigopsOp::Liminf));
    insert(&mut exports, "max", fun_n(2), op(MathBigopsOp::Max));
    insert(&mut exports, "min", fun_n(2), op(MathBigopsOp::Min));
    insert(
        &mut exports,
        "withlimits",
        fun_n(4),
        op(MathBigopsOp::Withlimits),
    );
    insert(
        &mut exports,
        "nolimits",
        fun_n(2),
        op(MathBigopsOp::Nolimits),
    );
    insert(
        &mut exports,
        "sum-nolimits",
        fun_n(1),
        op(MathBigopsOp::SumNolimits),
    );
    insert(
        &mut exports,
        "integral-nolimits",
        fun_n(1),
        op(MathBigopsOp::IntegralNolimits),
    );
    insert(
        &mut exports,
        "bigop-scripts",
        fun_n(3),
        op(MathBigopsOp::BigopScripts),
    );
    insert(
        &mut exports,
        "sum-scripts",
        fun_n(3),
        op(MathBigopsOp::SumScripts),
    );
    insert(
        &mut exports,
        "prod-scripts",
        fun_n(3),
        op(MathBigopsOp::ProdScripts),
    );
    insert(
        &mut exports,
        "integral-scripts",
        fun_n(3),
        op(MathBigopsOp::IntegralScripts),
    );
    insert(
        &mut exports,
        "oint-scripts",
        fun_n(3),
        op(MathBigopsOp::OintScripts),
    );
    module.typed_exports = exports;
}

/// Attach typed DN2 exports for `math/cases`.
pub fn populate_math_cases_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::MathCases(sub);
    insert(&mut exports, "case-arm", fun_n(2), op(MathCasesOp::CaseArm));
    insert(&mut exports, "cases", fun_n(1), op(MathCasesOp::Cases));
    insert(&mut exports, "cases-lr", fun_n(3), op(MathCasesOp::CasesLr));
    insert(
        &mut exports,
        "cases-delim",
        fun_n(3),
        op(MathCasesOp::CasesDelim),
    );
    insert(
        &mut exports,
        "left-cases",
        fun_n(1),
        op(MathCasesOp::LeftCases),
    );
    insert(
        &mut exports,
        "right-cases",
        fun_n(1),
        op(MathCasesOp::RightCases),
    );
    insert(
        &mut exports,
        "piecewise",
        fun_n(1),
        op(MathCasesOp::Piecewise),
    );
    insert(
        &mut exports,
        "otherwise",
        fun_n(1),
        op(MathCasesOp::Otherwise),
    );
    module.typed_exports = exports;
}

/// Attach typed DN2 exports for `math/align`.
pub fn populate_math_align_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::MathAlign(sub);
    insert(
        &mut exports,
        "align-row",
        fun_n(1),
        op(MathAlignOp::AlignRow),
    );
    insert(&mut exports, "aligned", fun_n(1), op(MathAlignOp::Aligned));
    insert(&mut exports, "align", fun_n(2), op(MathAlignOp::Align));
    insert(
        &mut exports,
        "align-left",
        fun_n(1),
        op(MathAlignOp::AlignLeft),
    );
    insert(
        &mut exports,
        "align-center",
        fun_n(1),
        op(MathAlignOp::AlignCenter),
    );
    insert(
        &mut exports,
        "align-right",
        fun_n(1),
        op(MathAlignOp::AlignRight),
    );
    insert(&mut exports, "align-at", fun_n(2), op(MathAlignOp::AlignAt));
    insert(&mut exports, "align-eq", fun_n(1), op(MathAlignOp::AlignEq));
    module.typed_exports = exports;
}

/// Attach typed DN2 exports for `math/stack`.
pub fn populate_math_stack_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::MathStack(sub);
    insert(&mut exports, "stack", fun_n(1), op(MathStackOp::Stack));
    insert(
        &mut exports,
        "stackrel",
        fun_n(2),
        op(MathStackOp::Stackrel),
    );
    insert(
        &mut exports,
        "overset-rel",
        fun_n(2),
        op(MathStackOp::OversetRel),
    );
    insert(
        &mut exports,
        "underset-rel",
        fun_n(2),
        op(MathStackOp::UndersetRel),
    );
    insert(&mut exports, "atop", fun_n(2), op(MathStackOp::Atop));
    insert(
        &mut exports,
        "substack",
        fun_n(1),
        op(MathStackOp::Substack),
    );
    module.typed_exports = exports;
}
