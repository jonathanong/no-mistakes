use crate::codebase::postgres::migration::extract_migration_facts;
use sqlparser::{dialect::PostgreSqlDialect, tokenizer::Tokenizer};

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source/ddl-compat/body-recovery")
            .join(format!("issue-2810-{name}.sql")),
    )
    .unwrap()
}

#[test]
fn schema_utilities_and_expression_range_bounds_are_complete_inside_bodies() {
    let sql = fixture("valid");
    let facts = extract_migration_facts(&sql);
    assert!(facts.unanalyzable_statements.is_empty(), "{facts:#?}");
    assert!(facts
        .tables
        .iter()
        .any(|table| table.table_name == "following_ranges"));
}

#[test]
fn compact_range_delimiters_preserve_adjacency_and_validate_both_bounds() {
    for (name, expected) in [("controls", true), ("malformed-controls", false)] {
        let sql = fixture(name);
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, &sql)
            .tokenize_with_location()
            .unwrap();
        for chunk in tokens.split(|token| token.token == sqlparser::tokenizer::Token::SemiColon) {
            if chunk
                .iter()
                .all(|token| matches!(token.token, sqlparser::tokenizer::Token::Whitespace(_)))
            {
                continue;
            }
            assert_eq!(
                super::super::BodyState::default().control_chunk(chunk),
                expected,
                "{name}: {chunk:#?}"
            );
        }
    }
}

#[test]
fn malformed_utilities_and_range_bounds_remain_reported_with_original_heads() {
    let sql = fixture("malformed");
    let facts = extract_migration_facts(&sql);
    assert_eq!(facts.unanalyzable_statements.len(), 3, "{facts:#?}");
    let range = facts
        .unanalyzable_statements
        .iter()
        .find(|failure| failure.head.contains("FOR"))
        .unwrap();
    assert!(range.head.contains("1..array_length("), "{range:#?}");
    assert!(!range.head.contains("1. ."), "{range:#?}");
    assert_eq!(
        range.line,
        sql.lines()
            .position(|line| line.starts_with("FOR"))
            .unwrap()
            + 1
    );
}
