//! Direct Native v2 runtime for math record constructors (DN2-3).

use crate::domain_native::{
    DomainNativeOp, MathAccentsOp, MathAlignOp, MathAtomsOp, MathBigopsOp, MathCasesOp,
    MathDelimitersOp, MathFracOp, MathMatrixOp, MathScriptsOp, MathSqrtOp, MathStackOp,
};
use crate::domain_native_failure::{take0, take1, take2, take3, take4};
use crate::value::RuntimeValue;
use crate::EvalError;

pub fn call_math_constructor(
    op: DomainNativeOp,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, EvalError> {
    match op {
        DomainNativeOp::MathAtoms(sub) => call_math_atoms(sub, args),
        DomainNativeOp::MathScripts(sub) => call_math_scripts(sub, args),
        DomainNativeOp::MathFrac(sub) => call_math_frac(sub, args),
        DomainNativeOp::MathSqrt(sub) => call_math_sqrt(sub, args),
        DomainNativeOp::MathDelimiters(sub) => call_math_delimiters(sub, args),
        DomainNativeOp::MathMatrix(sub) => call_math_matrix(sub, args),
        DomainNativeOp::MathAccents(sub) => call_math_accents(sub, args),
        DomainNativeOp::MathBigops(sub) => call_math_bigops(sub, args),
        DomainNativeOp::MathCases(sub) => call_math_cases(sub, args),
        DomainNativeOp::MathAlign(sub) => call_math_align(sub, args),
        DomainNativeOp::MathStack(sub) => call_math_stack(sub, args),
        other => Err(EvalError {
            message: format!("not a math constructor op: {other:?}"),
        }),
    }
}

fn call_math_atoms(op: MathAtomsOp, args: &[RuntimeValue]) -> Result<RuntimeValue, EvalError> {
    match op {
        MathAtomsOp::ClassOrd => constant_str("ord", args, "`math/atoms class-ord`"),
        MathAtomsOp::ClassOp => constant_str("op", args, "`math/atoms class-op`"),
        MathAtomsOp::ClassBin => constant_str("bin", args, "`math/atoms class-bin`"),
        MathAtomsOp::ClassRel => constant_str("rel", args, "`math/atoms class-rel`"),
        MathAtomsOp::ClassOpen => constant_str("open", args, "`math/atoms class-open`"),
        MathAtomsOp::ClassClose => constant_str("close", args, "`math/atoms class-close`"),
        MathAtomsOp::ClassPunct => constant_str("punct", args, "`math/atoms class-punct`"),
        MathAtomsOp::ClassFence => constant_str("fence", args, "`math/atoms class-fence`"),
        MathAtomsOp::Symbol => {
            let [glyph, class] = take2(args, "`math/atoms symbol`")?;
            Ok(math_symbol(glyph, class))
        }
        MathAtomsOp::Ord => {
            let [glyph] = take1(args, "`math/atoms ord`")?;
            Ok(math_symbol_class(glyph, "ord"))
        }
        MathAtomsOp::Op => {
            let [glyph] = take1(args, "`math/atoms op`")?;
            Ok(math_symbol_class(glyph, "op"))
        }
        MathAtomsOp::Bin => {
            let [glyph] = take1(args, "`math/atoms bin`")?;
            Ok(math_symbol_class(glyph, "bin"))
        }
        MathAtomsOp::Rel => {
            let [glyph] = take1(args, "`math/atoms rel`")?;
            Ok(math_symbol_class(glyph, "rel"))
        }
        MathAtomsOp::Open => {
            let [glyph] = take1(args, "`math/atoms open`")?;
            Ok(math_symbol_class(glyph, "open"))
        }
        MathAtomsOp::Close => {
            let [glyph] = take1(args, "`math/atoms close`")?;
            Ok(math_symbol_class(glyph, "close"))
        }
        MathAtomsOp::Punct => {
            let [glyph] = take1(args, "`math/atoms punct`")?;
            Ok(math_symbol_class(glyph, "punct"))
        }
        MathAtomsOp::Fence => {
            let [glyph] = take1(args, "`math/atoms fence`")?;
            Ok(math_symbol_class(glyph, "fence"))
        }
        MathAtomsOp::Row => {
            let [children] = take1(args, "`math/atoms row`")?;
            Ok(tagged("math-row", vec![("children", children.clone())]))
        }
        MathAtomsOp::Operatorname => {
            let [name] = take1(args, "`math/atoms operatorname`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-operatorname")),
                ("name".into(), name.clone()),
                ("class".into(), str_val("op")),
                ("variant".into(), str_val("operatorname")),
            ]))
        }
        MathAtomsOp::Mathrm => {
            let [body] = take1(args, "`math/atoms mathrm`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-text")),
                ("body".into(), body.clone()),
                ("variant".into(), str_val("roman")),
                ("class".into(), str_val("ord")),
            ]))
        }
        MathAtomsOp::Mathbf => {
            let [body] = take1(args, "`math/atoms mathbf`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-text")),
                ("body".into(), body.clone()),
                ("variant".into(), str_val("bold")),
                ("class".into(), str_val("ord")),
            ]))
        }
        MathAtomsOp::Textop => {
            let [glyph] = take1(args, "`math/atoms textop`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-textop")),
                ("glyph".into(), glyph.clone()),
                ("class".into(), str_val("op")),
            ]))
        }
        MathAtomsOp::Fraction => {
            let [numerator, denominator] = take2(args, "`math/atoms fraction`")?;
            Ok(math_fraction(numerator, denominator))
        }
        MathAtomsOp::Superscript => {
            let [base, exp] = take2(args, "`math/atoms superscript`")?;
            Ok(math_scripts(base, exp, &math_absent()))
        }
        MathAtomsOp::Subscript => {
            let [base, exp] = take2(args, "`math/atoms subscript`")?;
            Ok(math_scripts(base, &math_absent(), exp))
        }
    }
}

