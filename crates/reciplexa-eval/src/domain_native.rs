//! Domain-native package callables (Direct Native v2).
//!
//! Separate from kernel [`crate::value::BuiltinOp`]: these implement standard
//! package exports keyed by `package/module/export`.

use crate::value::RuntimeValue;
use crate::EvalError;

/// DN2-2 — `color/srgb` pure constructors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSrgbOp {
    Srgb,
    Rgb,
    Rgba,
    FromByte,
    Gray,
    Black,
    White,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    Orange,
    Gray50,
    Transparent,
    WithAlpha,
}

/// DN2-2 — `graphics/color` pure constructors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphicsColorOp {
    Rgb,
    Rgba,
    Black,
    White,
    Red,
    Green,
    Blue,
    Gray,
    Transparent,
}

/// DN2-2 — `graphics/page` pure constructors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphicsPageOp {
    A4,
    Letter,
    A5,
    A3,
    Legal,
    Square,
    Page,
    Pages,
    PageSize,
}

/// DN2-3 — `math/atoms` constructors and class constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathAtomsOp {
    ClassOrd,
    ClassOp,
    ClassBin,
    ClassRel,
    ClassOpen,
    ClassClose,
    ClassPunct,
    ClassFence,
    Symbol,
    Ord,
    Op,
    Bin,
    Rel,
    Open,
    Close,
    Punct,
    Fence,
    Row,
    Operatorname,
    Mathrm,
    Mathbf,
    Textop,
    Fraction,
    Superscript,
    Subscript,
}

/// DN2-3 — `math/scripts` superscript / subscript / over / under.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathScriptsOp {
    Absent,
    Superscript,
    Subscript,
    Scripts,
    Super,
    Sub,
    Overline,
    Underline,
    Overbrace,
    Underbrace,
    Overset,
    Underset,
    Over,
    Under,
}

/// DN2-3 — `math/frac` fraction constructors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathFracOp {
    Fraction,
    Over,
}

/// DN2-3 — `math/sqrt` radical constructors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathSqrtOp {
    AbsentIndex,
    Sqrt,
    Radical,
    RadicalIndexed,
    Root,
}

/// DN2-3 — `math/delimiters` delimiter / fence wrappers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathDelimitersOp {
    Delimiter,
    Paren,
    Brackets,
    Braces,
    Angles,
    Abs,
    Floor,
    Ceil,
}

/// DN2-3 — `math/matrix` matrix constructors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathMatrixOp {
    Matrix,
    Bmatrix,
    Pmatrix,
    Vmatrix,
    MatrixRow,
    Smallmatrix,
    MatrixEnv,
    ArrayEnv,
    MatrixDelim,
    BmatrixEnv,
    PmatrixEnv,
}

/// DN2-3 — `math/accents` accent wrappers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathAccentsOp {
    Accent,
    Hat,
    Bar,
    Vec,
    Tilde,
    Dot,
    Ddot,
    Check,
    Breve,
    Acute,
    Grave,
    Ring,
    Overline,
    Underline,
    Underbar,
    Widehat,
    Widetilde,
}

/// DN2-3 — `math/bigops` large operators with optional limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathBigopsOp {
    Bigop,
    Sum,
    Prod,
    Integral,
    Oint,
    Lim,
    Limsup,
    Liminf,
    Max,
    Min,
    Withlimits,
    Nolimits,
    SumNolimits,
    IntegralNolimits,
    BigopScripts,
    SumScripts,
    ProdScripts,
    IntegralScripts,
    OintScripts,
}

/// DN2-3 — `math/cases` piecewise / cases constructs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathCasesOp {
    CaseArm,
    Cases,
    CasesLr,
    CasesDelim,
    LeftCases,
    RightCases,
    Piecewise,
    Otherwise,
}

/// DN2-3 — `math/align` alignment rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathAlignOp {
    AlignRow,
    Aligned,
    Align,
    AlignLeft,
    AlignCenter,
    AlignRight,
    AlignAt,
    AlignEq,
}

/// DN2-3 — `math/stack` vertical stacks and relation symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathStackOp {
    Stack,
    Stackrel,
    OversetRel,
    UndersetRel,
    Atop,
    Substack,
}

