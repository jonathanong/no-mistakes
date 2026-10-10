use super::{tokenize_with_location, Prepared};
use sqlparser::{
    dialect::PostgreSqlDialect,
    tokenizer::{Location, Token, Tokenizer, Whitespace},
};

fn fixtures() -> Vec<&'static str> {
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/operator-comment-boundaries.sql"
    ))
    .lines()
    .collect()
}

#[test]
fn separates_an_adjacent_block_comment_from_a_custom_operator() {
    let sql = fixtures()[0];
    let prepared = Prepared::new(sql);
    let mut tokens = Tokenizer::new(&PostgreSqlDialect {}, prepared.sql())
        .tokenize_with_location()
        .unwrap();
    prepared.restore(&mut tokens);

    assert!(tokens.iter().any(|token| token.token == Token::ShiftLeft));
    assert!(tokens.iter().any(|token| token.token == Token::ShiftRight));
    let comment = tokens
        .iter()
        .find(|token| {
            matches!(
                &token.token,
                Token::Whitespace(Whitespace::MultiLineComment(comment)) if comment == "gap"
            )
        })
        .expect("comment token");
    assert_eq!(comment.span.start, Location { line: 1, column: 6 });
    assert!(tokens.iter().any(|token| matches!(
        &token.token,
        Token::Word(word) if word.value == "FOR"
    )));
}

#[test]
fn removes_every_synthetic_separator_and_restores_following_token_columns() {
    let sql = fixtures()[1];
    let prepared = Prepared::new(sql);
    let mut tokens = Tokenizer::new(&PostgreSqlDialect {}, prepared.sql())
        .tokenize_with_location()
        .unwrap();
    prepared.restore(&mut tokens);

    let comments = tokens
        .iter()
        .filter_map(|token| match &token.token {
            Token::Whitespace(Whitespace::MultiLineComment(comment)) => Some((comment, token)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(comments.len(), 3);
    for (comment, token) in comments {
        let opener = format!("/*{comment}");
        let source_start = sql.find(&opener).expect("comment opener in source");
        let expected_column = sql[..source_start].chars().count() as u64 + 1;
        assert_eq!(token.span.start.column, expected_column);
    }
    let one = tokens
        .iter()
        .find(|token| matches!(&token.token, Token::Number(number, _) if number == "1"))
        .expect("query result literal");
    assert_eq!(one.span.start.column, (sql.chars().count() - 1) as u64);
}

#[test]
fn preserves_comment_like_text_in_opaque_source_regions() {
    for sql in fixtures()[4..8].iter().copied() {
        let prepared = Prepared::new(sql);
        assert_eq!(prepared.sql(), sql, "{sql}");
    }
}

#[test]
fn keeps_nested_comment_and_unicode_locations_aligned() {
    let source = fixtures()[2..4].join("\n");
    let sql = source.as_str();
    let prepared = Prepared::new(sql);
    let mut tokens = Tokenizer::new(&PostgreSqlDialect {}, prepared.sql())
        .tokenize_with_location()
        .unwrap();
    prepared.restore(&mut tokens);

    let comment = tokens
        .iter()
        .find(|token| {
            matches!(
                &token.token,
                Token::Whitespace(Whitespace::MultiLineComment(_))
            )
        })
        .expect("nested block comment token");
    assert_eq!(comment.span.start, Location { line: 1, column: 6 });
    assert_eq!(comment.span.end.line, 2);
    let for_token = tokens
        .iter()
        .find(|token| {
            matches!(
                &token.token,
                Token::Word(word) if word.value == "FOR"
            )
        })
        .expect("FOR token");
    assert_eq!(for_token.span.start.line, 2);
    assert_eq!(
        for_token.span.start.column,
        sql.lines().last().unwrap().chars().count() as u64 - 2
    );
}

#[test]
fn leaves_operators_with_slashes_untouched_when_no_comment_starts() {
    let sql = fixtures()[8];
    let prepared = Prepared::new(sql);
    assert_eq!(prepared.sql(), sql);
}

#[test]
fn unterminated_escape_literal_stops_at_eof() {
    let sql = fixtures()[9];
    let prepared = Prepared::new(sql);
    assert_eq!(prepared.sql(), sql);
    assert!(tokenize_with_location(&PostgreSqlDialect {}, sql).is_err());
}
