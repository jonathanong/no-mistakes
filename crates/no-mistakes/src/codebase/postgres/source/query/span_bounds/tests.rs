use super::*;
use crate::codebase::postgres::source::dialect::PostgresSourceDialect;
use sqlparser::ast::{SetExpr, Statement};
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

#[test]
fn insert_source_does_not_invent_a_span_from_missing_prepared_tokens() {
    let sql =
        crate::codebase::postgres::source::tests::fixture("query-insert-source-call-spans.sql");
    let locations = Locations::new(&sql);
    let ast = Parser::parse_sql(&PostgresSourceDialect, &sql).unwrap();
    let Statement::Query(root) = &ast[0] else {
        panic!("outer query");
    };
    let SetExpr::Insert(Statement::Insert(insert)) =
        &*root.with.as_ref().unwrap().cte_tables[0].query.body
    else {
        panic!("INSERT CTE");
    };
    let source = insert.source.as_deref().expect("INSERT SELECT source");
    let mut tokens = Vec::new();
    Tokenizer::new(&PostgresSourceDialect, &sql)
        .tokenize_with_location_into_buf(&mut tokens)
        .unwrap();
    let bounds = QuerySpanBounds::new(&tokens, &locations);
    // The helper may receive a failed source location or an incomplete token
    // projection; neither permits a guessed source span.
    let unrelated_locations = Locations::new("");
    assert!(bounds
        .insert_source(source, sql.len(), None, None, &unrelated_locations)
        .is_none());
    assert!(QuerySpanBounds::new(&[], &locations)
        .insert_source(source, sql.len(), None, None, &locations)
        .is_none());
}

#[test]
fn incomplete_set_tokens_fall_back_without_claiming_branch_spans() {
    let sql = crate::codebase::postgres::source::tests::fixture("query-set-branch-spans.sql");
    let locations = Locations::new(&sql);
    let mut tokens = Vec::new();
    Tokenizer::new(&PostgresSourceDialect, &sql)
        .tokenize_with_location_into_buf(&mut tokens)
        .unwrap();
    let ast = Parser::parse_sql(&PostgresSourceDialect, &sql).unwrap();
    let Statement::Query(query) = &ast[0] else {
        panic!("set-operation query");
    };
    let SetExpr::SetOperation { left, right, .. } = query.body.as_ref() else {
        panic!("set-operation body");
    };
    let first_end = tokens
        .iter()
        .position(|token| token.token == Token::SemiColon)
        .unwrap();
    let first = &tokens[..first_end];
    let separator = first
        .iter()
        .position(|token| matches!(&token.token, Token::Word(word) if word.value == "UNION"))
        .unwrap();
    let parent = locations.span(query.span()).unwrap();
    let bounds = QuerySpanBounds::new(first, &locations);
    let body_end = bounds.body_end(query, Some(&parent), &locations);
    assert!(body_end.is_some());
    assert!(QuerySpanBounds::new(&[], &locations)
        .query(query, true, &locations)
        .is_none());
    assert!(bounds.query(query, false, &locations).is_none());
    assert!(bounds.clause_before(&[], 0, 0).is_none());
    // An unmatched close in incomplete prepared tokens has no opening owner.
    let orphan_close = first
        .iter()
        .find(|token| token.token == Token::RParen)
        .unwrap();
    assert!(
        QuerySpanBounds::new(std::slice::from_ref(orphan_close), &locations)
            .query(query, false, &locations)
            .is_none()
    );

    assert_eq!(
        bounds.set_branches(left, right, None, body_end, &locations),
        (None, None)
    );
    assert_eq!(
        bounds.set_branches(left, right, Some(&parent), None, &locations),
        (None, None)
    );

    // A missing separator or missing left operand must fall back to the AST
    // spans rather than manufacture a new branch boundary.
    let no_separator = QuerySpanBounds::new(&first[..separator], &locations);
    assert_eq!(
        no_separator.set_branches(left, right, Some(&parent), body_end, &locations),
        (None, None)
    );
    let no_left = QuerySpanBounds::new(&first[separator..], &locations);
    assert_eq!(
        no_left.set_branches(left, right, Some(&parent), body_end, &locations),
        (None, None)
    );

    let right_start = locations.position(right.span().start).unwrap().offset;
    assert_eq!(
        bounds.set_branches(
            left,
            right,
            Some(&parent),
            Some(right_start - 1),
            &locations
        ),
        (None, None)
    );
}
