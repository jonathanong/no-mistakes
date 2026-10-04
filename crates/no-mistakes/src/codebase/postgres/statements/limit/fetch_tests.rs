use super::{next_table_fetch, Tokens};
use sqlparser::ast::{Query, SetExpr, Statement, Visit, Visitor};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::Tokenizer;
use std::ops::ControlFlow;

#[derive(Default)]
struct SpanlessTableQuery(Option<Query>);

impl Visitor for SpanlessTableQuery {
    type Break = ();

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        let mut body = query.body.as_ref();
        while let SetExpr::Query(inner) = body {
            body = inner.body.as_ref();
        }
        if matches!(body, SetExpr::Table(_)) {
            self.0 = Some(query.clone());
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

#[test]
fn spanless_table_fetch_lookup_rejects_unmatched_or_missing_names() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/review-followups/sql/unspanned-table-fetches.sql"
    ));
    let dialect = PostgreSqlDialect {};
    let prepared = Tokenizer::new(&dialect, sql)
        .tokenize_with_location()
        .unwrap();
    let statements = Parser::new(&dialect)
        .with_tokens_with_locations(prepared.clone())
        .parse_statements()
        .unwrap();
    let tokens = Tokens::with_prepared(sql, &prepared);
    let mut visitor = SpanlessTableQuery::default();
    for statement in &statements {
        if let Statement::Query(query) = statement {
            let _ = query.visit(&mut visitor);
            if visitor.0.is_some() {
                break;
            }
        }
    }
    let query = visitor
        .0
        .expect("saved fixture contains a nested spanless TABLE query");
    let mut cursor = 0;
    let mut renamed = query.clone();
    let SetExpr::Table(table) = renamed.body.as_mut() else {
        panic!("fixture query is a spanless TABLE");
    };
    table.table_name = Some("not_in_source".into());
    assert_eq!(next_table_fetch(&renamed, &tokens, &mut cursor), None);

    let mut unnamed = query;
    let SetExpr::Table(table) = unnamed.body.as_mut() else {
        panic!("fixture query is a spanless TABLE");
    };
    table.table_name = None;
    assert_eq!(next_table_fetch(&unnamed, &tokens, &mut cursor), None);
}
