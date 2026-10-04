use super::{facts, SqlPinSource};

#[test]
fn read_only_pin_queries_keep_original_nested_locations_and_mapping() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/pin-query-read-preservation.sql"));
    let mut facts = facts(sql);
    let first = &mut facts[0];
    let SqlPinSource::ReadQuery(query) = &first.query.items[0].pins[0].source else {
        panic!("correlated subquery must retain read-only proof");
    };
    assert_eq!(query.items[0].line, 2);
    let column = query.items[0].column;
    first.map_lines(&|line, at| line + at + 100);
    let SqlPinSource::ReadQuery(query) = &first.query.items[0].pins[0].source else {
        unreachable!()
    };
    assert_eq!(query.items[0].line, 102 + column);
}