fn call_math_scripts(op: MathScriptsOp, args: &[RuntimeValue]) -> Result<RuntimeValue, EvalError> {
    match op {
        MathScriptsOp::Absent => {
            take0(args, "`math/scripts absent`")?;
            Ok(math_absent())
        }
        MathScriptsOp::Superscript => {
            let [base, exp] = take2(args, "`math/scripts superscript`")?;
            Ok(math_scripts(base, exp, &math_absent()))
        }
        MathScriptsOp::Subscript => {
            let [base, exp] = take2(args, "`math/scripts subscript`")?;
            Ok(math_scripts(base, &math_absent(), exp))
        }
        MathScriptsOp::Scripts => {
            let [base, sup, sub] = take3(args, "`math/scripts scripts`")?;
            Ok(math_scripts(base, sup, sub))
        }
        MathScriptsOp::Super => {
            let [base, exp] = take2(args, "`math/scripts super`")?;
            Ok(math_scripts(base, exp, &math_absent()))
        }
        MathScriptsOp::Sub => {
            let [base, exp] = take2(args, "`math/scripts sub`")?;
            Ok(math_scripts(base, &math_absent(), exp))
        }
        MathScriptsOp::Overline => {
            let [body] = take1(args, "`math/scripts overline`")?;
            Ok(tagged(
                "math-over",
                vec![("kind", str_val("overline")), ("body", body.clone())],
            ))
        }
        MathScriptsOp::Underline => {
            let [body] = take1(args, "`math/scripts underline`")?;
            Ok(tagged(
                "math-under",
                vec![("kind", str_val("underline")), ("body", body.clone())],
            ))
        }
        MathScriptsOp::Overbrace => {
            let [body] = take1(args, "`math/scripts overbrace`")?;
            Ok(tagged(
                "math-over",
                vec![("kind", str_val("overbrace")), ("body", body.clone())],
            ))
        }
        MathScriptsOp::Underbrace => {
            let [body] = take1(args, "`math/scripts underbrace`")?;
            Ok(tagged(
                "math-under",
                vec![("kind", str_val("underbrace")), ("body", body.clone())],
            ))
        }
        MathScriptsOp::Overset => {
            let [over, body] = take2(args, "`math/scripts overset`")?;
            Ok(tagged(
                "math-over",
                vec![
                    ("kind", str_val("overset")),
                    ("label", over.clone()),
                    ("body", body.clone()),
                ],
            ))
        }
        MathScriptsOp::Underset => {
            let [under, body] = take2(args, "`math/scripts underset`")?;
            Ok(tagged(
                "math-under",
                vec![
                    ("kind", str_val("underset")),
                    ("label", under.clone()),
                    ("body", body.clone()),
                ],
            ))
        }
        MathScriptsOp::Over => {
            let [over, body] = take2(args, "`math/scripts over`")?;
            Ok(tagged(
                "math-over",
                vec![
                    ("kind", str_val("over")),
                    ("label", over.clone()),
                    ("body", body.clone()),
                ],
            ))
        }
        MathScriptsOp::Under => {
            let [under, body] = take2(args, "`math/scripts under`")?;
            Ok(tagged(
                "math-under",
                vec![
                    ("kind", str_val("under")),
                    ("label", under.clone()),
                    ("body", body.clone()),
                ],
            ))
        }
    }
}

