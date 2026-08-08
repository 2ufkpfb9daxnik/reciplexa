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
        self.first_index.saturating_add(self.visible_count).min(self.total)
    }

    pub fn iter_indices(&self) -> impl Iterator<Item = usize> {
        self.first_index..self.end_index()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_large_list() {
        let w = VirtualWindow::compute(1000, 20, 200, 400);
        assert_eq!(w.first_index, 20);
        assert!(w.visible_count <= 12);
        assert!(w.iter_indices().all(|i| i < 1000));
    }

    #[test]
    fn total_zero_yields_empty_window() {
        let w = VirtualWindow::compute(0, 24, 200, 0);
        assert_eq!(w.total, 0);
        assert_eq!(w.first_index, 0);
        assert_eq!(w.visible_count, 0);
        assert_eq!(w.end_index(), 0);
        assert_eq!(w.iter_indices().count(), 0);
    }

    #[test]
    fn row_height_zero_clamped_to_one() {
        let w = VirtualWindow::compute(10, 0, 50, 0);
        assert_eq!(w.row_height_px, 1);
        assert!(w.visible_count > 0);
    }

    #[test]
    fn scroll_past_end_clamps_first_index() {
        let w = VirtualWindow::compute(5, 10, 100, 10_000);
        assert_eq!(w.first_index, 4);
        assert_eq!(w.end_index(), 5);
    }
}
