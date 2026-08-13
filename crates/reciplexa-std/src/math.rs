//! `rpx.std.math` — editable math tree (not text decoration / glyph layout).
//!
//! Aligns with `packages/math` SATySFi-shaped constructors. Honest scope:
//! trees + light metric stubs for hosts — not OpenType MATH layout.
//! See `lang/ja-math-deepen-plan.md` (M0–M3).
//!
//! Package synthetic RPX remains the Core import surface; prefer these Rust
//! APIs (`MathAtom`, `estimate_box`) for future layout and hosts.

use std::fmt;

use reciplexa_identity::document::StableNodeId;

/// Math atom class (TeX-like).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathClass {
    Ordinary,
    Operator,
    Binary,
    Relation,
    Open,
    Close,
    Punctuation,
    Fence,
}

impl MathClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ordinary => "ord",
            Self::Operator => "op",
            Self::Binary => "bin",
            Self::Relation => "rel",
            Self::Open => "open",
            Self::Close => "close",
            Self::Punctuation => "punct",
            Self::Fence => "fence",
        }
    }
}

impl fmt::Display for MathClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Thin muskip stub (TeX `\,` ≈ 3mu; 18mu = 1em).
pub const THIN_MUSKIP_EM: f64 = 3.0 / 18.0;

/// Medium muskip stub (TeX `\>` ≈ 4mu).
pub const MED_MUSKIP_EM: f64 = 4.0 / 18.0;

/// Thick muskip stub (TeX `\;` ≈ 5mu).
pub const THICK_MUSKIP_EM: f64 = 5.0 / 18.0;

/// TeX-ish inter-atom spacing (em) between adjacent math classes.
///
/// Fontless heuristic for row width estimates — not style-dependent `\scriptstyle`
/// suppression, not OpenType MATH `MathItalicsCorrection` / `MathKern`. Covers the
/// common Ord/Op/Bin/Rel/Open/Close/Punct/Fence pairs; unknown pairs → `0.0`.
pub fn class_spacing_em(left: MathClass, right: MathClass) -> f64 {
    use MathClass::*;
    match (left, right) {
        // Thin: Ord–Op, Op–Ord, Op–Op, Close–Op, …
        (Ordinary, Operator) | (Operator, Ordinary) => THIN_MUSKIP_EM,
        (Operator, Operator) => THIN_MUSKIP_EM,
        (Close, Operator) | (Operator, Open) => THIN_MUSKIP_EM,
        (Punctuation, _) => THIN_MUSKIP_EM,
        (Ordinary, Punctuation)
        | (Operator, Punctuation)
        | (Close, Punctuation)
        | (Relation, Punctuation)
        | (Fence, Punctuation) => THIN_MUSKIP_EM,
        // Medium: Ord–Bin–Ord; Bin with Op; Close–Bin / Bin–Open.
        (Ordinary, Binary) | (Binary, Ordinary) => MED_MUSKIP_EM,
        (Close, Binary) | (Binary, Open) => MED_MUSKIP_EM,
        (Operator, Binary) | (Binary, Operator) => MED_MUSKIP_EM,
        // Thick: Ord–Rel–Ord; Bin–Rel; Op–Rel.
        (Ordinary, Relation) | (Relation, Ordinary) => THICK_MUSKIP_EM,
        (Close, Relation) | (Relation, Open) => THICK_MUSKIP_EM,
        (Operator, Relation) | (Relation, Operator) => THICK_MUSKIP_EM,
        (Binary, Relation) | (Relation, Binary) => THICK_MUSKIP_EM,
        // Fence ≈ Open/Close for spacing purposes.
        (Fence, Operator) | (Operator, Fence) => THIN_MUSKIP_EM,
        (Fence, Binary) | (Binary, Fence) => MED_MUSKIP_EM,
        (Fence, Relation) | (Relation, Fence) => THICK_MUSKIP_EM,
        (Ordinary, Fence) | (Fence, Ordinary) => 0.0,
        (Close, Fence) | (Fence, Open) => 0.0,
        _ => 0.0,
    }
}

/// Accent kind for `MathAtom::Accent` (package `math/accents`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathAccentKind {
    Hat,
    Bar,
    Vec,
    Tilde,
    Dot,
    Ddot,
    /// TeX-ish `\check` (caron).
    Check,
    /// TeX-ish `\breve`.
    Breve,
    /// TeX-ish `\acute`.
    Acute,
    /// TeX-ish `\grave`.
    Grave,
    /// TeX-ish `\mathring` / ring.
    Ring,
    Overline,
    Underline,
    /// Compact under-bar (distinct from roomier underline/underbrace).
    Underbar,
    WideHat,
    WideTilde,
}

impl MathAccentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hat => "hat",
            Self::Bar => "bar",
            Self::Vec => "vec",
            Self::Tilde => "tilde",
            Self::Dot => "dot",
            Self::Ddot => "ddot",
            Self::Check => "check",
            Self::Breve => "breve",
            Self::Acute => "acute",
            Self::Grave => "grave",
            Self::Ring => "ring",
            Self::Overline => "overline",
            Self::Underline => "underline",
            Self::Underbar => "underbar",
            Self::WideHat => "widehat",
            Self::WideTilde => "widetilde",
        }
    }

    /// Parse package / `math-accent` kind string.
    pub fn from_str_name(name: &str) -> Option<Self> {
        Some(match name {
            "hat" => Self::Hat,
            "bar" => Self::Bar,
            "vec" => Self::Vec,
            "tilde" => Self::Tilde,
            "dot" => Self::Dot,
            "ddot" => Self::Ddot,
            "check" => Self::Check,
            "breve" => Self::Breve,
            "acute" => Self::Acute,
            "grave" => Self::Grave,
            "ring" => Self::Ring,
            "overline" => Self::Overline,
            "underline" => Self::Underline,
            "underbar" => Self::Underbar,
            "widehat" => Self::WideHat,
            "widetilde" => Self::WideTilde,
            _ => return None,
        })
    }
}

impl fmt::Display for MathAccentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Leaf or internal math node.
#[derive(Debug, Clone, PartialEq)]
pub enum MathAtom {
    Symbol {
        id: StableNodeId,
        glyph: String,
        class: MathClass,
    },
    Row {
        id: StableNodeId,
        children: Vec<MathAtom>,
    },
    Fraction {
        id: StableNodeId,
        numerator: Box<MathAtom>,
        denominator: Box<MathAtom>,
    },
    Radical {
        id: StableNodeId,
        index: Option<Box<MathAtom>>,
        radicand: Box<MathAtom>,
    },
    Scripts {
        id: StableNodeId,
        base: Box<MathAtom>,
        superscript: Option<Box<MathAtom>>,
        subscript: Option<Box<MathAtom>>,
    },
    Delimiter {
        id: StableNodeId,
        left: String,
        right: String,
        body: Box<MathAtom>,
        /// Stretchy fence scale vs body height/depth (1.0 ≈ match body; stub only).
        stretch_factor: f64,
    },
    Accent {
        id: StableNodeId,
        kind: MathAccentKind,
        base: Box<MathAtom>,
    },
    BigOp {
        id: StableNodeId,
        operator: String,
        lower: Option<Box<MathAtom>>,
        upper: Option<Box<MathAtom>>,
        body: Option<Box<MathAtom>>,
    },
    /// Matrix / bmatrix / pmatrix-style grid (`math/matrix`).
    Matrix {
        id: StableNodeId,
        kind: MathMatrixKind,
        rows: Vec<Vec<MathAtom>>,
        left: Option<String>,
        right: Option<String>,
    },
    /// Alignment environment rows (`math/align`).
    Aligned {
        id: StableNodeId,
        rows: Vec<Vec<MathAtom>>,
    },
    /// Vertical stack / atop / substack (`math/stack`).
    Stack {
        id: StableNodeId,
        kind: MathStackKind,
        children: Vec<MathAtom>,
    },
}

