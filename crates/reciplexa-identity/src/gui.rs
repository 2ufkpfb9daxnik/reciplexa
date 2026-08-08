//! GUI widget key path (`specification.md` GUI State Identity §4.4).

use core::fmt;

/// Segment of a widget key path within a semantic owner.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WidgetKeySegment {
    Named(String),
    Index(u32),
}

/// Path distinguishing widgets under the same semantic owner.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct WidgetKeyPath {
    segments: Vec<WidgetKeySegment>,
}

impl WidgetKeyPath {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn push_named(mut self, name: impl Into<String>) -> Self {
        self.segments.push(WidgetKeySegment::Named(name.into()));
        self
    }

    pub fn push_index(mut self, index: u32) -> Self {
        self.segments.push(WidgetKeySegment::Index(index));
        self
    }

    pub fn segments(&self) -> &[WidgetKeySegment] {
        &self.segments
    }
}

impl fmt::Display for WidgetKeyPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<_> = self
            .segments
            .iter()
            .map(|s| match s {
                WidgetKeySegment::Named(n) => n.clone(),
                WidgetKeySegment::Index(i) => i.to_string(),
            })
            .collect();
        write!(f, "{}", parts.join("/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_formats_named_and_index_segments() {
        let path = WidgetKeyPath::new().push_named("toolbar").push_index(2);
        assert_eq!(path.to_string(), "toolbar/2");
    }
}
