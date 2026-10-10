use super::*;
use crate::codebase::postgres::source::dialect::PostgresSourceDialect;
use crate::codebase::postgres::source::locations::Locations;
use sqlparser::ast::{SetExpr, Spanned, Statement};
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Span, Token, TokenWithSpan};

fn project_sql(sql: &str) -> PostgresSqlQuery {
    let ast = Parser::parse_sql(&PostgresSourceDialect, sql).unwrap();
    let Statement::Query(query) = &ast[0] else {
        panic!("query");
    };
    super::super::project(query, &Locations::new(sql))
}

fn rparen_at(locations: &Locations<'_>, offset: usize) -> AttachedToken {
    let position = locations.range(offset, offset).start;
    let location =
        sqlparser::tokenizer::Location::new(position.line as u64, position.column as u64);
    AttachedToken(TokenWithSpan::new(
        Token::RParen,
        Span::new(location, location),
    ))
}

#[test]
fn parser_spans_stop_at_the_partial_statement_before_the_cte_paren() {
    let sql = "WITH a AS (INSERT INTO t(id) VALUES (1) ON CONFLICT (id) DO NOTHING) SELECT 1;";
    let ast = Parser::parse_sql(&PostgresSourceDialect, sql).unwrap();
    let Statement::Query(query) = &ast[0] else {
        panic!("query");
    };
    let cte = &query.with.as_ref().unwrap().cte_tables[0];
    let SetExpr::Insert(statement) = cte.query.body.as_ref() else {
        panic!("insert");
    };
    let locations = Locations::new(sql);
    let statement_end = locations.span(statement.span()).unwrap().end.offset;
    let query_end = locations.span(cte.query.span()).unwrap().end.offset;
    let closing = locations
        .position(cte.closing_paren_token.0.span.start)
        .unwrap()
        .offset;
    assert_eq!(statement_end, query_end);
    assert!(closing > statement_end);
    assert_eq!(&sql[statement_end..closing], ") DO NOTHING");
}

#[test]
fn repair_extends_partial_statement_tokens_through_the_closing_paren() {
    let sql = "WITH a AS (INSERT INTO t(id) VALUES (1) ON CONFLICT (id) DO NOTHING) SELECT 1;";
    let mut facts = project_sql(sql);
    let child = &facts.nested_statements[0];
    assert_eq!(
        child.sql,
        "INSERT INTO t(id) VALUES (1) ON CONFLICT (id) DO NOTHING"
    );
    assert!(child.complete);
    let span = child.span.clone().unwrap();
    let scope = child.query_scope_id;
    let closing = {
        let ast = Parser::parse_sql(&PostgresSourceDialect, sql).unwrap();
        let Statement::Query(query) = &ast[0] else {
            panic!("query");
        };
        query.with.as_ref().unwrap().cte_tables[0]
            .closing_paren_token
            .clone()
    };
    let locations = Locations::new(sql);
    repair(&mut facts, &locations, scope, &closing);
    assert_eq!(facts.nested_statements[0].sql, span_sql(sql, &span));
    assert_eq!(facts.nested_statements[0].span, Some(span));
}

fn span_sql(sql: &str, span: &crate::codebase::postgres::source::PostgresSqlSpan) -> String {
    sql[span.start.offset..span.end.offset].to_owned()
}

#[test]
fn an_unmapped_or_misplaced_closing_paren_drops_truncated_sql() {
    let sql = "WITH a AS (INSERT INTO t(id) VALUES ('abc') ON CONFLICT (id) DO NOTHING) SELECT 1;";
    let locations = Locations::new(sql);
    let mut facts = project_sql(sql);
    let scope = facts.nested_statements[0].query_scope_id;
    let quote = sql.find('\'').unwrap();
    facts.nested_statements[0].span.as_mut().unwrap().end = locations.range(quote, quote).start;
    repair(
        &mut facts,
        &locations,
        scope,
        &rparen_at(&locations, quote + 2),
    );
    let child = &facts.nested_statements[0];
    assert!(!facts.complete);
    assert!(!child.complete);
    assert!(child.sql.is_empty());
    assert!(child.span.is_none());
    assert_eq!(child.unsupported.len(), 1);
    assert_eq!(child.unsupported[0].reason, BOUNDARY);

    facts.nested_statements[0].span = Some(locations.range(0, 4));
    repair(
        &mut facts,
        &locations,
        scope,
        &AttachedToken(TokenWithSpan::wrap(Token::RParen)),
    );
    assert!(facts.nested_statements[0].sql.is_empty());
    assert_eq!(
        facts.nested_statements[0]
            .unsupported
            .iter()
            .filter(|item| item.reason == BOUNDARY)
            .count(),
        2
    );

    let before = facts.unsupported.len();
    repair(&mut facts, &locations, scope, &rparen_at(&locations, 0));
    assert_eq!(facts.unsupported.len(), before);
}

