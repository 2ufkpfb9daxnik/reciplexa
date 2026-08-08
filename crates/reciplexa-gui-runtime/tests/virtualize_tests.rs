use reciplexa_gui_runtime::virtualize::*;

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
