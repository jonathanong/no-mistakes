use super::facts;
use crate::codebase::postgres::SqlPinSource;

#[test]
fn stored_any_sources_are_retained_without_becoming_caller_arrays() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/stored-array-facts.sql"
    ));
    let facts = facts(sql);
    assert_eq!(facts.len(), 8);
    for (index, items) in [
        (0, vec![1]),
        (1, vec![1]),
        (2, vec![0]),
        (6, vec![1]),
        (7, vec![]),
    ] {
        let pins = &facts[index].query.items[0].pins;
        assert_eq!(pins.len(), 1);
        assert_eq!(pins[0].column, "id");
        assert_eq!(pins[0].source, SqlPinSource::StoredArray(items));
    }
    assert_eq!(
        facts[7].query.items[0].pins[0].reads[0].column,
        "account_ids"
    );
    for index in [3, 4, 5] {
        assert_eq!(
            facts[index].query.items[0].pins[0].source,
            SqlPinSource::Value
        );
    }
}
