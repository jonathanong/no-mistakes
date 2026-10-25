use super::valid;
use sqlparser::{dialect::PostgreSqlDialect, tokenizer::Tokenizer};

fn fixture(name: &str) -> &'static str {
    match name {
        "valid" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/postgres-facts/source/ddl-foreign/valid.sql"
        )),
        _ => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/postgres-facts/source/ddl-foreign/invalid.sql"
        )),
    }
}

fn recognized(sql: &str) -> bool {
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, sql)
        .tokenize_with_location()
        .unwrap()
        .into_iter()
        .filter(|item| item.token != sqlparser::tokenizer::Token::SemiColon)
        .collect::<Vec<_>>();
    valid(&tokens)
}

#[test]
fn complete_foreign_utility_ddl_is_recognized() {
    for sql in fixture("valid").lines().filter(|line| !line.is_empty()) {
        assert!(recognized(sql), "valid PostgreSQL 17 DDL rejected: {sql}");
    }
}

#[test]
fn malformed_foreign_utility_ddl_is_rejected() {
    for sql in fixture("invalid").lines().filter(|line| !line.is_empty()) {
        assert!(!recognized(sql), "malformed DDL accepted: {sql}");
    }
}