fn call_math_frac(op: MathFracOp, args: &[RuntimeValue]) -> Result<RuntimeValue, EvalError> {
    match op {
        MathFracOp::Fraction | MathFracOp::Over => {
            let [numerator, denominator] = take2(args, "`math/frac fraction`")?;
            Ok(math_fraction(numerator, denominator))
        }
    }
}

fn call_math_sqrt(op: MathSqrtOp, args: &[RuntimeValue]) -> Result<RuntimeValue, EvalError> {
    match op {
        MathSqrtOp::AbsentIndex => {
            take0(args, "`math/sqrt absent-index`")?;
            Ok(math_absent())
        }
        MathSqrtOp::Sqrt | MathSqrtOp::Radical => {
            let [radicand] = take1(args, "`math/sqrt sqrt`")?;
            Ok(math_radical(&math_absent(), radicand))
        }
        MathSqrtOp::RadicalIndexed | MathSqrtOp::Root => {
            let [index, radicand] = take2(args, "`math/sqrt root`")?;
            Ok(math_radical(index, radicand))
        }
    }
}

fn call_math_delimiters(
    op: MathDelimitersOp,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, EvalError> {
    match op {
        MathDelimitersOp::Delimiter => {
            let [left, right, body] = take3(args, "`math/delimiters delimiter`")?;
            Ok(math_delimiter(left, right, body))
        }
        MathDelimitersOp::Paren => {
            let [body] = take1(args, "`math/delimiters paren`")?;
            Ok(math_delimiter_str("(", ")", body))
        }
        MathDelimitersOp::Brackets => {
            let [body] = take1(args, "`math/delimiters brackets`")?;
            Ok(math_delimiter_str("[", "]", body))
        }
        MathDelimitersOp::Braces => {
            let [body] = take1(args, "`math/delimiters braces`")?;
            Ok(math_delimiter_str("{", "}", body))
        }
        MathDelimitersOp::Angles => {
            let [body] = take1(args, "`math/delimiters angles`")?;
            Ok(math_delimiter_str("⟨", "⟩", body))
        }
        MathDelimitersOp::Abs => {
            let [body] = take1(args, "`math/delimiters abs`")?;
            Ok(math_delimiter_str("|", "|", body))
        }
        MathDelimitersOp::Floor => {
            let [body] = take1(args, "`math/delimiters floor`")?;
            Ok(math_delimiter_str("⌊", "⌋", body))
        }
        MathDelimitersOp::Ceil => {
            let [body] = take1(args, "`math/delimiters ceil`")?;
            Ok(math_delimiter_str("⌈", "⌉", body))
        }
    }
}

fn call_math_matrix(op: MathMatrixOp, args: &[RuntimeValue]) -> Result<RuntimeValue, EvalError> {
    match op {
        MathMatrixOp::Matrix => {
            let [rows] = take1(args, "`math/matrix matrix`")?;
            Ok(tagged(
                "math-matrix",
                vec![("kind", str_val("matrix")), ("rows", rows.clone())],
            ))
        }
        MathMatrixOp::Bmatrix => {
            let [rows] = take1(args, "`math/matrix bmatrix`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-matrix")),
                ("kind".into(), str_val("bmatrix")),
                ("rows".into(), rows.clone()),
                ("left".into(), str_val("[")),
                ("right".into(), str_val("]")),
            ]))
        }
        MathMatrixOp::Pmatrix => {
            let [rows] = take1(args, "`math/matrix pmatrix`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-matrix")),
                ("kind".into(), str_val("pmatrix")),
                ("rows".into(), rows.clone()),
                ("left".into(), str_val("(")),
                ("right".into(), str_val(")")),
            ]))
        }
        MathMatrixOp::Vmatrix => {
            let [rows] = take1(args, "`math/matrix vmatrix`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-matrix")),
                ("kind".into(), str_val("vmatrix")),
                ("rows".into(), rows.clone()),
                ("left".into(), str_val("|")),
                ("right".into(), str_val("|")),
            ]))
        }
        MathMatrixOp::MatrixRow => {
            let [cells] = take1(args, "`math/matrix matrix-row`")?;
            Ok(tagged("math-matrix-row", vec![("cells", cells.clone())]))
        }
        MathMatrixOp::Smallmatrix => {
            let [rows] = take1(args, "`math/matrix smallmatrix`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-matrix")),
                ("kind".into(), str_val("smallmatrix")),
                ("rows".into(), rows.clone()),
                ("script-style".into(), bool_val(true)),
            ]))
        }
        MathMatrixOp::MatrixEnv => {
            let [rows] = take1(args, "`math/matrix matrix-env`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-matrix-env")),
                ("kind".into(), str_val("matrix")),
                ("rows".into(), rows.clone()),
                ("delimiters".into(), str_val("none")),
            ]))
        }
        MathMatrixOp::ArrayEnv => {
            let [column_align, rows] = take2(args, "`math/matrix array-env`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-matrix-env")),
                ("kind".into(), str_val("array")),
                ("rows".into(), rows.clone()),
                ("column-align".into(), column_align.clone()),
            ]))
        }
        MathMatrixOp::MatrixDelim => {
            let [left, right, rows] = take3(args, "`math/matrix matrix-delim`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-matrix")),
                ("kind".into(), str_val("delimited")),
                ("left".into(), left.clone()),
                ("right".into(), right.clone()),
                ("rows".into(), rows.clone()),
            ]))
        }
        MathMatrixOp::BmatrixEnv => {
            let [rows] = take1(args, "`math/matrix bmatrix-env`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-matrix")),
                ("kind".into(), str_val("delimited-bmatrix")),
                ("left".into(), str_val("[")),
                ("right".into(), str_val("]")),
                ("rows".into(), rows.clone()),
            ]))
        }
        MathMatrixOp::PmatrixEnv => {
            let [rows] = take1(args, "`math/matrix pmatrix-env`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-matrix")),
                ("kind".into(), str_val("delimited-pmatrix")),
                ("left".into(), str_val("(")),
                ("right".into(), str_val(")")),
                ("rows".into(), rows.clone()),
            ]))
        }
    }
}

