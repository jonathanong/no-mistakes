use super::{parse_postgres_sql, parse_postgres_sql_lenient, PostgresParseError};

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

#[test]
fn lenient_parse_skips_do_blocks_and_accepts_virtual_generated() {
    let statements = parse_postgres_sql_lenient(
        "DO $$ BEGIN CREATE TYPE t AS ENUM ('a'); END $$;\n\
         CREATE TABLE items (\n\
           id uuid,\n\
           created_at timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp(id)) VIRTUAL\n\
         );",
    );
    assert_eq!(statements.len(), 1, "{statements:#?}");
    assert!(matches!(
        statements[0],
        sqlparser::ast::Statement::CreateTable(_)
    ));
}

#[test]
fn lenient_parse_skips_tokenizer_failures_and_empty_chunks() {
    assert!(parse_postgres_sql_lenient("CREATE TABLE t (id text, note 'unterminated").is_empty());
    assert!(parse_postgres_sql_lenient(";;;").is_empty());
    assert!(!parse_postgres_sql_lenient("GENERATED ALWAYS; CREATE TABLE t (id int);").is_empty());
    let identity =
        parse_postgres_sql_lenient("CREATE TABLE t (id int GENERATED ALWAYS AS IDENTITY);");
    assert_eq!(identity.len(), 1, "{identity:#?}");
}
