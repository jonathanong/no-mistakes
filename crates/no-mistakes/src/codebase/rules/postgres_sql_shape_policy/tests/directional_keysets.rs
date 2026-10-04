use super::bounded_iteration::{at, found};

#[test]
fn final_inclusive_and_mixed_direction_cursors_remain_whole_table_walks() {
    let config = "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [keyset-only-sweep]\n";
    let expected: Vec<_> = (2..=11).map(|line| (line, "keyset-only-sweep")).collect();
    assert_eq!(
        found("directional-keysets", config, "sql/pages.sql"),
        at(&expected)
    );
}