fn call_math_accents(op: MathAccentsOp, args: &[RuntimeValue]) -> Result<RuntimeValue, EvalError> {
    match op {
        MathAccentsOp::Accent => {
            let [kind, base] = take2(args, "`math/accents accent`")?;
            Ok(tagged(
                "math-accent",
                vec![("kind", kind.clone()), ("base", base.clone())],
            ))
        }
        MathAccentsOp::Hat => accent_kind("hat", args),
        MathAccentsOp::Bar => accent_kind("bar", args),
        MathAccentsOp::Vec => accent_kind("vec", args),
        MathAccentsOp::Tilde => accent_kind("tilde", args),
        MathAccentsOp::Dot => accent_kind("dot", args),
        MathAccentsOp::Ddot => accent_kind("ddot", args),
        MathAccentsOp::Check => accent_kind("check", args),
        MathAccentsOp::Breve => accent_kind("breve", args),
        MathAccentsOp::Acute => accent_kind("acute", args),
        MathAccentsOp::Grave => accent_kind("grave", args),
        MathAccentsOp::Ring => accent_kind("ring", args),
        MathAccentsOp::Overline => accent_kind("overline", args),
        MathAccentsOp::Underline => accent_kind("underline", args),
        MathAccentsOp::Underbar => accent_kind("underbar", args),
        MathAccentsOp::Widehat => accent_kind("widehat", args),
        MathAccentsOp::Widetilde => accent_kind("widetilde", args),
    }
}