/// Matrix delimiter flavor (package `math/matrix` kinds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathMatrixKind {
    Plain,
    BMatrix,
    PMatrix,
    VMatrix,
    Small,
    Delimited,
}

impl MathMatrixKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Plain => "matrix",
            Self::BMatrix => "bmatrix",
            Self::PMatrix => "pmatrix",
            Self::VMatrix => "vmatrix",
            Self::Small => "smallmatrix",
            Self::Delimited => "delimited",
        }
    }
}

impl fmt::Display for MathMatrixKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Vertical stack flavor (package `math/stack`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathStackKind {
    Stack,
    Atop,
    Substack,
    Stackrel,
}

impl MathStackKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stack => "stack",
            Self::Atop => "atop",
            Self::Substack => "substack",
            Self::Stackrel => "stackrel",
        }
    }
}

impl fmt::Display for MathStackKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl MathAtom {
    pub fn id(&self) -> StableNodeId {
        match self {
            Self::Symbol { id, .. }
            | Self::Row { id, .. }
            | Self::Fraction { id, .. }
            | Self::Radical { id, .. }
            | Self::Scripts { id, .. }
            | Self::Delimiter { id, .. }
            | Self::Accent { id, .. }
            | Self::BigOp { id, .. }
            | Self::Matrix { id, .. }
            | Self::Aligned { id, .. }
            | Self::Stack { id, .. } => *id,
        }
    }

    /// Spacing class for [`class_spacing_em`] (symbols keep their class; compounds ≈ Ord).
    pub fn spacing_class(&self) -> MathClass {
        match self {
            Self::Symbol { class, .. } => *class,
            Self::Delimiter { .. } => MathClass::Fence,
            Self::BigOp { .. } => MathClass::Operator,
            Self::Row { .. }
            | Self::Fraction { .. }
            | Self::Radical { .. }
            | Self::Scripts { .. }
            | Self::Accent { .. }
            | Self::Matrix { .. }
            | Self::Aligned { .. }
            | Self::Stack { .. } => MathClass::Ordinary,
        }
    }

    pub fn symbol(id: StableNodeId, glyph: impl Into<String>, class: MathClass) -> Self {
        Self::Symbol {
            id,
            glyph: glyph.into(),
            class,
        }
    }

    pub fn row(id: StableNodeId, children: Vec<MathAtom>) -> Self {
        Self::Row { id, children }
    }

    pub fn fraction(id: StableNodeId, numerator: MathAtom, denominator: MathAtom) -> Self {
        Self::Fraction {
            id,
            numerator: Box::new(numerator),
            denominator: Box::new(denominator),
        }
    }

    pub fn radical(id: StableNodeId, radicand: MathAtom) -> Self {
        Self::Radical {
            id,
            index: None,
            radicand: Box::new(radicand),
        }
    }

    pub fn radical_indexed(id: StableNodeId, index: MathAtom, radicand: MathAtom) -> Self {
        Self::Radical {
            id,
            index: Some(Box::new(index)),
            radicand: Box::new(radicand),
        }
    }

    pub fn scripts(
        id: StableNodeId,
        base: MathAtom,
        superscript: Option<MathAtom>,
        subscript: Option<MathAtom>,
    ) -> Self {
        Self::Scripts {
            id,
            base: Box::new(base),
            superscript: superscript.map(Box::new),
            subscript: subscript.map(Box::new),
        }
    }

    pub fn superscript(id: StableNodeId, base: MathAtom, script: MathAtom) -> Self {
        Self::scripts(id, base, Some(script), None)
    }

    pub fn subscript(id: StableNodeId, base: MathAtom, script: MathAtom) -> Self {
        Self::scripts(id, base, None, Some(script))
    }

    pub fn delimiter(
        id: StableNodeId,
        left: impl Into<String>,
        right: impl Into<String>,
        body: MathAtom,
    ) -> Self {
        Self::delimiter_with_stretch(id, left, right, body, 1.0)
    }

    /// Like [`delimiter`](Self::delimiter) with an explicit stretchy scale factor.
    pub fn delimiter_with_stretch(
        id: StableNodeId,
        left: impl Into<String>,
        right: impl Into<String>,
        body: MathAtom,
        stretch_factor: f64,
    ) -> Self {
        Self::Delimiter {
            id,
            left: left.into(),
            right: right.into(),
            body: Box::new(body),
            stretch_factor,
        }
    }

    pub fn paren(id: StableNodeId, body: MathAtom) -> Self {
        Self::delimiter(id, "(", ")", body)
    }

    pub fn accent(id: StableNodeId, kind: MathAccentKind, base: MathAtom) -> Self {
        Self::Accent {
            id,
            kind,
            base: Box::new(base),
        }
    }

    pub fn big_op(
        id: StableNodeId,
        operator: impl Into<String>,
        lower: Option<MathAtom>,
        upper: Option<MathAtom>,
        body: Option<MathAtom>,
    ) -> Self {
        Self::BigOp {
            id,
            operator: operator.into(),
            lower: lower.map(Box::new),
            upper: upper.map(Box::new),
            body: body.map(Box::new),
        }
    }

    pub fn matrix(id: StableNodeId, kind: MathMatrixKind, rows: Vec<Vec<MathAtom>>) -> Self {
        let (left, right) = match kind {
            MathMatrixKind::BMatrix => (Some("[".into()), Some("]".into())),
            MathMatrixKind::PMatrix => (Some("(".into()), Some(")".into())),
            MathMatrixKind::VMatrix => (Some("|".into()), Some("|".into())),
            _ => (None, None),
        };
        Self::Matrix {
            id,
            kind,
            rows,
            left,
            right,
        }
    }

    pub fn matrix_delimited(
        id: StableNodeId,
        left: impl Into<String>,
        right: impl Into<String>,
        rows: Vec<Vec<MathAtom>>,
    ) -> Self {
        Self::Matrix {
            id,
            kind: MathMatrixKind::Delimited,
            rows,
            left: Some(left.into()),
            right: Some(right.into()),
        }
    }

    pub fn aligned(id: StableNodeId, rows: Vec<Vec<MathAtom>>) -> Self {
        Self::Aligned { id, rows }
    }

    pub fn stack(id: StableNodeId, kind: MathStackKind, children: Vec<MathAtom>) -> Self {
        Self::Stack { id, kind, children }
    }

