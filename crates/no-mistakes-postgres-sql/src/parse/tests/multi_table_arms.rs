use super::super::{parse_postgres_sql, parse_postgres_sql_lenient};
use sqlparser::ast::Statement;

const STRICT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/multi-table-arms.sql"
));
const LENIENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/multi-table-arms-lenient.sql"
));
const QUALIFIED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/multi-table-arms-qualified.sql"
));

#[test]
fn multiple_table_arms_leave_a_following_select_parseable() {
    let parsed = parse_postgres_sql(STRICT).expect("three PostgreSQL query arms parse");
    assert_eq!(parsed.len(), 2);
    assert!(matches!(parsed[1], Statement::Query(_)));

    let recovered = parse_postgres_sql_lenient(LENIENT);
    assert!(recovered
        .iter()
        .any(|statement| matches!(statement, Statement::Query(_))));
    assert_eq!(parse_postgres_sql(QUALIFIED).unwrap().len(), 2);
}
