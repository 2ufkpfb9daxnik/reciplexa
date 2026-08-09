//! `rpx.std.text` — text runs, boxes, paragraphs, and font style.

use std::fmt;

use reciplexa_identity::document::StableNodeId;
use reciplexa_scene::{Shape, Text as SceneText};

use crate::core::{Color, Length, Point, Rect, Size};

/// Font family name plus optional fallbacks (codec-serializable later).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Font {
    pub family: String,
    pub fallbacks: Vec<String>,
}

impl Font {
    pub fn new(family: impl Into<String>) -> Self {
        Self {
            family: family.into(),
            fallbacks: Vec::new(),
        }
    }

    pub fn with_fallback(mut self, family: impl Into<String>) -> Self {
        self.fallbacks.push(family.into());
        self
    }

    pub fn is_named(&self) -> bool {
        !self.family.is_empty()
    }
}

impl Default for Font {
    fn default() -> Self {
        Self::new("sans-serif")
    }
}

impl fmt::Display for Font {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.family)?;
        for fb in &self.fallbacks {
            write!(f, ",{fb}")?;
        }
        Ok(())
    }
}

/// Typographic style for spans and paragraphs.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub font: Font,
    pub size: Length,
    pub color: Color,
    pub bold: bool,
    pub italic: bool,
}

impl TextStyle {
    pub fn body() -> Self {
        Self {
            font: Font::default(),
            size: Length::mm(4.0),
            color: Color::BLACK,
            bold: false,
            italic: false,
        }
    }

    pub fn heading(level: u8) -> Self {
        let size = match level {
            0 | 1 => Length::mm(8.0),
            2 => Length::mm(6.0),
            3 => Length::mm(5.0),
            _ => Length::mm(4.5),
        };
        Self {
            font: Font::default(),
            size,
            color: Color::BLACK,
            bold: true,
            italic: false,
        }
    }

    pub fn is_drawable(&self) -> bool {
        self.font.is_named() && self.size.is_positive() && self.color.is_valid()
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::body()
    }
}

impl fmt::Display for TextStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TextStyle{{font={}, size={}, bold={}, italic={}}}",
            self.font, self.size, self.bold, self.italic
        )
    }
}

/// Inline run of characters sharing one style.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub id: StableNodeId,
    pub text: String,
    pub style: TextStyle,
}

impl Span {
    pub fn new(id: StableNodeId, text: impl Into<String>, style: TextStyle) -> Self {
        Self {
            id,
            text: text.into(),
            style,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// Block of spans forming a paragraph.
#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    pub id: StableNodeId,
    pub spans: Vec<Span>,
}

impl Paragraph {
    pub fn plain(id: StableNodeId, text: impl Into<String>, style: TextStyle) -> Self {
        Self {
            id,
            spans: vec![Span::new(StableNodeId::new(id.get().saturating_add(1)), text, style)],
        }
    }

    pub fn plain_text(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.spans.is_empty() || self.spans.iter().all(|s| s.is_empty())
    }
}

/// Free-floating text drawable (baseline origin).
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub id: StableNodeId,
    pub origin: Point,
    pub content: String,
    pub style: TextStyle,
}

impl Text {
    pub fn new(
        id: StableNodeId,
        origin: Point,
        content: impl Into<String>,
        style: TextStyle,
    ) -> Self {
        Self {
            id,
            origin,
            content: content.into(),
            style,
        }
    }

    pub fn to_scene_shape(&self) -> Option<Shape> {
        if self.content.is_empty() || !self.style.is_drawable() {
            return None;
        }
        Some(Shape::Text(SceneText {
            x_mm: self.origin.x.as_mm(),
            y_mm: self.origin.y.as_mm(),
            size_mm: self.style.size.as_mm(),
            width_mm: None,
            height_mm: None,
            content: self.content.clone(),
            fill: self.style.color.to_scene(),
        }))
    }
}

/// Editable text box with layout frame.
#[derive(Debug, Clone, PartialEq)]
pub struct TextBox {
    pub id: StableNodeId,
    pub frame: Rect,
    pub paragraphs: Vec<Paragraph>,
    pub style: TextStyle,
}

impl TextBox {
    pub fn new(id: StableNodeId, frame: Rect, style: TextStyle) -> Self {
        Self {
            id,
            frame,
            paragraphs: Vec::new(),
            style,
        }
    }