fn call_math_bigops(op: MathBigopsOp, args: &[RuntimeValue]) -> Result<RuntimeValue, EvalError> {
    match op {
        MathBigopsOp::Bigop => {
            let [glyph, lower, upper, body] = take4(args, "`math/bigops bigop`")?;
            Ok(math_bigop(glyph, lower, upper, body, None))
        }
        MathBigopsOp::Sum => {
            let [lower, upper, body] = take3(args, "`math/bigops sum`")?;
            Ok(math_bigop_str("∑", lower, upper, body, None))
        }
        MathBigopsOp::Prod => {
            let [lower, upper, body] = take3(args, "`math/bigops prod`")?;
            Ok(math_bigop_str("∏", lower, upper, body, None))
        }
        MathBigopsOp::Integral => {
            let [lower, upper, body] = take3(args, "`math/bigops integral`")?;
            Ok(math_bigop_str("∫", lower, upper, body, None))
        }
        MathBigopsOp::Oint => {
            let [lower, upper, body] = take3(args, "`math/bigops oint`")?;
            Ok(math_bigop_str("∮", lower, upper, body, None))
        }
        MathBigopsOp::Lim => {
            let [lower, body] = take2(args, "`math/bigops lim`")?;
            Ok(math_bigop_str("lim", lower, &math_absent(), body, None))
        }
        MathBigopsOp::Limsup => {
            let [lower, body] = take2(args, "`math/bigops limsup`")?;
            Ok(math_bigop_str("limsup", lower, &math_absent(), body, None))
        }
        MathBigopsOp::Liminf => {
            let [lower, body] = take2(args, "`math/bigops liminf`")?;
            Ok(math_bigop_str("liminf", lower, &math_absent(), body, None))
        }
        MathBigopsOp::Max => {
            let [lower, body] = take2(args, "`math/bigops max`")?;
            Ok(math_bigop_str("max", lower, &math_absent(), body, None))
        }
        MathBigopsOp::Min => {
            let [lower, body] = take2(args, "`math/bigops min`")?;
            Ok(math_bigop_str("min", lower, &math_absent(), body, None))
        }
        MathBigopsOp::Withlimits => {
            let [op, lower, upper, body] = take4(args, "`math/bigops withlimits`")?;
            Ok(math_bigop(op, lower, upper, body, Some("display")))
        }
        MathBigopsOp::Nolimits => {
            let [op, body] = take2(args, "`math/bigops nolimits`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-bigop")),
                ("glyph".into(), op.clone()),
                ("lower".into(), math_absent()),
                ("upper".into(), math_absent()),
                ("body".into(), body.clone()),
                ("limits".into(), str_val("script")),
            ]))
        }
        MathBigopsOp::SumNolimits => {
            let [body] = take1(args, "`math/bigops sum-nolimits`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-bigop")),
                ("glyph".into(), str_val("∑")),
                ("lower".into(), math_absent()),
                ("upper".into(), math_absent()),
                ("body".into(), body.clone()),
            ]))
        }
        MathBigopsOp::IntegralNolimits => {
            let [body] = take1(args, "`math/bigops integral-nolimits`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-bigop")),
                ("glyph".into(), str_val("∫")),
                ("lower".into(), math_absent()),
                ("upper".into(), math_absent()),
                ("body".into(), body.clone()),
            ]))
        }
        MathBigopsOp::BigopScripts => {
            let [glyph, sup, sub, body] = take4(args, "`math/bigops bigop-scripts`")?;
            Ok(math_bigop_scripts(glyph, sup, sub, body))
        }
        MathBigopsOp::SumScripts => {
            let [sup, sub, body] = take3(args, "`math/bigops sum-scripts`")?;
            Ok(math_bigop_scripts_str("∑", sup, sub, body))
        }
        MathBigopsOp::ProdScripts => {
            let [sup, sub, body] = take3(args, "`math/bigops prod-scripts`")?;
            Ok(math_bigop_scripts_str("∏", sup, sub, body))
        }
        MathBigopsOp::IntegralScripts => {
            let [sup, sub, body] = take3(args, "`math/bigops integral-scripts`")?;
            Ok(math_bigop_scripts_str("∫", sup, sub, body))
        }
        MathBigopsOp::OintScripts => {
            let [sup, sub, body] = take3(args, "`math/bigops oint-scripts`")?;
            Ok(math_bigop_scripts_str("∮", sup, sub, body))
        }
    }
}

