use crate::codebase::postgres::{extract_migration_facts, parse};
use sqlparser::{dialect::PostgreSqlDialect, tokenizer::Tokenizer};

const VALID: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/postgres-facts/source/create-utility/valid.sql"
));
const MALFORMED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/postgres-facts/source/create-utility/malformed.sql"
));

#[test]
fn complete_create_utilities_parse_or_remain_recognized_unsupported() {
    for sql in VALID.lines() {
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, sql.trim_end_matches(';'))
            .tokenize_with_location()
            .unwrap();
        assert!(super::super::valid(&tokens), "{sql}");
        let facts = extract_migration_facts(sql);
        assert!(
            facts
                .unanalyzable_statements
                .iter()
                .all(|failure| failure.valid_unsupported),
            "{sql}: {:?}",
            facts.unanalyzable_statements
        );
    }
}

#[test]
fn malformed_create_utility_neighbors_remain_failures() {
    for sql in MALFORMED.lines() {
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, sql.trim_end_matches(';'))
            .tokenize_with_location()
            .unwrap();
        assert!(!super::super::valid(&tokens), "{sql}");
        let facts = extract_migration_facts(sql);
        assert!(!facts.unanalyzable_statements.is_empty(), "{sql}");
        assert!(
            facts
                .unanalyzable_statements
                .iter()
                .all(|failure| !failure.valid_unsupported),
            "{sql}: {:?}",
            facts.unanalyzable_statements
        );
    }
}

#[test]
fn recognized_create_utilities_keep_original_tokens_and_heads() {
    for sql in VALID.lines().filter(|sql| sql.starts_with("CrEaTe")) {
        let statements = parse::parse_postgres_sql_with_function_sources(sql);
        assert_eq!(statements.len(), 1, "{sql}");
        let statement = &statements[0];
        let failure = statement.recovery_failure.as_ref().unwrap();
        assert!(failure.valid_unsupported);
        assert!(failure.head.starts_with("CrEaTe"), "{:?}", failure);
        let retained = statement
            .chunk_tokens
            .as_ref()
            .unwrap()
            .iter()
            .map(|token| token.token.to_string())
            .collect::<String>();
        assert_eq!(retained, sql.trim_end_matches(';'));
        assert_eq!((failure.line, failure.column), (1, 1));
    }
}
