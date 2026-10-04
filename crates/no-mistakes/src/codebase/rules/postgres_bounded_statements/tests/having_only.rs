use super::{fixture_root, names};

#[test]
fn having_without_an_aggregate_is_still_a_single_group() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/having-only.sql")).unwrap();
    assert_eq!(
        names(&sql),
        [
            "orders", "orders", "orders", "orders", "orders", "orders", "orders", "orders",
            "orders",
        ]
    );
}