fn call_math_cases(op: MathCasesOp, args: &[RuntimeValue]) -> Result<RuntimeValue, EvalError> {
    match op {
        MathCasesOp::CaseArm => {
            let [body, guard] = take2(args, "`math/cases case-arm`")?;
            Ok(tagged(
                "math-case-arm",
                vec![("body", body.clone()), ("guard", guard.clone())],
            ))
        }
        MathCasesOp::Cases => {
            let [arms] = take1(args, "`math/cases cases`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-cases")),
                ("arms".into(), arms.clone()),
                ("left".into(), str_val("{")),
                ("right".into(), str_val("")),
            ]))
        }
        MathCasesOp::CasesLr | MathCasesOp::CasesDelim => {
            let [left, right, arms] = take3(args, "`math/cases cases-lr`")?;
            let mut fields = vec![
                ("tag".into(), str_val("math-cases")),
                ("arms".into(), arms.clone()),
                ("left".into(), left.clone()),
                ("right".into(), right.clone()),
            ];
            if matches!(op, MathCasesOp::CasesDelim) {
                fields.push(("delimited".into(), bool_val(true)));
            }
            Ok(record(fields))
        }
        MathCasesOp::LeftCases => {
            let [arms] = take1(args, "`math/cases left-cases`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-cases")),
                ("arms".into(), arms.clone()),
                ("left".into(), str_val("{")),
                ("right".into(), str_val("")),
                ("delimited".into(), bool_val(true)),
            ]))
        }
        MathCasesOp::RightCases => {
            let [arms] = take1(args, "`math/cases right-cases`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-cases")),
                ("arms".into(), arms.clone()),
                ("left".into(), str_val("")),
                ("right".into(), str_val("}")),
                ("delimited".into(), bool_val(true)),
            ]))
        }
        MathCasesOp::Piecewise => {
            let [arms] = take1(args, "`math/cases piecewise`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-cases")),
                ("arms".into(), arms.clone()),
                ("left".into(), str_val("{")),
                ("right".into(), str_val("")),
                ("kind".into(), str_val("piecewise")),
            ]))
        }
        MathCasesOp::Otherwise => {
            let [body] = take1(args, "`math/cases otherwise`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-case-arm")),
                ("body".into(), body.clone()),
                ("guard".into(), math_symbol_class_str("otherwise", "ord")),
            ]))
        }
    }
}

fn call_math_align(op: MathAlignOp, args: &[RuntimeValue]) -> Result<RuntimeValue, EvalError> {
    match op {
        MathAlignOp::AlignRow => {
            let [cells] = take1(args, "`math/align align-row`")?;
            Ok(tagged("math-align-row", vec![("cells", cells.clone())]))
        }
        MathAlignOp::Aligned => {
            let [rows] = take1(args, "`math/align aligned`")?;
            Ok(tagged("math-aligned", vec![("rows", rows.clone())]))
        }
        MathAlignOp::Align => {
            let [position, body] = take2(args, "`math/align align`")?;
            Ok(tagged(
                "math-align",
                vec![("position", position.clone()), ("body", body.clone())],
            ))
        }
        MathAlignOp::AlignLeft => {
            let [body] = take1(args, "`math/align align-left`")?;
            Ok(tagged(
                "math-align",
                vec![("position", str_val("left")), ("body", body.clone())],
            ))
        }
        MathAlignOp::AlignCenter => {
            let [body] = take1(args, "`math/align align-center`")?;
            Ok(tagged(
                "math-align",
                vec![("position", str_val("center")), ("body", body.clone())],
            ))
        }
        MathAlignOp::AlignRight => {
            let [body] = take1(args, "`math/align align-right`")?;
            Ok(tagged(
                "math-align",
                vec![("position", str_val("right")), ("body", body.clone())],
            ))
        }
        MathAlignOp::AlignAt => {
            let [marker, body] = take2(args, "`math/align align-at`")?;
            Ok(tagged(
                "math-align-at",
                vec![("marker", marker.clone()), ("body", body.clone())],
            ))
        }
        MathAlignOp::AlignEq => {
            let [rows] = take1(args, "`math/align align-eq`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-align-eq")),
                ("rows".into(), rows.clone()),
                ("relation".into(), str_val("=")),
            ]))
        }
    }
}

