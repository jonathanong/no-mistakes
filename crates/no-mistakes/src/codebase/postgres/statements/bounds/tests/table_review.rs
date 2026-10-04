use super::shape;
use sqlparser::ast::{SetExpr, Statement};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

#[test]
fn long_union_advances_to_quoted_table_arm() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/table-long-union.sql"),
    )
    .unwrap();
    let facts = crate::codebase::postgres::extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 1);
    let mut names = Vec::new();
    collect_table_names(&facts.bounds[0].query, &mut names);
    assert_eq!(names, ["\"Final_Arm\""]);
}

fn collect_table_names(
    query: &crate::codebase::postgres::statements::SqlBoundQuery,
    names: &mut Vec<String>,
) {
    use crate::codebase::postgres::statements::SqlBoundItemKind;
    for item in &query.items {
        match &item.kind {
            SqlBoundItemKind::Table(name) => names.push(name.clone()),
            SqlBoundItemKind::Query(query) => collect_table_names(query, names),
            _ => {}
        }
    }
}

#[test]
fn table_arms_recover_the_source_identifier_spelling() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/table-quoting.sql"
    ));
    assert_eq!(
        shape(sql),
        [
            "select: () (\"Order Items\")",
            "select: () (public.\"Order Items\")",
            "select: () (())",
            "select: () (accounts)",
            "select: () (\"Accounts\")",
            "select: () (accounts)",
            "select: () (\"Accounts\")",
        ]
    );
}

#[test]
fn table_arms_keep_both_spellings_when_source_tokens_are_unavailable() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/table-quoting.sql"
    ));
    let first = sql.split(';').next().expect("saved TABLE fixture");
    let statements = Parser::parse_sql(&PostgreSqlDialect {}, first).expect("first TABLE query");
    let Statement::Query(query) = &statements[0] else {
        panic!("first fixture statement must be a query");
    };
    let SetExpr::SetOperation { right, .. } = &*query.body else {
        panic!("first fixture query must have a TABLE arm");
    };
    let SetExpr::Table(table) = &**right else {
        panic!("right arm must be TABLE");
    };
    let bound = super::super::table::bound(table, &super::super::Scope::default(), (1, 1));
    assert_eq!(super::query(&bound), "order items \"Order Items\"");
}
