use super::super::{parse_postgres_sql, parse_postgres_sql_lenient};
use sqlparser::ast::{Query, Spanned, Visit, Visitor};
use std::ops::ControlFlow;

const SQL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/postgres-facts/source/fetch-expressions.sql"
));

#[test]
fn original_fetch_quantities_survive_the_single_prepared_parse() {
    let statements = parse_postgres_sql(SQL).expect("valid parenthesized FETCH expressions");
    assert_eq!(statements.len(), 8);
    struct Quantities(Vec<(String, bool)>);
    impl Visitor for Quantities {
        type Break = ();
        fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
            if let Some(fetch) = &query.fetch {
                let quantity = fetch.quantity.as_ref().expect("explicit quantity");
                assert!(quantity.span().start.line > 0);
                self.0.push((quantity.to_string(), fetch.with_ties));
            }
            ControlFlow::Continue(())
        }
    }
    let mut quantities = Quantities(Vec::new());
    let _ = statements.visit(&mut quantities);
    assert_eq!(quantities.0.len(), 9);
    assert_eq!(quantities.0[0], ("(COALESCE(NULL, 100))".into(), false));
    assert_eq!(quantities.0[6], ("(100)".into(), true));
    // The count's inner LIMIT is a separate query and remains a LIMIT.
    assert!(statements.last().unwrap().to_string().contains("LIMIT 1"));
    assert_eq!(parse_postgres_sql_lenient(SQL), statements);
}

#[test]
fn incomplete_fetch_expression_clauses_remain_invalid() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/fetch-expressions-invalid.sql"));
    for statement in sql.lines() {
        assert!(parse_postgres_sql(statement).is_err(), "{statement}");
    }
}

#[test]
fn source_fetch_preserves_original_sql_without_a_delimiter() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/fetch-no-delimiter.sql"
    ));
    let facts = crate::codebase::postgres::parse_postgres_source(
        &crate::codebase::postgres::PostgresSqlSource {
            sql: sql.into(),
            file_name: None,
        },
    );
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    assert_eq!(facts.statements.len(), 1);
    assert_eq!(facts.statements[0].sql, sql.trim_start_matches("/* 雪 */ "));
    assert_eq!(facts.statements[0].span.end.offset, sql.len());
}

#[test]
fn ordinary_limit_and_literal_fetch_are_unchanged() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/fetch-ordinary.sql"
    ));
    let statements = parse_postgres_sql(sql).expect("ordinary caps");
    assert_eq!(statements.len(), 2);
    assert!(statements[0].to_string().contains("LIMIT 10"));
    assert!(statements[1]
        .to_string()
        .contains("FETCH FIRST 10 ROWS ONLY"));
}

#[test]
fn quoted_fetch_words_take_the_ordinary_parse_path() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/sql-fetch-no-candidate.sql"
    ));
    let statements = parse_postgres_sql(sql).expect("FETCH in a string is not a clause");
    assert_eq!(statements.len(), 1);
    assert!(statements[0].to_string().contains("LIMIT 50"));
    assert!(statements[0].to_string().contains("FETCH FIRST (ignored)"));
}
