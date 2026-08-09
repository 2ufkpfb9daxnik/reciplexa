//! `rpx.std.layout` — stack / row / column / padding / align stubs.

use std::fmt;

use reciplexa_identity::document::StableNodeId;

use crate::core::{Length, Size};

/// Cross-axis / main-axis alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

impl Align {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
            Self::Stretch => "stretch",
        }
    }
}

impl fmt::Display for Align {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Uniform or per-side padding in millimeters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Padding {
    pub top: Length,
    pub right: Length,
    pub bottom: Length,
    pub left: Length,
}

impl Padding {
    pub const ZERO: Self = Self {
        top: Length::ZERO,
        right: Length::ZERO,
        bottom: Length::ZERO,
        left: Length::ZERO,
    };

    pub const fn all(v: Length) -> Self {
        Self {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }

    pub const fn symmetric(vertical: Length, horizontal: Length) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub fn horizontal(self) -> Length {
        self.left.saturating_add(self.right)
    }

    pub fn vertical(self) -> Length {
        self.top.saturating_add(self.bottom)
    }

    pub fn is_non_negative(self) -> bool {
        self.top.is_non_negative()
            && self.right.is_non_negative()
            && self.bottom.is_non_negative()
            && self.left.is_non_negative()
    }
}

impl Default for Padding {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Padding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pad(t={}, r={}, b={}, l={})",
            self.top, self.right, self.bottom, self.left
        )
    }
}

/// Layout axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// Child reference inside a layout container (typed stub).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutChild {
    pub id: StableNodeId,
}

impl LayoutChild {
    pub const fn new(id: StableNodeId) -> Self {
        Self { id }
    }
}

/// Generic stack along an axis.
#[derive(Debug, Clone, PartialEq)]
pub struct Stack {
    pub id: StableNodeId,
    pub axis: Axis,
    pub gap: Length,
    pub padding: Padding,
    pub align: Align,
    pub children: Vec<LayoutChild>,
}

impl Stack {
    pub fn new(id: StableNodeId, axis: Axis) -> Self {
        Self {
            id,
            axis,
            gap: Length::ZERO,
            padding: Padding::ZERO,
            align: Align::Start,
            children: Vec::new(),
        }
    }

    pub fn with_gap(mut self, gap: Length) -> Self {
        self.gap = gap;
        self
    }

    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn push(&mut self, child: LayoutChild) {
        self.children.push(child);
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Minimum size estimate: padding + gaps only (children sizes unknown).
    pub fn intrinsic_chrome(&self) -> Size {
        let gaps = if self.children.len() > 1 {
            self.gap.scale((self.children.len() - 1) as f64)
        } else {
            Length::ZERO
        };
        match self.axis {
            Axis::Horizontal => Size::new(
                self.padding.horizontal().saturating_add(gaps),
                self.padding.vertical(),
            ),
            Axis::Vertical => Size::new(
                self.padding.horizontal(),
                self.padding.vertical().saturating_add(gaps),
            ),
        }
    }
}

/// Horizontal stack alias.
#[derive(Debug, Clone, PartialEq)]
pub struct Row(pub Stack);

impl Row {
    pub fn new(id: StableNodeId) -> Self {
        Self(Stack::new(id, Axis::Horizontal))
    }

    pub fn stack(&self) -> &Stack {
        &self.0
    }

    pub fn stack_mut(&mut self) -> &mut Stack {
        &mut self.0
    }
}

/// Vertical stack alias.
#[derive(Debug, Clone, PartialEq)]
pub struct Column(pub Stack);

impl Column {
    pub fn new(id: StableNodeId) -> Self {
        Self(Stack::new(id, Axis::Vertical))
    }

    pub fn stack(&self) -> &Stack {
        &self.0
    }

    pub fn stack_mut(&mut self) -> &mut Stack {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> StableNodeId {
        StableNodeId::new(n)
    }

    #[test]
    fn align_padding_display() {
        assert_eq!(Align::Start.as_str(), "start");
        assert_eq!(Align::Center.to_string(), "center");
        assert_eq!(Align::End.as_str(), "end");
        assert_eq!(Align::Stretch.as_str(), "stretch");
        assert_eq!(Padding::default(), Padding::ZERO);
        let p = Padding::all(Length::mm(2.0));
        assert!(p.is_non_negative());
        assert_eq!(p.horizontal().as_mm(), 4.0);
        assert_eq!(p.vertical().as_mm(), 4.0);
        let s = Padding::symmetric(Length::mm(1.0), Length::mm(3.0));
        assert_eq!(s.top.as_mm(), 1.0);
        assert_eq!(s.left.as_mm(), 3.0);
        assert!(p.to_string().contains("pad("));
        assert!(!Padding {
            top: Length::mm(-1.0),
            ..Padding::ZERO
        }
        .is_non_negative());
    }

    #[test]
    fn stack_row_column() {
        let mut stack = Stack::new(id(1), Axis::Vertical)
            .with_gap(Length::mm(2.0))
            .with_padding(Padding::all(Length::mm(1.0)))
            .with_align(Align::Center);
        stack.push(LayoutChild::new(id(2)));
        stack.push(LayoutChild::new(id(3)));
        assert_eq!(stack.child_count(), 2);
        let chrome = stack.intrinsic_chrome();
        assert_eq!(chrome.width.as_mm(), 2.0);
        assert_eq!(chrome.height.as_mm(), 4.0); // pad 2 + gap 2
        let mut row = Row::new(id(10));
        row.stack_mut().push(LayoutChild::new(id(11)));
        assert_eq!(row.stack().axis, Axis::Horizontal);
        assert_eq!(row.stack().child_count(), 1);
        let empty = Stack::new(id(1), Axis::Horizontal);
        assert_eq!(empty.intrinsic_chrome(), Size::ZERO);
        let mut col = Column::new(id(20));
        col.stack_mut().push(LayoutChild::new(id(21)));
        col.stack_mut().push(LayoutChild::new(id(22)));
        col.stack_mut().push(LayoutChild::new(id(23)));
        col.stack_mut().gap = Length::mm(1.0);
        let c = col.stack().intrinsic_chrome();
        assert_eq!(c.height.as_mm(), 2.0);
    }
}
