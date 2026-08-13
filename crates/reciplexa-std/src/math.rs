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

/// Accent kind for `MathAtom::Accent` (package `math/accents`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathAccentKind {
    Hat,
    Bar,
    Vec,
    Tilde,
    Dot,
    Ddot,
    Overline,
    Underline,
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
            Self::Overline => "overline",
            Self::Underline => "underline",
            Self::WideHat => "widehat",
            Self::WideTilde => "widetilde",
        }
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

/// Script-size shrink factor used by `estimate_box` (fontless heuristic).
pub const SCRIPT_SCALE: f64 = 0.7;

/// Extra height (em) above the base for over-accents (hat/bar/…); stub clearance.
pub const ACCENT_CLEARANCE_EM: f64 = 0.35;

/// Extra depth (em) below the base for underline-style accents.
pub const ACCENT_UNDER_CLEARANCE_EM: f64 = 0.35;

impl MathAtom {
    /// Rough width/height/depth estimate without fonts (layout scaffolding only).
    pub fn estimate_box(&self) -> MathBox {
        match self {
            Self::Symbol { glyph, .. } => {
                let w = (glyph.chars().count() as f64).max(0.5);
                MathBox::new(w, 0.7, 0.2)
            }
            Self::Row { children, .. } => {
                let boxes: Vec<_> = children.iter().map(|c| c.estimate_box()).collect();
                MathBox::combine_row(&boxes)
            }
            Self::Fraction {
                numerator,
                denominator,
                ..
            } => {
                let num = numerator.estimate_box();
                let den = denominator.estimate_box();
                let width = num.width.max(den.width) + 0.2;
                // Stack: num above rule, den below; baseline at rule.
                MathBox::new(width, num.total_height() + 0.15, den.total_height() + 0.15)
            }
            Self::Radical {
                index, radicand, ..
            } => {
                let body = radicand.estimate_box();
                let mut width = body.width + 0.55;
                let mut height = body.height + 0.25;
                let depth = body.depth;
                if let Some(idx) = index {
                    let ib = idx.estimate_box();
                    let scaled_w = ib.width * SCRIPT_SCALE;
                    width += scaled_w * 0.5;
                    height = height.max(ib.height * SCRIPT_SCALE + 0.1);
                }
                MathBox::new(width, height, depth)
            }
            Self::Scripts {
                base,
                superscript,
                subscript,
                ..
            } => {
                let b = base.estimate_box();
                let mut width = b.width;
                let mut height = b.height;
                let mut depth = b.depth;
                let mut script_w = 0.0_f64;
                if let Some(sup) = superscript {
                    let s = sup.estimate_box();
                    script_w = script_w.max(s.width * SCRIPT_SCALE);
                    height = height.max(b.height * 0.5 + s.height * SCRIPT_SCALE);
                }
                if let Some(sub) = subscript {
                    let s = sub.estimate_box();
                    script_w = script_w.max(s.width * SCRIPT_SCALE);
                    depth = depth.max(b.depth * 0.5 + s.depth * SCRIPT_SCALE + 0.15);
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
                let inner = body.estimate_box();
                let pad = 0.35 * (left.chars().count() + right.chars().count()) as f64;
                // Stretchy heuristic: fence height/depth track body, scaled by stretch_factor.
                let factor = stretch_factor.max(0.0);
                let height = (inner.height * factor).max(0.9);
                let depth = (inner.depth * factor).max(0.3);
                MathBox::new(inner.width + pad, height, depth)
            }
            Self::Accent { kind, base, .. } => {
                let b = base.estimate_box();
                let width = b.width.max(0.8);
                match kind {
                    MathAccentKind::Underline => {
                        MathBox::new(width, b.height, b.depth + ACCENT_UNDER_CLEARANCE_EM)
                    }
                    MathAccentKind::Overline
                    | MathAccentKind::WideHat
                    | MathAccentKind::WideTilde => {
                        // Wide marks get a touch more clearance than compact accents.
                        MathBox::new(width, b.height + ACCENT_CLEARANCE_EM + 0.1, b.depth)
                    }
                    _ => MathBox::new(width, b.height + ACCENT_CLEARANCE_EM, b.depth),
                }
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
                    let lb = lo.estimate_box();
                    width = width.max(lb.width * SCRIPT_SCALE);
                    depth += lb.total_height() * SCRIPT_SCALE;
                }
                if let Some(up) = upper {
                    let ub = up.estimate_box();
                    width = width.max(ub.width * SCRIPT_SCALE);
                    height += ub.total_height() * SCRIPT_SCALE;
                }
                if let Some(b) = body {
                    let bb = b.estimate_box();
                    width += bb.width + 0.2;
                    height = height.max(bb.height);
                    depth = depth.max(bb.depth);
                }
                MathBox::new(width, height, depth)
            }
            Self::Matrix {
                rows, left, right, ..
            } => estimate_grid_box(rows, left.as_deref(), right.as_deref()),
            Self::Aligned { rows, .. } => estimate_grid_box(rows, None, None),
            Self::Stack { children, .. } => {
                let boxes: Vec<_> = children.iter().map(|c| c.estimate_box()).collect();
                let width = boxes.iter().map(|b| b.width).fold(0.0_f64, f64::max);
                let total: f64 = boxes.iter().map(|b| b.total_height() + 0.1).sum();
                MathBox::new(width, total * 0.55, total * 0.45)
            }
        }
    }
}

fn estimate_grid_box(rows: &[Vec<MathAtom>], left: Option<&str>, right: Option<&str>) -> MathBox {
    let mut col_widths: Vec<f64> = Vec::new();
    let mut total_h = 0.0_f64;
    for row in rows {
        let mut row_h = 0.0_f64;
        for (ci, cell) in row.iter().enumerate() {
            let b = cell.estimate_box();
            if col_widths.len() <= ci {
                col_widths.resize(ci + 1, 0.0);
            }
            col_widths[ci] = col_widths[ci].max(b.width);
            row_h = row_h.max(b.total_height());
        }
        total_h += row_h + 0.2;
    }
    let mut width: f64 = col_widths.iter().sum::<f64>() + 0.25 * col_widths.len() as f64;
    if let (Some(l), Some(r)) = (left, right) {
        width += 0.35 * (l.chars().count() + r.chars().count()) as f64;
    }
    MathBox::new(width.max(0.5), total_h * 0.55, total_h * 0.45)
}