    pub fn atop(id: StableNodeId, top: MathAtom, bottom: MathAtom) -> Self {
        Self::stack(id, MathStackKind::Atop, vec![top, bottom])
    }

    pub fn child_count(&self) -> usize {
        match self {
            Self::Symbol { .. } => 0,
            Self::Row { children, .. } => children.len(),
            Self::Fraction { .. } => 2,
            Self::Radical { index, .. } => 1 + usize::from(index.is_some()),
            Self::Scripts {
                superscript,
                subscript,
                ..
            } => 1 + usize::from(superscript.is_some()) + usize::from(subscript.is_some()),
            Self::Delimiter { .. } => 1,
            Self::Accent { .. } => 1,
            Self::BigOp {
                lower, upper, body, ..
            } => {
                usize::from(lower.is_some())
                    + usize::from(upper.is_some())
                    + usize::from(body.is_some())
            }
            Self::Matrix { rows, .. } | Self::Aligned { rows, .. } => {
                rows.iter().map(|r| r.len()).sum()
            }
            Self::Stack { children, .. } => children.len(),
        }
    }

    /// Linearized debug form (not for layout).
    ///
    /// Multi-character script / limit bodies are wrapped in `{…}` (TeX-style) so
    /// e.g. `a^{12}` is unambiguous versus `a^12`.
    pub fn linearize(&self) -> String {
        match self {
            Self::Symbol { glyph, .. } => glyph.clone(),
            Self::Row { children, .. } => children.iter().map(|c| c.linearize()).collect(),
            Self::Fraction {
                numerator,
                denominator,
                ..
            } => format!("({}/{})", numerator.linearize(), denominator.linearize()),
            Self::Radical {
                index, radicand, ..
            } => match index {
                Some(i) => format!("root[{}]{{{}}}", i.linearize(), radicand.linearize()),
                None => format!("sqrt{{{}}}", radicand.linearize()),
            },
            Self::Scripts {
                base,
                superscript,
                subscript,
                ..
            } => {
                let mut s = base.linearize();
                if let Some(sup) = superscript {
                    s.push('^');
                    s.push_str(&brace_script_body(sup.linearize()));
                }
                if let Some(sub) = subscript {
                    s.push('_');
                    s.push_str(&brace_script_body(sub.linearize()));
                }
                s
            }
            Self::Delimiter {
                left, right, body, ..
            } => format!("{left}{}{right}", body.linearize()),
            Self::Accent { kind, base, .. } => format!("{}{{{}}}", kind.as_str(), base.linearize()),
            Self::BigOp {
                operator,
                lower,
                upper,
                body,
                ..
            } => {
                let mut s = operator.clone();
                if let Some(lo) = lower {
                    s.push('_');
                    s.push_str(&brace_script_body(lo.linearize()));
                }
                if let Some(up) = upper {
                    s.push('^');
                    s.push_str(&brace_script_body(up.linearize()));
                }
                if let Some(b) = body {
                    s.push('{');
                    s.push_str(&b.linearize());
                    s.push('}');
                }
                s
            }
            Self::Matrix {
                kind,
                rows,
                left,
                right,
                ..
            } => {
                let body: Vec<String> = rows
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|c| c.linearize())
                            .collect::<Vec<_>>()
                            .join("&")
                    })
                    .collect();
                let inner = body.join("\\\\");
                match (left.as_deref(), right.as_deref()) {
                    (Some(l), Some(r)) => format!("{l}{inner}{r}"),
                    _ => format!("{}{{{}}}", kind.as_str(), inner),
                }
            }
            Self::Aligned { rows, .. } => {
                let body: Vec<String> = rows
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|c| c.linearize())
                            .collect::<Vec<_>>()
                            .join("&")
                    })
                    .collect();
                format!("align{{{}}}", body.join("\\\\"))
            }
            Self::Stack { kind, children, .. } => {
                let inner: String = children
                    .iter()
                    .map(|c| c.linearize())
                    .collect::<Vec<_>>()
                    .join(",");
                format!("{}{{{}}}", kind.as_str(), inner)
            }
        }
    }
}

/// Wrap multi-character script/limit bodies so linearize stays unambiguous.
fn brace_script_body(s: String) -> String {
    if s.chars().count() == 1 {
        s
    } else {
        format!("{{{s}}}")
    }
}

impl fmt::Display for MathAtom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.linearize())
    }
}

/// Fontless axis-aligned math box (abstract em units).
///
/// `height` is above the baseline; `depth` is below. Not OpenType MATH metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MathBox {
    pub width: f64,
    pub height: f64,
    pub depth: f64,
}

impl MathBox {
    pub const fn new(width: f64, height: f64, depth: f64) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }

    pub fn total_height(self) -> f64 {
        self.height + self.depth
    }

    fn combine_row(parts: &[MathBox]) -> Self {
        let width = parts.iter().map(|b| b.width).sum();
        let height = parts.iter().map(|b| b.height).fold(0.0_f64, f64::max);
        let depth = parts.iter().map(|b| b.depth).fold(0.0_f64, f64::max);
        Self::new(width, height, depth)
    }
}

/// Script-size shrink factor used by `estimate_box` in [`EstimateStyle::Display`]
/// (fontless heuristic).
pub const SCRIPT_SCALE: f64 = 0.7;

/// Tighter script shrink for [`EstimateStyle::Text`] (inline math); smaller than
/// [`SCRIPT_SCALE`]. Heuristic only — not TeX `\textstyle` / OpenType MATH.
pub const SCRIPT_SCALE_TEXT: f64 = 0.5;

/// Fontless math style for [`MathAtom::estimate_box_with_style`].
///
/// [`Display`](EstimateStyle::Display) matches prior `estimate_box` defaults;
/// [`Text`](EstimateStyle::Text) shrinks scripts/limits more (inline-ish stub).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EstimateStyle {
    /// Display / block math (default script scale [`SCRIPT_SCALE`]).
    #[default]
    Display,
    /// Text / inline math (tighter [`SCRIPT_SCALE_TEXT`]).
    Text,
}

impl EstimateStyle {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Display => "display",
            Self::Text => "text",
        }
    }

    /// Parse `"text"` / `"display"` (case-sensitive). Unknown → `None`.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "text" => Some(Self::Text),
            "display" => Some(Self::Display),
            _ => None,
        }
    }

    /// Script / limit shrink factor for this style.
    pub fn script_scale(self) -> f64 {
        match self {
            Self::Display => SCRIPT_SCALE,
            Self::Text => SCRIPT_SCALE_TEXT,
        }
    }
}

impl fmt::Display for EstimateStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Extra height (em) above the base for over-accents (hat/bar/…); stub clearance.
pub const ACCENT_CLEARANCE_EM: f64 = 0.35;

/// Extra depth (em) below the base for compact under accents (underbar).
pub const ACCENT_UNDER_CLEARANCE_EM: f64 = 0.35;

/// Extra height (em) on top of [`ACCENT_CLEARANCE_EM`] for wide over marks.
pub const ACCENT_WIDE_EXTRA_EM: f64 = 0.1;

/// Gap (em) between stackrel / overset / underset bands — fontless stub.
pub const STACKREL_GAP_EM: f64 = 0.15;