fn call_math_stack(op: MathStackOp, args: &[RuntimeValue]) -> Result<RuntimeValue, EvalError> {
    match op {
        MathStackOp::Stack => {
            let [children] = take1(args, "`math/stack stack`")?;
            Ok(tagged("math-stack", vec![("children", children.clone())]))
        }
        MathStackOp::Stackrel => {
            let [relation, base] = take2(args, "`math/stack stackrel`")?;
            Ok(tagged(
                "math-stackrel",
                vec![("relation", relation.clone()), ("base", base.clone())],
            ))
        }
        MathStackOp::OversetRel => {
            let [relation, base] = take2(args, "`math/stack overset-rel`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-stackrel")),
                ("relation".into(), relation.clone()),
                ("base".into(), base.clone()),
                ("kind".into(), str_val("overset")),
            ]))
        }
        MathStackOp::UndersetRel => {
            let [relation, base] = take2(args, "`math/stack underset-rel`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-stackrel")),
                ("relation".into(), relation.clone()),
                ("base".into(), base.clone()),
                ("kind".into(), str_val("underset")),
            ]))
        }
        MathStackOp::Atop => {
            let [top, bottom] = take2(args, "`math/stack atop`")?;
            Ok(record(vec![
                ("tag".into(), str_val("math-stack")),
                ("children".into(), cons_list([top.clone(), bottom.clone()])),
                ("kind".into(), str_val("atop")),
            ]))
        }
        MathStackOp::Substack => {
            let [rows] = take1(args, "`math/stack substack`")?;
            Ok(tagged("math-substack", vec![("rows", rows.clone())]))
        }
    }
}

fn constant_str(value: &str, args: &[RuntimeValue], ctx: &str) -> Result<RuntimeValue, EvalError> {
    take0(args, ctx)?;
    Ok(str_val(value))
}

fn accent_kind(kind: &str, args: &[RuntimeValue]) -> Result<RuntimeValue, EvalError> {
    let [base] = take1(args, &format!("`math/accents {kind}`"))?;
    Ok(tagged(
        "math-accent",
        vec![("kind", str_val(kind)), ("base", base.clone())],
    ))
}

fn math_symbol(glyph: &RuntimeValue, class: &RuntimeValue) -> RuntimeValue {
    record(vec![
        ("tag".into(), str_val("math-symbol")),
        ("glyph".into(), glyph.clone()),
        ("class".into(), class.clone()),
    ])
}

fn math_symbol_class(glyph: &RuntimeValue, class: &str) -> RuntimeValue {
    record(vec![
        ("tag".into(), str_val("math-symbol")),
        ("glyph".into(), glyph.clone()),
        ("class".into(), str_val(class)),
    ])
}

fn math_symbol_class_str(glyph: &str, class: &str) -> RuntimeValue {
    record(vec![
        ("tag".into(), str_val("math-symbol")),
        ("glyph".into(), str_val(glyph)),
        ("class".into(), str_val(class)),
    ])
}

fn math_fraction(numerator: &RuntimeValue, denominator: &RuntimeValue) -> RuntimeValue {
    record(vec![
        ("tag".into(), str_val("math-fraction")),
        ("numerator".into(), numerator.clone()),
        ("denominator".into(), denominator.clone()),
    ])
}

