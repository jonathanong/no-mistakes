use super::{fixture_root, names};

#[test]
fn uncapped_selects_report_their_unbounded_pin_subqueries() {
    let sql =
        std::fs::read_to_string(fixture_root().join("sql/pin-subquery-offenders.sql")).unwrap();
    assert_eq!(names(&sql), ["orders", "orders", "accounts"]);
}