/// Extra depth (em) for underbrace-style under marks (package `underbrace` → underline /
/// labeled under → stackrel). Slightly roomier than compact [`ACCENT_UNDER_CLEARANCE_EM`].
pub const UNDERBRACE_CLEARANCE_EM: f64 = 0.5;

/// Fontless under/over accent clearance table: `(above_em, below_em)`.
///
/// Heuristic only — not OpenType MATH accent attachment / TeX `\fontdimen`.
pub fn accent_clearance_em(kind: MathAccentKind) -> (f64, f64) {
    match kind {
        MathAccentKind::Hat
        | MathAccentKind::Bar
        | MathAccentKind::Vec
        | MathAccentKind::Tilde
        | MathAccentKind::Dot
        | MathAccentKind::Ddot
        | MathAccentKind::Check
        | MathAccentKind::Breve
        | MathAccentKind::Acute
        | MathAccentKind::Grave
        | MathAccentKind::Ring => (ACCENT_CLEARANCE_EM, 0.0),
        MathAccentKind::Overline | MathAccentKind::WideHat | MathAccentKind::WideTilde => {
            (ACCENT_CLEARANCE_EM + ACCENT_WIDE_EXTRA_EM, 0.0)
        }
        MathAccentKind::Underline => (0.0, UNDERBRACE_CLEARANCE_EM),
        MathAccentKind::Underbar => (0.0, ACCENT_UNDER_CLEARANCE_EM),
    }
}

/// [`accent_clearance_em`] for a package kind name (`"hat"`, `"check"`, …).
pub fn accent_clearance_em_named(name: &str) -> Option<(f64, f64)> {
    MathAccentKind::from_str_name(name).map(accent_clearance_em)
}

/// Unicode mark glyph used by live layout for an accent kind (heuristic stub).
pub fn accent_mark_glyph(kind: MathAccentKind) -> &'static str {
    match kind {
        MathAccentKind::Hat | MathAccentKind::WideHat => "^",
        MathAccentKind::Bar | MathAccentKind::Overline | MathAccentKind::Underbar => "¯",
        MathAccentKind::Vec => "→",
        MathAccentKind::Tilde | MathAccentKind::WideTilde => "~",
        MathAccentKind::Dot => "˙",
        MathAccentKind::Ddot => "¨",
        MathAccentKind::Check => "ˇ",
        MathAccentKind::Breve => "˘",
        MathAccentKind::Acute => "´",
        MathAccentKind::Grave => "`",
        MathAccentKind::Ring => "˚",
        MathAccentKind::Underline => "_",
    }
}

/// Accent mark attachment offset (em; y upward) from the base origin.
///
/// Centers the mark on the base width and places it in the clearance band
/// from [`accent_clearance_em`]. Heuristic only — not OpenType MATH accent
/// attachment.
pub fn accent_attachment_offset(kind: MathAccentKind, base: MathBox) -> (f64, f64) {
    let (above, below) = accent_clearance_em(kind);
    let mark_w = 0.8_f64;
    let dx = (base.width - mark_w) * 0.5;
    if above > 0.0 {
        (dx, base.height + above * 0.5)
    } else {
        (dx, -(base.depth + below * 0.5))
    }
}

/// Fraction rule (vinculum) thickness in em — fontless stub, not TeX `\fontdimen8`.
pub const FRAC_RULE_THICKNESS_EM: f64 = 0.04;

/// Clearance (em) between numerator box and the fraction rule.
pub const FRAC_NUM_CLEARANCE_EM: f64 = 0.15;

/// Clearance (em) between the fraction rule and denominator box.
pub const FRAC_DEN_CLEARANCE_EM: f64 = 0.15;

/// Radical over-bar (vinculum) thickness in em — fontless stub.
pub const RADICAL_VINCULUM_THICKNESS_EM: f64 = 0.04;

/// Clearance (em) between radicand and the vinculum.
pub const RADICAL_VINCULUM_CLEARANCE_EM: f64 = 0.25;

/// Horizontal pad (em) for the radical surd / index gutter.
pub const RADICAL_SURD_PAD_EM: f64 = 0.55;

/// Fraction rule thickness + num/den clearance (em). Heuristic only.
pub fn fraction_rule_metrics() -> (f64, f64, f64) {
    (
        FRAC_RULE_THICKNESS_EM,
        FRAC_NUM_CLEARANCE_EM,
        FRAC_DEN_CLEARANCE_EM,
    )
}

/// Radical vinculum / index placement stub relative to the radicand box origin.
///
/// Returns `(vinculum_y, index_x, index_y)` with `x` rightward and `y` upward from
/// the radicand baseline. Missing index yields `0.0` for the index pair.
/// Heuristic only — not OpenType MATH radical metrics / TeX `\fontdimen`.
pub fn radical_vinculum_index_offsets(
    radicand: MathBox,
    index: Option<MathBox>,
) -> (f64, f64, f64) {
    let vinculum_y = radicand.height + RADICAL_VINCULUM_CLEARANCE_EM;
    let (index_x, index_y) = match index {
        Some(idx) => {
            let scaled_w = idx.width * SCRIPT_SCALE;
            let index_x = -(scaled_w * 0.55 + RADICAL_SURD_PAD_EM * 0.15);
            let index_y = radicand.height * 0.55 + idx.depth * SCRIPT_SCALE * 0.25;
            (index_x, index_y)
        }
        None => (0.0, 0.0),
    };
    (vinculum_y, index_x, index_y)
}

/// Fontless script attachment offsets relative to the base box origin.
///
/// Returns `(sub_x, sub_y, sup_x, sup_y)` where `x` grows rightward and `y` grows
/// upward from the baseline. Missing scripts yield `0.0` for that pair.
/// Heuristic only — not OpenType MATH / TeX `\fontdimen` attachment.
pub fn scripts_attachment_offsets(
    base: MathBox,
    sub: Option<MathBox>,
    sup: Option<MathBox>,
) -> (f64, f64, f64, f64) {
    let sub_x = if sub.is_some() { base.width } else { 0.0 };
    let sub_y = sub
        .map(|s| -(base.depth * 0.35 + s.height * SCRIPT_SCALE * 0.5))
        .unwrap_or(0.0);
    let sup_x = if sup.is_some() { base.width } else { 0.0 };
    let sup_y = sup
        .map(|s| base.height * 0.55 + s.depth * SCRIPT_SCALE * 0.25)
        .unwrap_or(0.0);
    (sub_x, sub_y, sup_x, sup_y)
}

/// Display-style big-op limit offsets relative to the operator box origin.
///
/// Returns `(lower_x, lower_y, upper_x, upper_y)` with `x` rightward and `y`
/// upward from the operator baseline. Limits are horizontally centered on the
/// operator; missing limits yield `0.0` for that pair.
/// Heuristic only — not OpenType MATH / TeX `\displaylimits` metrics.
pub fn bigop_limit_offsets(
    op_box: MathBox,
    lower: Option<MathBox>,
    upper: Option<MathBox>,
) -> (f64, f64, f64, f64) {
    let lower_x = lower
        .map(|lo| (op_box.width - lo.width * SCRIPT_SCALE) * 0.5)
        .unwrap_or(0.0);
    let lower_y = lower
        .map(|lo| -(op_box.depth + lo.height * SCRIPT_SCALE + 0.1))
        .unwrap_or(0.0);
    let upper_x = upper
        .map(|up| (op_box.width - up.width * SCRIPT_SCALE) * 0.5)
        .unwrap_or(0.0);
    let upper_y = upper
        .map(|up| op_box.height + up.depth * SCRIPT_SCALE + 0.1)
        .unwrap_or(0.0);
    (lower_x, lower_y, upper_x, upper_y)
}

