use super::{facts, shape};

const SQL: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/oversized-cte-aliased-pins.sql"));

#[test]
fn compacted_positional_aliases_do_not_gain_catalog_key_identity() {
    assert!(!crate::codebase::postgres::extract_sql_statement_facts(SQL).parse_failed);
    let shapes = shape(SQL);
    assert_eq!(shapes.len(), 5);
    assert!(shapes[1].contains("orders[id=value]"));
    for index in [0, 2, 3, 4] {
        assert!(
            !shapes[index].contains("orders[id=value]"),
            "{}",
            shapes[index]
        );
    }
    assert!(facts(SQL).iter().all(|fact| !fact.query.capped));
}