#[test]
fn a_non_paren_closing_token_is_not_a_statement_boundary() {
    let sql = "WITH a AS (INSERT INTO t(id) VALUES (1)) SELECT 1;";
    let locations = Locations::new(sql);
    let mut facts = project_sql(sql);
    let scope = facts.nested_statements[0].query_scope_id;
    let position = locations.range(sql.len(), sql.len()).start;
    let location =
        sqlparser::tokenizer::Location::new(position.line as u64, position.column as u64);
    repair(
        &mut facts,
        &locations,
        scope,
        &AttachedToken(TokenWithSpan::new(
            Token::LParen,
            Span::new(location, location),
        )),
    );
    assert!(facts.nested_statements[0].sql.is_empty());
    assert!(!facts.nested_statements[0].complete);
}

#[test]
fn significant_end_keeps_internal_trivia_and_rejects_an_open_token() {
    assert_eq!(significant_end("").unwrap(), None);
    assert_eq!(significant_end(" \n/* c */").unwrap(), None);
    assert_eq!(significant_end("\r\n").unwrap(), None);
    assert_eq!(significant_end("é").unwrap(), Some("é".len()));
    let conflict = ") DO NOTHING";
    assert_eq!(significant_end(conflict).unwrap(), Some(conflict.len()));
    let commented = " ) /* a /* b */ c */ DO NOTHING  ";
    let end = significant_end(commented).unwrap().unwrap();
    assert_eq!(&commented[..end], " ) /* a /* b */ c */ DO NOTHING");
    assert!(significant_end("/*").is_err());
    assert!(significant_end("'abc").is_err());
    assert_eq!(significant_end("(").unwrap(), Some(1));
}

fn token_ending(token: Token, column: u64) -> TokenWithSpan {
    let start = sqlparser::tokenizer::Location::new(1, 1);
    let end = sqlparser::tokenizer::Location::new(1, column);
    TokenWithSpan::new(token, Span::new(start, end))
}

#[test]
fn boundary_end_rejects_tokens_that_do_not_cover_the_tail() {
    assert!(boundary_end("ab", &[]).is_err());
    assert!(boundary_end("ab", &[token_ending(Token::RParen, 1)]).is_err());
    assert!(boundary_end(
        "ab",
        &[
            TokenWithSpan::wrap(Token::RParen),
            token_ending(Token::RParen, 3),
        ],
    )
    .is_err());
    let end = boundary_end(
        "ab",
        &[token_ending(Token::EOF, 1), token_ending(Token::RParen, 3)],
    )
    .unwrap();
    assert_eq!(end, Some(2));
}

#[test]
fn a_span_outside_the_closing_paren_is_not_a_complete_slice() {
    let sql = "WITH a AS (INSERT INTO t(id) VALUES (1)) SELECT 1;";
    let locations = Locations::new(sql);
    let mut facts = project_sql(sql);
    let scope = facts.nested_statements[0].query_scope_id;
    let start = facts.nested_statements[0]
        .span
        .as_ref()
        .unwrap()
        .start
        .offset;
    assert!(start > 0);
    repair(
        &mut facts,
        &locations,
        scope,
        &rparen_at(&locations, start - 1),
    );
    assert!(facts.nested_statements[0].sql.is_empty());
    assert!(!facts.nested_statements[0].complete);

    let mut facts = project_sql(sql);
    let scope = facts.nested_statements[0].query_scope_id;
    facts.nested_statements[0].span = Some(locations.range(0, sql.len()));
    repair(&mut facts, &locations, scope, &rparen_at(&locations, 4));
    assert!(facts.nested_statements[0].sql.is_empty());
    assert!(!facts.complete);
}