/// Stackrel / overset / underset vertical placement stub.
///
/// Returns `(upper_y, lower_y)` with `y` upward from a shared baseline between
/// the two bands, separated by [`STACKREL_GAP_EM`]. Heuristic only — not TeX
/// `\stackrel` / `\overset` metrics.
pub fn stackrel_spacing_offsets(upper: MathBox, lower: MathBox) -> (f64, f64) {
    let upper_y = STACKREL_GAP_EM * 0.5 + upper.depth;
    let lower_y = -(STACKREL_GAP_EM * 0.5 + lower.height);
    (upper_y, lower_y)
}

/// Underbrace-style clearance below the body (em). Heuristic only.
pub fn underbrace_spacing() -> f64 {
    UNDERBRACE_CLEARANCE_EM
}

/// TeX-ish `\vphantom`: keep height/depth, **zero width**.
///
/// Fontless estimate stub for invisible vertical reserve — not a `MathAtom`
/// variant / OpenType MATH phantom node.
pub fn phantom_box(inner: MathBox) -> MathBox {
    MathBox::new(0.0, inner.height, inner.depth)
}

/// TeX-ish `\smash`: keep width, **zero height and depth**.
///
/// Fontless estimate stub so neighbors pack horizontally without vertical
/// contribution — not a `MathAtom` variant / real smash glyph.
pub fn smash_box(inner: MathBox) -> MathBox {
    MathBox::new(inner.width, 0.0, 0.0)
}

/// Horizontal packing of a cell inside its column (fontless matrix stub).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixColumnAlign {
    Left,
    Center,
    Right,
}

/// Max cell width per column for matrix / aligned / cases grids.
///
/// Used by [`MathAtom::estimate_box`] for [`MathAtom::Matrix`] and
/// [`MathAtom::Aligned`]. Not real TeX column templates / `array` `*{c}`.
pub fn matrix_column_widths(rows: &[Vec<MathAtom>]) -> Vec<f64> {
    let mut col_widths: Vec<f64> = Vec::new();
    for row in rows {
        for (ci, cell) in row.iter().enumerate() {
            let w = cell.estimate_box().width;
            if col_widths.len() <= ci {
                col_widths.resize(ci + 1, 0.0);
            }
            col_widths[ci] = col_widths[ci].max(w);
        }
    }
    col_widths
}

/// Cases / piecewise columns are left-aligned (TeX-like heuristic stub).
pub fn cases_column_align(_column: usize) -> MatrixColumnAlign {
    MatrixColumnAlign::Left
}

/// Nominal per-row height (em) for cases left-brace stretch when cells are empty/thin.
pub const CASES_ROW_HEIGHT_EM: f64 = 1.0;

/// Total vertical extent (height+depth) for a cases / choice left brace.
///
/// Stretchy-delimiter-inspired stub: `nrows.max(1) × row_height`. Used by
/// [`MathAtom::estimate_box`] for delimited matrices (`math-cases`). Not
/// OpenType MATH stretchy fences.
pub fn cases_brace_total_height_em(nrows: usize, row_height: f64) -> f64 {
    let n = nrows.max(1) as f64;
    n * row_height.max(0.0)
}

/// X offset of a cell within its column band (`0` = left edge of the column).
pub fn matrix_cell_x_in_column(cell_width: f64, col_width: f64, align: MatrixColumnAlign) -> f64 {
    let slack = (col_width - cell_width).max(0.0);
    match align {
        MatrixColumnAlign::Left => 0.0,
        MatrixColumnAlign::Center => slack * 0.5,
        MatrixColumnAlign::Right => slack,
    }
}

/// Inter-column gutter (em) used by grid `estimate_box` and [`aligned_column_x`].
pub const ALIGNED_COLUMN_GUTTER_EM: f64 = 0.25;

/// Absolute x of the left edge of column `col` in an aligned / matrix grid.
///
/// Sums prior [`matrix_column_widths`] plus [`ALIGNED_COLUMN_GUTTER_EM`] gutters.
/// Out-of-range `col` snaps past the last column band (same gutters as
/// `estimate_grid_box`). Heuristic only — not TeX `align` `&` tab stops.
pub fn aligned_column_x(rows: &[Vec<MathAtom>], col: usize) -> f64 {
    let widths = matrix_column_widths(rows);
    if widths.is_empty() {
        return 0.0;
    }
    let mut x = 0.0_f64;
    for w in widths.iter().take(col.min(widths.len())) {
        x += w + ALIGNED_COLUMN_GUTTER_EM;
    }
    x
}

impl MathAtom {
    /// Rough width/height/depth estimate without fonts (layout scaffolding only).
    ///
    /// Defaults to [`EstimateStyle::Display`] (same metrics as before Wave 17).
    pub fn estimate_box(&self) -> MathBox {
        self.estimate_box_with_style(EstimateStyle::Display)
    }

