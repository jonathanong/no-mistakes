use super::*;
use crate::codebase::postgres::source::dialect::PostgresSourceDialect;
use sqlparser::ast::Statement;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Location, Span, Tokenizer};

#[test]
fn incomplete_token_boundaries_do_not_escape_their_query_scope() {
    let sql = crate::codebase::postgres::source::tests::fixture("query-readonly-cte-spans.sql");
    let locations = Locations::new(&sql);
    let mut tokens = Vec::new();
    Tokenizer::new(&PostgresSourceDialect, &sql)
        .tokenize_with_location_into_buf(&mut tokens)
        .unwrap();
    let ast = Parser::parse_sql(&PostgresSourceDialect, &sql).unwrap();
    let Statement::Query(query) = &ast[4] else {
        panic!("set-operation query");
    };
    let SetExpr::SetOperation { left, right, .. } = query.body.as_ref() else {
        panic!("set operation");
    };
    let invalid = Location::new(0, 0);
    tokens.push(TokenWithSpan::new(
        Token::LParen,
        Span::new(invalid, invalid),
    ));
    let bounds = QuerySpanBounds::new(&tokens, &locations);
    // A root query passed as a child has no enclosing `(` to claim.
    assert!(bounds.query(query, false, &locations).is_none());
    assert_eq!(
        bounds.set_branches(left, right, None, &locations),
        (None, None)
    );

    let left_start = locations.position(left.span().start).unwrap().offset;
    let right_start = locations.position(right.span().start).unwrap().offset;
    let close = tokens
        .iter()
        .position(|token| {
            token.token == Token::RParen
                && locations
                    .position(token.span.start)
                    .is_some_and(|position| {
                        position.offset > left_start && position.offset < right_start
                    })
        })
        .unwrap();
    tokens.remove(close);
    let incomplete = QuerySpanBounds::new(&tokens, &locations);
    let parent = locations.span(query.span()).unwrap();
    // Simulate a damaged prepared token range: the unmatched opener must not
    // make the branch search consume an unrelated sibling.
    assert!(incomplete
        .set_branches(left, right, Some(&parent), &locations)
        .0
        .is_some());

    let right_open = tokens
        .iter()
        .find(|token| {
            token.token == Token::LParen
                && locations
                    .position(token.span.start)
                    .is_some_and(|position| position.offset > right_start)
        })
        .unwrap();
    let end = locations.position(right_open.span.end).unwrap().offset;
    let short_parent = locations.range(left_start, end);
    let (_, right_span) = bounds.set_branches(left, right, Some(&short_parent), &locations);
    assert!(right_span.unwrap().end.offset <= short_parent.end.offset);
}
