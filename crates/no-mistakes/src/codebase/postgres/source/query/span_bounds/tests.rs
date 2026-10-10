use super::*;
use crate::codebase::postgres::source::dialect::PostgresSourceDialect;
use sqlparser::ast::Statement;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Location, Span, Tokenizer};

#[test]
fn incomplete_prepared_tokens_do_not_claim_a_query_boundary() {
    let sql = crate::codebase::postgres::source::tests::fixture("query-readonly-cte-spans.sql");
    let locations = Locations::new(&sql);
    let mut tokens = Vec::new();
    Tokenizer::new(&PostgresSourceDialect, &sql)
        .tokenize_with_location_into_buf(&mut tokens)
        .unwrap();
    let ast = Parser::parse_sql(&PostgresSourceDialect, &sql).unwrap();
    let Statement::Query(root) = &ast[0] else {
        panic!("read-only CTE query");
    };
    let cte = &root.with.as_ref().unwrap().cte_tables[0].query;
    let first_end = tokens
        .iter()
        .position(|token| token.token == Token::SemiColon)
        .unwrap();
    let mut first = tokens[..first_end].to_vec();
    // Invalid prepared locations are ignored, not turned into invented spans.
    let invalid = Location::new(0, 0);
    first.push(TokenWithSpan::new(
        Token::LParen,
        Span::new(invalid, invalid),
    ));
    let bounds = QuerySpanBounds::new(&first, &locations);
    assert!(bounds.query(root, false, &locations).is_none());
    let span = bounds.query(cte, false, &locations).unwrap();
    assert_eq!(locations.slice(&span), "SELECT now()");

    // A missing CTE close cannot be inferred from the function call's close.
    let cte_close = first
        .iter()
        .enumerate()
        .filter(|(_, token)| token.token == Token::RParen)
        .nth(1)
        .unwrap()
        .0;
    let incomplete = QuerySpanBounds::new(&first[..cte_close], &locations);
    assert!(incomplete.query(cte, false, &locations).is_none());

    let Statement::Query(other) = &ast[2] else {
        panic!("later read-only query");
    };
    // A preceding statement is not an opening delimiter for this child query.
    assert!(QuerySpanBounds::new(&tokens, &locations)
        .query(other, false, &locations)
        .is_none());
}