    /// Like [`estimate_box`](Self::estimate_box), with display vs text script scale.
    ///
    /// [`EstimateStyle::Text`] uses [`SCRIPT_SCALE_TEXT`] for scripts / radical
    /// index / big-op limits (shrinks more than display). Recursive children keep
    /// the same style. Heuristic only — not TeX style / OpenType MATH.
    pub fn estimate_box_with_style(&self, style: EstimateStyle) -> MathBox {
        let script_scale = style.script_scale();
        match self {
            Self::Symbol { glyph, .. } => {
                let w = (glyph.chars().count() as f64).max(0.5);
                MathBox::new(w, 0.7, 0.2)
            }
            Self::Row { children, .. } => {
                let boxes: Vec<_> = children
                    .iter()
                    .map(|c| c.estimate_box_with_style(style))
                    .collect();
                let mut row = MathBox::combine_row(&boxes);
                if children.len() >= 2 {
                    let mut gap = 0.0_f64;
                    for i in 0..children.len() - 1 {
                        gap += class_spacing_em(
                            children[i].spacing_class(),
                            children[i + 1].spacing_class(),
                        );
                    }
                    row.width += gap;
                }
                row
            }
            Self::Fraction {
                numerator,
                denominator,
                ..
            } => {
                let num = numerator.estimate_box_with_style(style);
                let den = denominator.estimate_box_with_style(style);
                let width = num.width.max(den.width) + 0.2;
                let (rule, num_clr, den_clr) = fraction_rule_metrics();
                // Stack: num + clearance + rule above baseline; den + clearance below.
                MathBox::new(
                    width,
                    num.total_height() + num_clr + rule,
                    den.total_height() + den_clr,
                )
            }
            Self::Radical {
                index, radicand, ..
            } => {
                let body = radicand.estimate_box_with_style(style);
                let mut width = body.width + RADICAL_SURD_PAD_EM;
                let mut height =
                    body.height + RADICAL_VINCULUM_CLEARANCE_EM + RADICAL_VINCULUM_THICKNESS_EM;
                let depth = body.depth;
                if let Some(idx) = index {
                    let ib = idx.estimate_box_with_style(style);
                    let scaled_w = ib.width * script_scale;
                    width += scaled_w * 0.5;
                    let (_vy, _ix, iy) = radical_vinculum_index_offsets(body, Some(ib));
                    height = height.max(iy + ib.height * script_scale);
                }
                MathBox::new(width, height, depth)
            }
            Self::Scripts {
                base,
                superscript,
                subscript,
                ..
            } => {
                let b = base.estimate_box_with_style(style);
                let mut width = b.width;
                let mut height = b.height;
                let mut depth = b.depth;
                let mut script_w = 0.0_f64;
                if let Some(sup) = superscript {
                    let s = sup.estimate_box_with_style(style);
                    script_w = script_w.max(s.width * script_scale);
                    height = height.max(b.height * 0.5 + s.height * script_scale);
                }
                if let Some(sub) = subscript {
                    let s = sub.estimate_box_with_style(style);
                    script_w = script_w.max(s.width * script_scale);
                    depth = depth.max(b.depth * 0.5 + s.depth * script_scale + 0.15);
                }
                width += script_w;
                MathBox::new(width, height, depth)
            }
            Self::Delimiter {
                left,
                right,
                body,
                stretch_factor,
                ..
            } => {
                let inner = body.estimate_box_with_style(style);
                let pad = 0.35 * (left.chars().count() + right.chars().count()) as f64;
                // Stretchy heuristic: fence height/depth track body, scaled by stretch_factor.
                let factor = stretch_factor.max(0.0);
                let height = (inner.height * factor).max(0.9);
                let depth = (inner.depth * factor).max(0.3);
                MathBox::new(inner.width + pad, height, depth)
            }
            Self::Accent { kind, base, .. } => {
                let b = base.estimate_box_with_style(style);
                let width = b.width.max(0.8);
                let (above, below) = accent_clearance_em(*kind);
                MathBox::new(width, b.height + above, b.depth + below)
            }
            Self::BigOp {
                operator,
                lower,
                upper,
                body,
                ..
            } => {
                let op_w = (operator.chars().count() as f64).max(1.0);
                let mut width = op_w;
                let mut height = 0.9;
                let mut depth = 0.3;
                if let Some(lo) = lower {
                    let lb = lo.estimate_box_with_style(style);
                    width = width.max(lb.width * script_scale);
                    depth += lb.total_height() * script_scale;
                }
                if let Some(up) = upper {
                    let ub = up.estimate_box_with_style(style);
                    width = width.max(ub.width * script_scale);
                    height += ub.total_height() * script_scale;
                }
                if let Some(b) = body {
                    let bb = b.estimate_box_with_style(style);
                    width += bb.width + 0.2;
                    height = height.max(bb.height);
                    depth = depth.max(bb.depth);
                }
                MathBox::new(width, height, depth)
            }
            Self::Matrix {
                rows, left, right, ..
            } => estimate_grid_box(rows, left.as_deref(), right.as_deref(), style),
            Self::Aligned { rows, .. } => estimate_grid_box(rows, None, None, style),
            Self::Stack { kind, children, .. } => {
                let boxes: Vec<_> = children
                    .iter()
                    .map(|c| c.estimate_box_with_style(style))
                    .collect();
                let width = boxes.iter().map(|b| b.width).fold(0.0_f64, f64::max);
                if *kind == MathStackKind::Stackrel && boxes.len() >= 2 {
                    let upper = boxes[0];
                    let lower = boxes[1];
                    let (uy, ly) = stackrel_spacing_offsets(upper, lower);
                    let height = uy + upper.height;
                    let depth = (-ly) + lower.depth;
                    MathBox::new(width, height, depth)
                } else {
                    let gap = match kind {
                        MathStackKind::Stackrel => STACKREL_GAP_EM,
                        _ => 0.1,
                    };
                    let total: f64 = boxes.iter().map(|b| b.total_height() + gap).sum();
                    MathBox::new(width, total * 0.55, total * 0.45)
                }
            }
        }
    }
}

fn estimate_grid_box(
    rows: &[Vec<MathAtom>],
    left: Option<&str>,
    right: Option<&str>,
    style: EstimateStyle,
) -> MathBox {
    let mut col_widths: Vec<f64> = Vec::new();
    let mut total_h = 0.0_f64;
    let mut max_row_h = 0.0_f64;
    for row in rows {
        let mut row_h = 0.0_f64;
        for (ci, cell) in row.iter().enumerate() {
            let b = cell.estimate_box_with_style(style);
            if col_widths.len() <= ci {
                col_widths.resize(ci + 1, 0.0);
            }
            col_widths[ci] = col_widths[ci].max(b.width);
            row_h = row_h.max(b.total_height());
        }
        max_row_h = max_row_h.max(row_h);
        total_h += row_h + 0.2;
    }
    let mut width: f64 =
        col_widths.iter().sum::<f64>() + ALIGNED_COLUMN_GUTTER_EM * col_widths.len() as f64;
    if let Some(l) = left {
        width += 0.35 * l.chars().count() as f64;
    }
    if let Some(r) = right {
        width += 0.35 * r.chars().count() as f64;
    }
    // Content-based height/depth split (prior grid stub for undelimited matrices).
    let mut height = total_h * 0.55;
    let mut depth = total_h * 0.45;
    // Cases / choice left brace: stretchy-delimiter heuristic —
    // brace total height = rows × row_height (measured max row, floor CASES_ROW_HEIGHT_EM).
    // Replaces the content+gap split so the fence tracks row count explicitly.
    if left.is_some() {
        let row_h = max_row_h.max(CASES_ROW_HEIGHT_EM);
        let brace_total = cases_brace_total_height_em(rows.len(), row_h);
        height = brace_total * 0.55;
        depth = brace_total * 0.45;
    }
    MathBox::new(width.max(0.5), height, depth)
}

/// Millimeters per em for naive math → Text glyph placement (matches doc paragraph size).
pub const MATH_LAYOUT_EM_TO_MM: f64 = 4.0;

/// Convert math-space offset (em; **y upward** from baseline) to scene mm
/// (y downward). `origin` is the baseline point in scene mm.
fn math_offset_to_scene_mm(origin: (f64, f64), dx_em: f64, dy_em: f64) -> (f64, f64) {
    (
        origin.0 + dx_em * MATH_LAYOUT_EM_TO_MM,
        origin.1 - dy_em * MATH_LAYOUT_EM_TO_MM,
    )
}