/// DN2-4 — `japanese/classes` JLReq character class tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JapaneseClassesOp {
    Class,
    AllClassIds,
    AllClasses,
    ClassName,
    ClassNameJa,
    Cl01,
    Cl02,
    Cl03,
    Cl04,
    Cl05,
    Cl06,
    Cl07,
    Cl08,
    Cl09,
    Cl10,
    Cl11,
    Cl12,
    Cl13,
    Cl14,
    Cl15,
    Cl16,
    Cl17,
    Cl18,
    Cl19,
    Cl20,
    Cl21,
    Cl22,
    Cl23,
    Cl24,
    Cl25,
    Cl26,
    Cl27,
    Cl28,
    Cl29,
    Cl30,
    AdvanceEm,
    IsSquareLetter,
    IsPunctuationClass,
    IsKanaClass,
    IsWesternClass,
}

/// DN2-4 — `japanese/linebreak` kinsoku / break-opportunity exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JapaneseLinebreakOp {
    SampleLineHeadProhibited,
    SampleLineEndProhibited,
    SampleInseparable,
    Opportunity,
    LineHeadProhibitedClass,
    LineEndProhibitedClass,
    InseparablePair,
    PairRule,
    SamplePairRules,
    BreakBetween,
    HangableClass,
    NumericBeforeCloseProhibited,
    KinsokuProfile,
    ClassifySample,
}

/// DN2-4 — `japanese/kihon` kihon-hanmen / line-rate / vertical stubs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JapaneseKihonOp {
    WritingModeHorizontal,
    WritingModeVertical,
    LineRateSolid,
    LineRateCompact,
    LineRateDefault,
    LineRateRelaxed,
    LineRateLoose,
    SolidSetting,
    CharacterFrame,
    LineMetrics,
    KihonHanmen,
    DefaultHorizontalKihon,
    DefaultVerticalKihon,
    HeadingBandEm,
    IndentEm,
    TrimSize,
    A5Trim,
    B5JisTrim,
    A4Trim,
    Margins,
    PlaceHanmen,
    ColumnCountOne,
    ColumnCountTwo,
    MultiColumn,
    VerticalFlow,
    TateDigits,
    VerticalStack,
    TateChuYokoSpan,
    VerticalText,
    PlaceVerticalText,
    VerticalTextStack,
}

/// DN2-4 — `japanese/markup` document-oriented JP markup helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JapaneseMarkupOp {
    Heading,
    Section,
    Paragraph,
    Note,
    Emphasis,
    Warn,
    Todo,
    Ruby,
    JukugoRuby,
    TateChuYoko,
    TategakiParagraph,
    TategakiText,
    MarkupBridge,
    DocWithMarkup,
    Ul,
    Doc,
}

/// DN2-5 — `document/page` flow-oriented doc record constructors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentPageOp {
    A4,
    Letter,
    A5,
    A3,
    Legal,
    Page,
    Flow,
    Section,
    Heading,
    Paragraph,
    ParagraphIndented,
    Columns,
    UnorderedList,
    OrderedList,
    ListItem,
    Table,
    Figure,
    Note,
    Spacer,
    BlockHeading,
    BlockParagraph,
    BlockColumns,
    BlockList,
    BlockTable,
    BlockFigure,
    BlockNote,
    BlockSpacer,
    /// Explicit page break: next flow block starts a new scene page.
    PageBreak,
    BlockPageBreak,
    /// 版面 margins in mm `(top right bottom left)`.
    Margins,
    /// `(page-framed paper margins flow)` — same `doc-page` tag plus margins.
    PageFramed,
}

/// DN2-2 — `graphics/shapes` pure constructors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphicsShapesOp {
    Circle,
    Rect,
    Ellipse,
    Line,
    Path,
    Polyline,
    Polygon,
    Ring,
    Frame,
    Group,
    Text,
    TextBox,
    Image,
    Translate,
    Rotate,
    Scale,
    Opacity,
    Fill,
    Stroke,
    Paint,
}

