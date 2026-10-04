use super::super::{parse_postgres_sql, parse_postgres_sql_lenient, PreparedSql};
use crate::codebase::postgres::{
    extract_sql_statement_facts, parse_postgres_source, PostgresSqlSource,
};
use sqlparser::ast::Statement;

const SQL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/insert-table-only.sql"
));

#[test]
fn insert_table_only_sources_parse_with_target_aliases_and_overrides() {
    let statements = parse_postgres_sql(SQL).expect("all PostgreSQL INSERT TABLE ONLY forms parse");
    assert_eq!(statements.len(), 5);
    assert!(statements
        .iter()
        .all(|statement| matches!(statement, Statement::Insert(_))));
    assert_eq!(parse_postgres_sql_lenient(SQL).len(), 5);

    let facts = extract_sql_statement_facts(SQL);
    assert!(!facts.parse_failed);
    assert_eq!(facts.inserts.len(), 5);
    assert_eq!(facts.selects.len(), 5);
    assert!(facts.selects.iter().all(|fact| fact.in_insert_select));
    assert_eq!(
        facts
            .selects
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [2, 3, 4, 5, 6]
    );
    let sources: Vec<_> = facts
        .selects
        .iter()
        .map(|fact| fact.tables.as_slice())
        .collect();
    assert_eq!(
        sources,
        [
            &["accounts"],
            &["Accounts"],
            &["public.accounts"],
            &["public.Accounts"],
            &["accounts"],
        ]
    );

    // The parser clone may hide ONLY, but the request's located source tokens
    // retain the original text and line positions for other fact collectors.
    let prepared = PreparedSql::new(SQL);
    assert_eq!(
        prepared
            .tokens()
            .iter()
            .filter(|token| token.token.to_string() == "ONLY")
            .map(|token| token.span.start.line as usize)
            .collect::<Vec<_>>(),
        [2, 3, 4, 5, 6]
    );

    let source_facts = parse_postgres_source(&PostgresSqlSource {
        sql: SQL.to_owned(),
        file_name: None,
    });
    assert!(source_facts.diagnostics.is_empty(), "{source_facts:#?}");
    assert_eq!(source_facts.statements.len(), 5);
    for statement in &source_facts.statements {
        assert_eq!(
            &SQL[statement.span.start.offset..statement.span.end.offset],
            statement.sql
        );
    }
}

#[test]
fn malformed_insert_prefixes_do_not_gain_table_only_queries() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/insert-table-only-invalid.sql"
    ));
    for statement in sql.lines().filter(|line| !line.starts_with("--")) {
        assert!(parse_postgres_sql(statement).is_err(), "{statement}");
    }
}