/// Very naive math atom → scene Text glyphs (and occasional Line rules).
///
/// Default: one [`reciplexa_scene::Text`] per Unicode scalar in
/// [`MathAtom::linearize`], advancing by `estimate_box().width / n`
/// (monospace heuristic).
///
/// [`MathAtom::Scripts`]: base at `origin`, sub/sup placed via
/// [`scripts_attachment_offsets`] (LL7).
/// [`MathAtom::BigOp`]: operator at `origin`, limits via
/// [`bigop_limit_offsets`] (LL8).
/// [`MathAtom::Fraction`]: num/den stacked with a [`reciplexa_scene::Line`]
/// rule between them (LL9).
/// [`MathAtom::Radical`]: radicand (+ optional index) with vinculum
/// [`reciplexa_scene::Line`] via [`radical_vinculum_index_offsets`] (LL11).
/// [`MathAtom::Delimiter`]: left/right fence Text glyphs with taller
/// `size_mm` from the stretchy height/depth heuristic (LL13).
/// [`MathAtom::Matrix`]: cells placed via [`matrix_column_widths`] /
/// [`aligned_column_x`] / [`matrix_cell_x_in_column`] (LL14).
/// [`MathAtom::Accent`]: accent mark glyph via [`accent_clearance_em`] (LL15).
/// [`MathAtom::Aligned`]: cells snapped to [`aligned_column_x`] bands (LL18).
/// Not OpenType MATH / glyph metrics.
pub fn layout_math_atom_to_shapes(
    atom: &MathAtom,
    origin: (f64, f64),
) -> Vec<reciplexa_scene::Shape> {
    match atom {
        MathAtom::Aligned { rows, .. } => {
            layout_math_grid_to_shapes(rows, origin, None, None, /* left_align */ true)
        }
        MathAtom::Accent { kind, base, .. } => {
            let base_box = base.estimate_box();
            let (dx, dy) = accent_attachment_offset(*kind, base_box);
            let mut shapes = layout_math_atom_to_shapes(base, origin);
            let mark = accent_mark_glyph(*kind);
            let mark_origin = math_offset_to_scene_mm(origin, dx, dy);
            let size_mm = MATH_LAYOUT_EM_TO_MM;
            let width_mm = 0.8 * MATH_LAYOUT_EM_TO_MM;
            shapes.push(reciplexa_scene::Shape::Text(reciplexa_scene::Text {
                x_mm: mark_origin.0,
                y_mm: mark_origin.1,
                size_mm,
                width_mm: Some(width_mm),
                height_mm: Some(size_mm),
                content: mark.to_string(),
                fill: reciplexa_scene::Color::BLACK,
            }));
            shapes
        }
        MathAtom::Matrix {
            rows, left, right, ..
        } => layout_math_grid_to_shapes(
            rows,
            origin,
            left.as_deref(),
            right.as_deref(),
            /* cases-style left brace ⇒ left-align cells */ left.is_some(),
        ),
        MathAtom::Delimiter {
            left,
            right,
            body,
            stretch_factor,
            ..
        } => {
            let inner = body.estimate_box();
            let (fence_h, fence_d) = delimiter_fence_extent_em(inner, *stretch_factor);
            let fence_size_em = fence_h + fence_d;
            let left_w = delimiter_fence_pad_em(left);
            let right_w = delimiter_fence_pad_em(right);
            let mut shapes = layout_fence_chars(left, origin, fence_size_em, left_w);
            let body_origin = math_offset_to_scene_mm(origin, left_w, 0.0);
            shapes.extend(layout_math_atom_to_shapes(body, body_origin));
            let right_origin = math_offset_to_scene_mm(origin, left_w + inner.width, 0.0);
            shapes.extend(layout_fence_chars(
                right,
                right_origin,
                fence_size_em,
                right_w,
            ));
            shapes
        }
        MathAtom::Scripts {
            base,
            superscript,
            subscript,
            ..
        } => {
            let base_box = base.estimate_box();
            let sub_box = subscript.as_ref().map(|s| s.estimate_box());
            let sup_box = superscript.as_ref().map(|s| s.estimate_box());
            let (sub_x, sub_y, sup_x, sup_y) =
                scripts_attachment_offsets(base_box, sub_box, sup_box);
            let mut shapes = layout_math_atom_to_shapes(base, origin);
            if let Some(sub) = subscript {
                let o = math_offset_to_scene_mm(origin, sub_x, sub_y);
                shapes.extend(layout_math_atom_to_shapes(sub, o));
            }
            if let Some(sup) = superscript {
                let o = math_offset_to_scene_mm(origin, sup_x, sup_y);
                shapes.extend(layout_math_atom_to_shapes(sup, o));
            }
            shapes
        }
        MathAtom::BigOp {
            operator,
            lower,
            upper,
            body,
            id,
            ..
        } => {
            let op_atom = MathAtom::symbol(*id, operator.clone(), MathClass::Operator);
            let op_box = op_atom.estimate_box();
            let lower_box = lower.as_ref().map(|a| a.estimate_box());
            let upper_box = upper.as_ref().map(|a| a.estimate_box());
            let (lx, ly, ux, uy) = bigop_limit_offsets(op_box, lower_box, upper_box);
            let mut shapes = layout_math_atom_to_shapes(&op_atom, origin);
            if let Some(lo) = lower {
                let o = math_offset_to_scene_mm(origin, lx, ly);
                shapes.extend(layout_math_atom_to_shapes(lo, o));
            }
            if let Some(up) = upper {
                let o = math_offset_to_scene_mm(origin, ux, uy);
                shapes.extend(layout_math_atom_to_shapes(up, o));
            }
            if let Some(b) = body {
                // Body sits to the right of the operator (display-style stub).
                let body_origin = (
                    origin.0 + (op_box.width + 0.2) * MATH_LAYOUT_EM_TO_MM,
                    origin.1,
                );
                shapes.extend(layout_math_atom_to_shapes(b, body_origin));
            }
            shapes
        }
        MathAtom::Fraction {
            numerator,
            denominator,
            ..
        } => {
            let num_box = numerator.estimate_box();
            let den_box = denominator.estimate_box();
            let frac_box = atom.estimate_box();
            let (rule_em, num_clr, den_clr) = fraction_rule_metrics();
            let width = frac_box.width;
            let num_dx = (width - num_box.width) * 0.5;
            let den_dx = (width - den_box.width) * 0.5;
            // Stack: rule on baseline; num above (rule + clearance + num.depth);
            // den below (clearance + den.height). Math y-up → scene y-down.
            let num_dy = rule_em + num_clr + num_box.depth;
            let den_dy = -(den_clr + den_box.height);
            let mut shapes = Vec::new();
            shapes.extend(layout_math_atom_to_shapes(
                numerator,
                math_offset_to_scene_mm(origin, num_dx, num_dy),
            ));
            let (ox, oy) = origin;
            let rule_w_mm = (rule_em * MATH_LAYOUT_EM_TO_MM).max(0.15);
            shapes.push(reciplexa_scene::Shape::Line(reciplexa_scene::Line {
                x1_mm: ox,
                y1_mm: oy,
                x2_mm: ox + width * MATH_LAYOUT_EM_TO_MM,
                y2_mm: oy,
                stroke: reciplexa_scene::Color::BLACK,
                width_mm: rule_w_mm,
            }));
            shapes.extend(layout_math_atom_to_shapes(
                denominator,
                math_offset_to_scene_mm(origin, den_dx, den_dy),
            ));
            shapes
        }
        MathAtom::Radical {
            index, radicand, ..
        } => {
            let body = radicand.estimate_box();
            let idx_box = index.as_ref().map(|i| i.estimate_box());
            let (vinculum_y, index_x, index_y) = radical_vinculum_index_offsets(body, idx_box);
            // Radicand origin shifted right by surd pad (heuristic gutter).
            let rad_origin = math_offset_to_scene_mm(origin, RADICAL_SURD_PAD_EM, 0.0);
            let mut shapes = layout_math_atom_to_shapes(radicand, rad_origin);
            if let Some(idx) = index {
                let o = math_offset_to_scene_mm(origin, index_x, index_y);
                shapes.extend(layout_math_atom_to_shapes(idx, o));
            }
            let (ox, oy) = rad_origin;
            let bar_y = oy - vinculum_y * MATH_LAYOUT_EM_TO_MM;
            let bar_w_mm = (RADICAL_VINCULUM_THICKNESS_EM * MATH_LAYOUT_EM_TO_MM).max(0.15);
            shapes.push(reciplexa_scene::Shape::Line(reciplexa_scene::Line {
                x1_mm: ox,
                y1_mm: bar_y,
                x2_mm: ox + body.width * MATH_LAYOUT_EM_TO_MM,
                y2_mm: bar_y,
                stroke: reciplexa_scene::Color::BLACK,
                width_mm: bar_w_mm,
            }));
            shapes
        }
        _ => layout_math_linearize_glyphs(atom, origin),
    }
}

