use super::facts;
use crate::codebase::postgres::SqlPinSource;

#[test]
fn scalar_result_proofs_retain_argument_items_and_direct_leaf_requirements() {
    let facts = facts(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/scalar-reducer-arrays.sql"
    )));
    assert_eq!(facts.len(), 16);
    for index in [0, 1, 2] {
        let SqlPinSource::Array {
            items,
            scalar_columns,
            ..
        } = &facts[index].query.items[0].pins[0].source
        else {
            panic!("scalar reducer lost its argument owner");
        };
        assert_eq!(items, &[1]);
        assert!(scalar_columns.is_empty());
    }
    // A direct constructor leaf still requires scalar catalog evidence.
    for (index, column) in [
        (10, "status"),
        (11, "account_ids"),
        (13, "account_ids"),
        (15, "account_ids"),
    ] {
        let SqlPinSource::Array {
            items,
            scalar_columns,
            ..
        } = &facts[index].query.items[0].pins[0].source
        else {
            panic!("direct column lost its type proof");
        };
        assert_eq!(items, &[1]);
        assert_eq!(scalar_columns, &[(1, column.to_string())]);
    }
}