/// Stable dispatch id for a typed package export callable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DomainNativeOp {
    /// DN2-0 harness: `(native/test ping)` → `42`.
    TestPing,
    /// DN2-1 — `length/units` unit constructors and helpers.
    LengthUnitsMm,
    LengthUnitsCm,
    LengthUnitsPt,
    LengthUnitsBp,
    LengthUnitsInch,
    LengthUnitsQ,
    LengthUnitsPx,
    LengthUnitsEm,
    LengthUnitsZero,
    LengthUnitsToMm,
    LengthUnitsFromMm,
    LengthUnitsAddMm,
    LengthUnitsScaleLength,
    /// DN2-2 — pure record constructors.
    ColorSrgb(ColorSrgbOp),
    GraphicsColor(GraphicsColorOp),
    GraphicsPage(GraphicsPageOp),
    GraphicsShapes(GraphicsShapesOp),
    /// DN2-3 — math record constructors.
    MathAtoms(MathAtomsOp),
    MathScripts(MathScriptsOp),
    MathFrac(MathFracOp),
    MathSqrt(MathSqrtOp),
    MathDelimiters(MathDelimitersOp),
    MathMatrix(MathMatrixOp),
    MathAccents(MathAccentsOp),
    MathBigops(MathBigopsOp),
    MathCases(MathCasesOp),
    MathAlign(MathAlignOp),
    MathStack(MathStackOp),
    /// DN2-4 — japanese record constructors.
    JapaneseClasses(JapaneseClassesOp),
    JapaneseLinebreak(JapaneseLinebreakOp),
    JapaneseKihon(JapaneseKihonOp),
    JapaneseMarkup(JapaneseMarkupOp),
    /// DN2-5 — `document/page` doc record constructors.
    DocumentPage(DocumentPageOp),
}

/// Qualified registry key: `package/module/export`.
pub fn qualified_export_key(module_path: &str, export_name: &str) -> String {
    format!("{module_path}/{export_name}")
}

/// Internal eval/typecheck slot for a native export binding (valid RPX kebab ident).
pub fn dn2_slot(module_path: &str, export_name: &str) -> String {
    format!("dn2slot-{}-{}", module_path.replace('/', "-"), export_name)
}

/// Apply a domain-native callable (arity-checked).
pub fn call_domain_native(
    op: DomainNativeOp,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, EvalError> {
    match op {
        DomainNativeOp::TestPing => {
            crate::domain_native_failure::take0(args, "native/test/ping")?;
            Ok(RuntimeValue::Int(42))
        }
        DomainNativeOp::LengthUnitsMm
        | DomainNativeOp::LengthUnitsCm
        | DomainNativeOp::LengthUnitsPt
        | DomainNativeOp::LengthUnitsBp
        | DomainNativeOp::LengthUnitsInch
        | DomainNativeOp::LengthUnitsQ
        | DomainNativeOp::LengthUnitsPx
        | DomainNativeOp::LengthUnitsEm
        | DomainNativeOp::LengthUnitsZero
        | DomainNativeOp::LengthUnitsToMm
        | DomainNativeOp::LengthUnitsFromMm
        | DomainNativeOp::LengthUnitsAddMm
        | DomainNativeOp::LengthUnitsScaleLength => {
            crate::domain_length_units::call_length_units(op, args)
        }
        DomainNativeOp::ColorSrgb(_)
        | DomainNativeOp::GraphicsColor(_)
        | DomainNativeOp::GraphicsPage(_)
        | DomainNativeOp::GraphicsShapes(_) => {
            crate::domain_pure_constructors::call_pure_constructor(op, args)
        }
        DomainNativeOp::MathAtoms(_)
        | DomainNativeOp::MathScripts(_)
        | DomainNativeOp::MathFrac(_)
        | DomainNativeOp::MathSqrt(_)
        | DomainNativeOp::MathDelimiters(_)
        | DomainNativeOp::MathMatrix(_)
        | DomainNativeOp::MathAccents(_)
        | DomainNativeOp::MathBigops(_)
        | DomainNativeOp::MathCases(_)
        | DomainNativeOp::MathAlign(_)
        | DomainNativeOp::MathStack(_) => {
            crate::domain_math_constructors::call_math_constructor(op, args)
        }
        DomainNativeOp::JapaneseClasses(_)
        | DomainNativeOp::JapaneseLinebreak(_)
        | DomainNativeOp::JapaneseKihon(_)
        | DomainNativeOp::JapaneseMarkup(_) => {
            crate::domain_japanese_constructors::call_japanese_constructor(op, args)
        }
        DomainNativeOp::DocumentPage(_) => {
            crate::domain_document_constructors::call_document_constructor(op, args)
        }
    }
}