/// Stretchy fence height/depth (em) matching [`MathAtom::estimate_box`] for
/// [`MathAtom::Delimiter`].
pub fn delimiter_fence_extent_em(inner: MathBox, stretch_factor: f64) -> (f64, f64) {
    let factor = stretch_factor.max(0.0);
    let height = (inner.height * factor).max(0.9);
    let depth = (inner.depth * factor).max(0.3);
    (height, depth)
}

fn delimiter_fence_pad_em(fence: &str) -> f64 {
    0.35 * fence.chars().count().max(1) as f64
}

/// Place fence glyph(s) with an explicit em size (taller stretchy delimiters).
fn layout_fence_chars(
    fence: &str,
    origin: (f64, f64),
    size_em: f64,
    total_width_em: f64,
) -> Vec<reciplexa_scene::Shape> {
    let (ox, oy) = origin;
    let n = fence.chars().count().max(1) as f64;
    let size_mm = size_em.max(0.5) * MATH_LAYOUT_EM_TO_MM;
    let advance = (total_width_em.max(0.35) * MATH_LAYOUT_EM_TO_MM) / n;
    fence
        .chars()
        .enumerate()
        .map(|(i, ch)| {
            reciplexa_scene::Shape::Text(reciplexa_scene::Text {
                x_mm: ox + i as f64 * advance,
                y_mm: oy,
                size_mm,
                width_mm: Some(advance),
                height_mm: Some(size_mm),
                content: ch.to_string(),
                fill: reciplexa_scene::Color::BLACK,
            })
        })
        .collect()
}

/// Matrix / cases grid → scene shapes using column-width bands (LL14).
///
/// When `left_align` is true (delimited / cases), cells use
/// [`cases_column_align`]; otherwise columns are centered.
fn layout_math_grid_to_shapes(
    rows: &[Vec<MathAtom>],
    origin: (f64, f64),
    left: Option<&str>,
    right: Option<&str>,
    left_align: bool,
) -> Vec<reciplexa_scene::Shape> {
    let col_widths = matrix_column_widths(rows);
    let row_heights: Vec<f64> = rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|c| c.estimate_box().total_height())
                .fold(0.0_f64, f64::max)
                .max(0.5)
        })
        .collect();
    let gaps = 0.2_f64;
    let content_h: f64 = if row_heights.is_empty() {
        0.5
    } else {
        row_heights.iter().sum::<f64>() + gaps * (row_heights.len().saturating_sub(1) as f64)
    };
    let left_pad = left.map(delimiter_fence_pad_em).unwrap_or(0.0);
    let right_pad = right.map(delimiter_fence_pad_em).unwrap_or(0.0);
    let grid_w: f64 = if col_widths.is_empty() {
        0.0
    } else {
        col_widths.iter().sum::<f64>() + ALIGNED_COLUMN_GUTTER_EM * col_widths.len() as f64
    };

    let mut shapes = Vec::new();
    if let Some(l) = left {
        let max_row = row_heights
            .iter()
            .copied()
            .fold(0.0_f64, f64::max)
            .max(CASES_ROW_HEIGHT_EM);
        let brace_total = cases_brace_total_height_em(rows.len(), max_row);
        shapes.extend(layout_fence_chars(l, origin, brace_total, left_pad));
    }

    // Top of first row sits `content_h * 0.55` above the shared baseline (matches
    // undelimited grid estimate height/depth split).
    let mut y_cursor = content_h * 0.55;
    for (ri, row) in rows.iter().enumerate() {
        let rh = row_heights.get(ri).copied().unwrap_or(0.5);
        let row_baseline = y_cursor - rh * 0.55;
        for (ci, cell) in row.iter().enumerate() {
            let cell_box = cell.estimate_box();
            let col_w = col_widths.get(ci).copied().unwrap_or(cell_box.width);
            let align = if left_align {
                cases_column_align(ci)
            } else {
                MatrixColumnAlign::Center
            };
            let dx = left_pad
                + aligned_column_x(rows, ci)
                + matrix_cell_x_in_column(cell_box.width, col_w, align);
            let o = math_offset_to_scene_mm(origin, dx, row_baseline);
            shapes.extend(layout_math_atom_to_shapes(cell, o));
        }
        y_cursor -= rh + gaps;
    }

    if let Some(r) = right {
        let max_row = row_heights
            .iter()
            .copied()
            .fold(0.0_f64, f64::max)
            .max(CASES_ROW_HEIGHT_EM);
        let brace_total = cases_brace_total_height_em(rows.len(), max_row);
        let right_origin = math_offset_to_scene_mm(origin, left_pad + grid_w, 0.0);
        shapes.extend(layout_fence_chars(r, right_origin, brace_total, right_pad));
    }
    shapes
}

fn layout_math_linearize_glyphs(
    atom: &MathAtom,
    origin: (f64, f64),
) -> Vec<reciplexa_scene::Shape> {
    let content = atom.linearize();
    let mbox = atom.estimate_box();
    let (ox, oy) = origin;
    let size_mm = MATH_LAYOUT_EM_TO_MM;
    let n = content.chars().count().max(1) as f64;
    let total_w_mm = mbox.width.max(0.5) * MATH_LAYOUT_EM_TO_MM;
    let advance = total_w_mm / n;
    let height_mm = Some(mbox.total_height().max(0.5) * MATH_LAYOUT_EM_TO_MM);
    content
        .chars()
        .enumerate()
        .map(|(i, ch)| {
            reciplexa_scene::Shape::Text(reciplexa_scene::Text {
                x_mm: ox + i as f64 * advance,
                y_mm: oy,
                size_mm,
                width_mm: Some(advance),
                height_mm,
                content: ch.to_string(),
                fill: reciplexa_scene::Color::BLACK,
            })
        })
        .collect()
}
