//! Wave 22 P1 tip: measure_columns aligns with kihon multi-column package tags.

use reciplexa_std::japanese::measure_columns;

#[test]
fn tip_wave22_measure_columns_kihon_link() {
    // Package `japanese/kihon`: `column-count-two` (=2) + `multi-column` record
    // carries `(hanmen, columns, gutter-em)` — std `measure_columns` is the
    // fontless geometry helper hosts can use until document lower consumes
    // `jlreq-multi-column`.
    let columns = 2u32; // column-count-two
    let gutter_em = 1.0;
    let total_em = 41.0;
    let (col_w, xs) = measure_columns(total_em, columns, gutter_em);
    assert!((col_w - 20.0).abs() < 1e-9);
    assert_eq!(xs, vec![0.0, 21.0]);
}
