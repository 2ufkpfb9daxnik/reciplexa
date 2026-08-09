//! `rpx.std.document` — page / flow / section stubs for Japanese document package.

use std::fmt;

use reciplexa_identity::document::StableNodeId;
use reciplexa_scene::PaperSize;

use crate::core::{Length, Size};
use crate::text::{Paragraph, TextStyle};

/// Document page with paper size and flow root.
#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub id: StableNodeId,
    pub paper: PaperSize,
    pub flow: Flow,
}

impl Page {
    pub fn a4(id: StableNodeId) -> Self {
        Self {
            id,
            paper: PaperSize::a4(),
            flow: Flow::new(StableNodeId::new(id.get().saturating_add(1))),
        }
    }

    pub fn is_positive(&self) -> bool {
        self.paper.is_positive()
    }
}

/// Block flow of sections (reading order).
#[derive(Debug, Clone, PartialEq)]
pub struct Flow {
    pub id: StableNodeId,
    pub sections: Vec<Section>,
}

impl Flow {
    pub fn new(id: StableNodeId) -> Self {
        Self {
            id,
            sections: Vec::new(),
        }
    }

    pub fn push_section(&mut self, section: Section) {
        self.sections.push(section);
    }

    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }
}

/// Named section containing block children.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub id: StableNodeId,
    pub title: Option<Heading>,
    pub blocks: Vec<Block>,
}

impl Section {
    pub fn new(id: StableNodeId) -> Self {
        Self {
            id,
            title: None,
            blocks: Vec::new(),
        }
    }

    pub fn with_heading(mut self, heading: Heading) -> Self {
        self.title = Some(heading);
        self
    }

    pub fn push(&mut self, block: Block) {
        self.blocks.push(block);
    }
}

/// Heading level 1..=6.
#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    pub id: StableNodeId,
    pub level: u8,
    pub text: String,
    pub style: TextStyle,
}

impl Heading {
    pub fn new(id: StableNodeId, level: u8, text: impl Into<String>) -> Self {
        let level = level.clamp(1, 6);
        Self {
            id,
            level,
            text: text.into(),
            style: TextStyle::heading(level),
        }
    }

    pub fn is_valid_level(self) -> bool {
        (1..=6).contains(&self.level)
    }
}

/// Ordered or unordered list.
#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub id: StableNodeId,
    pub ordered: bool,
    pub items: Vec<ListItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub id: StableNodeId,
    pub paragraphs: Vec<Paragraph>,
}

impl List {
    pub fn unordered(id: StableNodeId) -> Self {
        Self {
            id,
            ordered: false,
            items: Vec::new(),
        }
    }

    pub fn ordered(id: StableNodeId) -> Self {
        Self {
            id,
            ordered: true,
            items: Vec::new(),
        }
    }

    pub fn push_item(&mut self, item: ListItem) {
        self.items.push(item);
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Simple table stub (rows of cell text).
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub id: StableNodeId,
    pub columns: usize,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    pub fn new(id: StableNodeId, columns: usize) -> Self {
        Self {
            id,
            columns: columns.max(1),
            rows: Vec::new(),
        }
    }

    pub fn push_row(&mut self, cells: Vec<String>) -> bool {
        if cells.len() != self.columns {
            return false;
        }
        self.rows.push(cells);
        true
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}

/// Captionable figure referencing a visual node.
#[derive(Debug, Clone, PartialEq)]
pub struct Figure {
    pub id: StableNodeId,
    pub visual: StableNodeId,
    pub caption: Option<String>,
    pub preferred_size: Option<Size>,
}

impl Figure {
    pub fn new(id: StableNodeId, visual: StableNodeId) -> Self {
        Self {
            id,
            visual,
            caption: None,
            preferred_size: None,
        }
    }

    pub fn with_caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    pub fn with_size(mut self, size: Size) -> Self {
        self.preferred_size = Some(size);
        self
    }
}

/// Block-level content inside a section.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Paragraph(Paragraph),
    Heading(Heading),
    List(List),
    Table(Table),
    Figure(Figure),
    Spacer(Length),
}

impl Block {
    pub fn id(&self) -> Option<StableNodeId> {
        match self {
            Self::Paragraph(p) => Some(p.id),
            Self::Heading(h) => Some(h.id),
            Self::List(l) => Some(l.id),
            Self::Table(t) => Some(t.id),
            Self::Figure(f) => Some(f.id),
            Self::Spacer(_) => None,
        }
    }
}

impl fmt::Display for Heading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "h{}:{}", self.level, self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::TextStyle;

    fn id(n: u64) -> StableNodeId {
        StableNodeId::new(n)
    }

    #[test]
    fn page_flow_section_heading() {
        let mut page = Page::a4(id(1));
        assert!(page.is_positive());
        assert!(page.flow.is_empty());
        let h = Heading::new(id(3), 0, "Title");
        assert_eq!(h.level, 1);
        assert!(h.is_valid_level());
        assert!(h.to_string().starts_with("h1:"));
        let h6 = Heading::new(id(4), 9, "Deep");
        assert_eq!(h6.level, 6);
        let mut section = Section::new(id(2)).with_heading(h);
        section.push(Block::Paragraph(Paragraph::plain(
            id(5),
            "body",
            TextStyle::body(),
        )));
        section.push(Block::Spacer(Length::mm(4.0)));
        page.flow.push_section(section);
        assert!(!page.flow.is_empty());
        assert_eq!(page.flow.sections[0].blocks[0].id(), Some(id(5)));
        assert_eq!(page.flow.sections[0].blocks[1].id(), None);
    }

    #[test]
    fn list_table_figure() {
        let mut list = List::unordered(id(1));
        assert!(list.is_empty());
        list.push_item(ListItem {
            id: id(2),
            paragraphs: vec![Paragraph::plain(id(3), "a", TextStyle::body())],
        });
        assert_eq!(list.len(), 1);
        let ordered = List::ordered(id(4));
        assert!(ordered.ordered);
        let mut table = Table::new(id(5), 0);
        assert_eq!(table.columns, 1);
        assert!(!table.push_row(vec!["a".into(), "b".into()]));
        assert!(table.push_row(vec!["a".into()]));
        assert_eq!(table.row_count(), 1);
        let fig = Figure::new(id(6), id(7))
            .with_caption("cap")
            .with_size(Size::mm(10.0, 10.0));
        assert_eq!(fig.caption.as_deref(), Some("cap"));
        assert!(matches!(
            Block::Figure(fig.clone()).id(),
            Some(i) if i == id(6)
        ));
        assert_eq!(Block::Heading(Heading::new(id(8), 2, "x")).id(), Some(id(8)));
        assert_eq!(Block::List(list).id(), Some(id(1)));
        assert_eq!(Block::Table(table).id(), Some(id(5)));
    }
}
