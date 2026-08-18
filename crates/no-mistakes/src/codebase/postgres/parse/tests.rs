use super::{parse_postgres_sql, PostgresParseError};

#[test]
fn parse_postgres_sql_accepts_create_table() {
    let statements = parse_postgres_sql("CREATE TABLE t (id int);").expect("parse");
    assert_eq!(statements.len(), 1);
}

#[test]
fn parse_postgres_sql_returns_error_for_unparseable_sql() {
    let error = parse_postgres_sql("CREATE TABLE broken (").expect_err("unparseable");
    assert!(!error.message.is_empty());
    assert!(error.to_string().contains(&error.message));
}

#[test]
fn parse_error_from_parser_error_preserves_message() {
    let error = PostgresParseError::from(sqlparser::parser::ParserError::ParserError(
        "boom".to_string(),
    ));
    assert_eq!(error.message, "boom");
    assert_eq!(error.to_string(), "boom");
}

#[test]
fn empty_sql_is_not_a_panic() {
    let statements = parse_postgres_sql("").expect("empty sql");
    assert!(statements.is_empty());
}

#[test]
fn parse_error_from_tokenizer_error_preserves_message() {
    let error = PostgresParseError::from(sqlparser::parser::ParserError::TokenizerError(
        "bad token".to_string(),
    ));
    assert_eq!(error.message, "bad token");
}

#[test]
fn parse_error_from_recursion_limit_is_not_empty() {
    let error = PostgresParseError::from(sqlparser::parser::ParserError::RecursionLimitExceeded);
    assert!(!error.message.is_empty());
}
