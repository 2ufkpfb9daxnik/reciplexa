//! `rpx.std.slide` — slide / master / theme / placeholder / notes / transition stubs.

use std::fmt;

use reciplexa_identity::document::StableNodeId;
use reciplexa_scene::PaperSize;

use crate::core::{Color, Size};
use crate::document::Block;

/// Slide deck theme colors and fonts (minimal).
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub id: StableNodeId,
    pub name: String,
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
}

impl Theme {
    pub fn light(id: StableNodeId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            background: Color::WHITE,
            foreground: Color::BLACK,
            accent: Color::BLUE,
        }
    }

    pub fn dark(id: StableNodeId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            background: Color::BLACK,
            foreground: Color::WHITE,
            accent: Color::BLUE,
        }
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("Theme({})", self.name))
    }
}

/// Placeholder role on a master.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceholderKind {
    Title,
    Body,
    Figure,
    Footer,
    SlideNumber,
}

impl PlaceholderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Body => "body",
            Self::Figure => "figure",
            Self::Footer => "footer",
            Self::SlideNumber => "slide-number",
        }
    }
}

impl fmt::Display for PlaceholderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Placeholder {
    pub id: StableNodeId,
    pub kind: PlaceholderKind,
    pub size: Size,
}

impl Placeholder {
    pub fn new(id: StableNodeId, kind: PlaceholderKind, size: Size) -> Self {
        Self { id, kind, size }
    }
}

/// Master layout referenced by slides.
#[derive(Debug, Clone, PartialEq)]
pub struct Master {
    pub id: StableNodeId,
    pub name: String,
    pub size: PaperSize,
    pub placeholders: Vec<Placeholder>,
}

impl Master {
    pub fn widescreen(id: StableNodeId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            size: PaperSize {
                width_mm: 338.67,
                height_mm: 190.5,
            },
            placeholders: Vec::new(),
        }
    }

    pub fn push_placeholder(&mut self, ph: Placeholder) {
        self.placeholders.push(ph);
    }

    pub fn find(&self, kind: PlaceholderKind) -> Option<&Placeholder> {
        self.placeholders.iter().find(|p| p.kind == kind)
    }
}

/// Transition stub between slides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    None,
    Fade,
    Push,
    Dissolve,
}

impl Transition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Fade => "fade",
            Self::Push => "push",
            Self::Dissolve => "dissolve",
        }
    }
}

impl Default for Transition {
    fn default() -> Self {
        Self::None
    }
}

impl fmt::Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Speaker notes attached to a slide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notes {
    pub id: StableNodeId,
    pub text: String,
}

impl Notes {
    pub fn new(id: StableNodeId, text: impl Into<String>) -> Self {
        Self {
            id,
            text: text.into(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// One slide instance.
#[derive(Debug, Clone, PartialEq)]
pub struct Slide {
    pub id: StableNodeId,
    pub master: StableNodeId,
    pub theme: StableNodeId,
    pub blocks: Vec<Block>,
    pub notes: Option<Notes>,
    pub transition: Transition,
}

impl Slide {
    pub fn new(id: StableNodeId, master: StableNodeId, theme: StableNodeId) -> Self {
        Self {
            id,
            master,
            theme,
            blocks: Vec::new(),
            notes: None,
            transition: Transition::None,
        }
    }

    pub fn with_notes(mut self, notes: Notes) -> Self {
        self.notes = Some(notes);
        self
    }

    pub fn with_transition(mut self, transition: Transition) -> Self {
        self.transition = transition;
        self
    }

    pub fn push_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
}
