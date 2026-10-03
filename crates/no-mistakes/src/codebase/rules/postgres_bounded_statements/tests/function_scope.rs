use super::{fixture_root, unbounded};
use crate::codebase::postgres::extract_sql_statement_facts;

#[test]
fn table_functions_do_not_hide_outer_schema_qualified_table_references() {
    let sql =
        std::fs::read_to_string(fixture_root().join("sql/function-correlation-scope.sql")).unwrap();
    assert!(!extract_sql_statement_facts(&sql).parse_failed);
    assert_eq!(
        unbounded(&sql),
        vec![("accounts".into(), 2), ("accounts".into(), 3)]
    );
}
