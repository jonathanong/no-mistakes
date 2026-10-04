use super::super::{parse_postgres_sql, parse_postgres_sql_lenient};
use sqlparser::ast::{SetExpr, Statement};

const SQL: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/table-arm-statement-boundaries.sql"));

#[test]
fn right_table_arms_preserve_following_statement_boundaries_and_ast_identity() {
    let strict = parse_postgres_sql(SQL).expect("TABLE must leave the next statement intact");
    assert_eq!(strict.len(), 4);
    assert_eq!(parse_postgres_sql_lenient(SQL), strict);
    for (statement, name) in strict.iter().zip(["Topics", "topics", "topics"]) {
        let Statement::Query(query) = statement else {
            panic!("query expected");
        };
        let SetExpr::SetOperation { right, .. } = &*query.body else {
            panic!("set operation expected");
        };
        let SetExpr::Table(table) = &**right else {
            panic!("original TABLE AST must survive compatibility normalization");
        };
        assert_eq!(table.table_name.as_deref(), Some(name));
    }
}

#[test]
fn a_single_right_table_arm_keeps_its_existing_ast_with_or_without_a_delimiter() {
    let root = crate::test_support::rule_fixture_root("postgres-bounded-statements");
    for fixture in ["table-arm-boundary-final.sql", "table-arm-boundary-eof.sql"] {
        let sql = std::fs::read_to_string(root.join("sql").join(fixture)).unwrap();
        let strict = parse_postgres_sql(&sql).unwrap();
        assert_eq!(strict.len(), 1);
        assert_eq!(parse_postgres_sql_lenient(&sql), strict);
        let Statement::Query(query) = &strict[0] else {
            panic!("query expected");
        };
        let SetExpr::SetOperation { right, .. } = &*query.body else {
            panic!("set operation expected");
        };
        assert!(matches!(&**right, SetExpr::Table(_)));
    }
}