    pub fn with_plain(mut self, text: impl Into<String>) -> Self {
        let para_id = StableNodeId::new(self.id.get().saturating_add(1));
        self.paragraphs
            .push(Paragraph::plain(para_id, text, self.style.clone()));
        self
    }

    pub fn plain_text(&self) -> String {
        self.paragraphs
            .iter()
            .map(|p| p.plain_text())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn to_scene_shape(&self) -> Option<Shape> {
        if !self.frame.is_drawable() || !self.style.is_drawable() {
            return None;
        }
        let content = self.plain_text();
        if content.is_empty() {
            return None;
        }
        Some(Shape::Text(SceneText {
            x_mm: self.frame.origin.x.as_mm(),
            y_mm: self.frame.origin.y.as_mm() + self.frame.height().as_mm()
                - self.style.size.as_mm(),
            size_mm: self.style.size.as_mm(),
            width_mm: Some(self.frame.width().as_mm()),
            height_mm: Some(self.frame.height().as_mm()),
            content,
            fill: self.style.color.to_scene(),
        }))
    }

    pub fn estimated_size(&self) -> Size {
        self.frame.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> StableNodeId {
        StableNodeId::new(n)
    }

    #[test]
    fn font_and_text_style() {
        let f = Font::new("Noto Sans").with_fallback("sans-serif");
        assert!(f.is_named());
        assert!(f.to_string().contains("Noto Sans"));
        assert!(!Font::new("").is_named());
        assert_eq!(Font::default().family, "sans-serif");
        let body = TextStyle::body();
        assert!(body.is_drawable());
        assert!(TextStyle::heading(1).bold);
        assert!(TextStyle::heading(2).size.as_mm() < TextStyle::heading(1).size.as_mm());
        assert!(TextStyle::heading(3).size.is_positive());
        assert!(TextStyle::heading(9).size.is_positive());
        assert!(!TextStyle {
            size: Length::ZERO,
            ..TextStyle::body()
        }
        .is_drawable());
        assert!(TextStyle::default().to_string().contains("TextStyle"));
    }

    #[test]
    fn span_paragraph_text_textbox() {
        let style = TextStyle::body();
        let span = Span::new(id(1), "hi", style.clone());
        assert!(!span.is_empty());
        assert!(Span::new(id(1), "", style.clone()).is_empty());
        let p = Paragraph::plain(id(10), "hello", style.clone());
        assert_eq!(p.plain_text(), "hello");
        assert!(!p.is_empty());
        assert!(Paragraph {
            id: id(1),
            spans: vec![]
        }
        .is_empty());
        let t = Text::new(id(2), Point::mm(0.0, 0.0), "x", style.clone());
        assert!(matches!(t.to_scene_shape(), Some(Shape::Text(_))));
        assert!(Text::new(id(2), Point::ORIGIN, "", style.clone())
            .to_scene_shape()
            .is_none());
        let box_ = TextBox::new(id(3), Rect::from_xywh(0.0, 0.0, 50.0, 20.0), style)
            .with_plain("line");
        assert_eq!(box_.plain_text(), "line");
        assert!(matches!(box_.to_scene_shape(), Some(Shape::Text(_))));
        assert_eq!(box_.estimated_size(), Size::mm(50.0, 20.0));
        assert!(TextBox::new(id(3), Rect::from_xywh(0.0, 0.0, 0.0, 1.0), TextStyle::body())
            .with_plain("x")
            .to_scene_shape()
            .is_none());
        assert!(TextBox::new(id(3), Rect::from_xywh(0.0, 0.0, 10.0, 10.0), TextStyle::body())
            .to_scene_shape()
            .is_none());
    }
}
