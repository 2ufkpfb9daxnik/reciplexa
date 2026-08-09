//! `rpx.std.math` — editable math tree (not text decoration).

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
    },
}

impl MathAtom {
    pub fn id(&self) -> StableNodeId {
        match self {
            Self::Symbol { id, .. }
            | Self::Row { id, .. }
            | Self::Fraction { id, .. }
            | Self::Radical { id, .. }
            | Self::Scripts { id, .. }
            | Self::Delimiter { id, .. } => *id,
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

    pub fn delimiter(
        id: StableNodeId,
        left: impl Into<String>,
        right: impl Into<String>,
        body: MathAtom,
    ) -> Self {
        Self::Delimiter {
            id,
            left: left.into(),
            right: right.into(),
            body: Box::new(body),
        }
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
        }
    }

    /// Linearized debug form (not for layout).
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
                    s.push_str(&sup.linearize());
                }
                if let Some(sub) = subscript {
                    s.push('_');
                    s.push_str(&sub.linearize());
                }
                s
            }
            Self::Delimiter {
                left,
                right,
                body,
                ..
            } => format!("{left}{}{right}", body.linearize()),
        }
    }
}

impl fmt::Display for MathAtom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.linearize())
    }
}