fn math_scripts(base: &RuntimeValue, sup: &RuntimeValue, sub: &RuntimeValue) -> RuntimeValue {
    record(vec![
        ("tag".into(), str_val("math-scripts")),
        ("base".into(), base.clone()),
        ("superscript".into(), sup.clone()),
        ("subscript".into(), sub.clone()),
    ])
}

fn math_absent() -> RuntimeValue {
    record(vec![("tag".into(), str_val("math-absent"))])
}

fn math_radical(index: &RuntimeValue, radicand: &RuntimeValue) -> RuntimeValue {
    record(vec![
        ("tag".into(), str_val("math-radical")),
        ("index".into(), index.clone()),
        ("radicand".into(), radicand.clone()),
    ])
}

fn math_delimiter(left: &RuntimeValue, right: &RuntimeValue, body: &RuntimeValue) -> RuntimeValue {
    record(vec![
        ("tag".into(), str_val("math-delimiter")),
        ("left".into(), left.clone()),
        ("right".into(), right.clone()),
        ("body".into(), body.clone()),
    ])
}

fn math_delimiter_str(left: &str, right: &str, body: &RuntimeValue) -> RuntimeValue {
    record(vec![
        ("tag".into(), str_val("math-delimiter")),
        ("left".into(), str_val(left)),
        ("right".into(), str_val(right)),
        ("body".into(), body.clone()),
    ])
}

fn math_bigop(
    glyph: &RuntimeValue,
    lower: &RuntimeValue,
    upper: &RuntimeValue,
    body: &RuntimeValue,
    limits: Option<&str>,
) -> RuntimeValue {
    let mut fields = vec![
        ("tag".into(), str_val("math-bigop")),
        ("glyph".into(), glyph.clone()),
        ("lower".into(), lower.clone()),
        ("upper".into(), upper.clone()),
        ("body".into(), body.clone()),
    ];
    if let Some(limits) = limits {
        fields.push(("limits".into(), str_val(limits)));
    }
    record(fields)
}

fn math_bigop_str(
    glyph: &str,
    lower: &RuntimeValue,
    upper: &RuntimeValue,
    body: &RuntimeValue,
    limits: Option<&str>,
) -> RuntimeValue {
    math_bigop(&str_val(glyph), lower, upper, body, limits)
}

fn math_bigop_scripts(
    glyph: &RuntimeValue,
    sup: &RuntimeValue,
    sub: &RuntimeValue,
    body: &RuntimeValue,
) -> RuntimeValue {
    record(vec![
        ("tag".into(), str_val("math-bigop-scripts")),
        ("glyph".into(), glyph.clone()),
        ("superscript".into(), sup.clone()),
        ("subscript".into(), sub.clone()),
        ("body".into(), body.clone()),
    ])
}

fn math_bigop_scripts_str(
    glyph: &str,
    sup: &RuntimeValue,
    sub: &RuntimeValue,
    body: &RuntimeValue,
) -> RuntimeValue {
    math_bigop_scripts(&str_val(glyph), sup, sub, body)
}

fn tagged(tag: &str, fields: Vec<(&str, RuntimeValue)>) -> RuntimeValue {
    let mut out = vec![("tag".into(), str_val(tag))];
    for (k, v) in fields {
        out.push((k.into(), v));
    }
    record(out)
}

fn cons_list(items: [RuntimeValue; 2]) -> RuntimeValue {
    let mut acc = RuntimeValue::Variant {
        tag: "nil".into(),
        payload: None,
    };
    for item in items.into_iter().rev() {
        acc = RuntimeValue::Variant {
            tag: "cons".into(),
            payload: Some(Box::new(RuntimeValue::Record(vec![
                ("head".into(), item),
                ("tail".into(), acc),
            ]))),
        };
    }
    acc
}

fn record(fields: Vec<(String, RuntimeValue)>) -> RuntimeValue {
    RuntimeValue::Record(fields)
}

fn str_val(s: &str) -> RuntimeValue {
    RuntimeValue::String(s.into())
}

fn bool_val(b: bool) -> RuntimeValue {
    RuntimeValue::Bool(b)
}
