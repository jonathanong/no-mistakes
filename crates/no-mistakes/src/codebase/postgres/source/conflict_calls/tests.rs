use super::*;
use sqlparser::tokenizer::Tokenizer;

#[test]
fn conflict_extension_is_narrow_and_keeps_nested_query_clauses() {
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres/function-call-clauses/conflict-recovery.sql"
    ));
    let outcomes = fixture
        .lines()
        .map(|line| {
            let tokens = Tokenizer::new(&PostgreSqlDialect {}, line.trim_end_matches(';'))
                .tokenize_with_location()
                .unwrap();
            recover(&tokens)
        })
        .collect::<Vec<_>>();
    assert!(outcomes[0].is_none());
    assert!(outcomes[1].as_ref().unwrap().1.is_empty());
    assert!(outcomes[2].is_none());
    assert!(outcomes[3].is_none());
    let calls = &outcomes[4].as_ref().unwrap().1;
    assert_eq!(
        calls.iter().map(|call| call.clause).collect::<Vec<_>>(),
        [
            Some(SqlFunctionClause::SelectList),
            Some(SqlFunctionClause::Where)
        ]
    );
}

#[test]
fn prepared_conflict_program_preserves_native_delimiters_and_errors() {
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres/function-call-clauses/conflict-program.sql"
    ));
    let outcomes = fixture
        .lines()
        .map(|line| {
            let tokens = Tokenizer::new(&PostgreSqlDialect {}, line)
                .tokenize_with_location()
                .unwrap();
            program::parse(tokens)
        })
        .collect::<Vec<_>>();
    let (statements, calls) = outcomes[0].as_ref().unwrap();
    assert_eq!(statements.len(), 2);
    assert_eq!(calls.len(), 1);
    assert!(outcomes[1].is_err());
    assert!(outcomes[2].is_err());
    // This is the native parser's enclosing-block END boundary, not an EOF waiver.
    assert_eq!(outcomes[3].as_ref().unwrap().0.len(), 1);
    assert!(outcomes[4].as_ref().unwrap().1.is_empty());
}

#[test]
fn prepared_conflict_extension_preserves_native_outer_with_query() {
    use crate::codebase::postgres::{function_calls, SqlFunctionClause};
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres/function-call-clauses/conflict-program.sql"
    ));
    let lines = fixture.lines().skip(5).collect::<Vec<_>>();
    let mut first = None;
    for (index, line) in lines.iter().enumerate() {
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, line)
            .tokenize_with_location()
            .unwrap();
        let (statements, mut calls) = program::parse(tokens).unwrap();
        assert!(matches!(&statements[0], Statement::Query(query) if query.with.is_some()));
        function_calls::collect(&statements[0], &mut calls);
        let mut clauses = calls.iter().map(|call| call.clause).collect::<Vec<_>>();
        clauses.sort();
        if index == 0 {
            let native = Parser::new(&PostgreSqlDialect {})
                .try_with_sql(line)
                .unwrap()
                .parse_statements()
                .unwrap();
            assert_eq!(statements, native);
            assert_eq!(clauses, [Some(SqlFunctionClause::SelectList)]);
            first = Some(statements);
        } else {
            assert_eq!(
                clauses,
                [
                    Some(SqlFunctionClause::Where),
                    Some(SqlFunctionClause::SelectList)
                ]
            );
            assert_eq!(statements, first.as_ref().unwrap().clone());
        }
    }
}

#[test]
fn failed_prepared_projection_does_not_publish_partial_conflict_calls() {
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres/function-call-clauses/conflict-invalid-view.sql"
    ));
    let prepared = crate::codebase::postgres::parse::PreparedSql::new(fixture);
    assert!(prepared.parse_policy().is_err());
    assert!(prepared.functions().is_empty());
    let statements = crate::codebase::postgres::extract_sql_statement_facts(fixture);
    assert!(statements.parse_failed);
    assert_eq!(statements.function_calls.len(), 1);
}

#[test]
fn public_parser_never_returns_a_lossy_conflict_ast() {
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres/function-call-clauses/conflict-recovery.sql"
    ));
    let sql = fixture.lines().nth(5).unwrap();
    let native = Parser::new(&PostgreSqlDialect {})
        .try_with_sql(sql)
        .unwrap()
        .parse_statements();
    let public = crate::codebase::postgres::parse_postgres_sql(sql);
    assert!(native.is_err());
    assert!(public.is_err());
    let prepared = crate::codebase::postgres::parse::PreparedSql::new(sql);
    assert!(prepared.parse_policy().is_ok());
    assert_eq!(prepared.functions().len(), 2);
    // Request-owned facts are a separate projection, not a successful public AST.
    assert!(prepared.parse().is_err());
    assert!(prepared.functions().is_empty());
}
