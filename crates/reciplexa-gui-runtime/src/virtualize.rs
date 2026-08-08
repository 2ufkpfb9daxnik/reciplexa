//! List virtualization windowing (Phase 8).

/// Visible window over a logical list for large layer trees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualWindow {
    pub first_index: usize,
    pub visible_count: usize,
    pub total: usize,
    pub row_height_px: u32,
    pub viewport_height_px: u32,
    pub scroll_offset_px: u32,
}

impl VirtualWindow {
    pub fn compute(
        total: usize,
        row_height_px: u32,
        viewport_height_px: u32,
        scroll_offset_px: u32,
    ) -> Self {
        let row_h = row_height_px.max(1);
        let visible_count = (viewport_height_px / row_h).saturating_add(2) as usize;
        let first_index = (scroll_offset_px / row_h) as usize;
        let first_index = first_index.min(total.saturating_sub(1).max(0));
        Self {
            first_index,
            visible_count: visible_count.min(total.saturating_sub(first_index)),
            total,
            row_height_px: row_h,
            viewport_height_px,
            scroll_offset_px,
        }
    }

    pub fn end_index(&self) -> usize {
        self.first_index
            .saturating_add(self.visible_count)
            .min(self.total)
    }

    pub fn iter_indices(&self) -> impl Iterator<Item = usize> {
        self.first_index..self.end_index()
    }
}
