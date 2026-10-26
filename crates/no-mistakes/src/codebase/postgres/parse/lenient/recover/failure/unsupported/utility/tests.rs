use sqlparser::{dialect::PostgreSqlDialect, tokenizer::Tokenizer};

#[test]
fn malformed_utility_guards_reject_even_when_native_parser_accepts_a_prefix() {
    // sqlparser already admits this malformed EXPLAIN through its native path.
    // The fallback must still reject it rather than widen that admission.
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/regress-utilities/malformed.sql"
    ))
    .to_owned()
        + include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/postgres-facts/source/regress-utilities/guard-malformed.sql"
        ));
    for sql in fixture.lines() {
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, sql.trim_end_matches(';'))
            .tokenize_with_location()
            .unwrap();
        assert!(!super::valid(&tokens), "{sql}");
    }
}

#[test]
fn postgres_regression_utility_witnesses_are_complete() {
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/regress-utilities/corpus-valid.sql"
    ));
    for sql in fixture.lines().filter(|line| !line.starts_with("--")) {
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, sql.trim_end_matches(';'))
            .tokenize_with_location()
            .unwrap();
        assert!(super::valid(&tokens), "{sql}");
        let facts = crate::codebase::postgres::extract_migration_facts(sql);
        assert!(
            facts
                .unanalyzable_statements
                .iter()
                .all(|item| item.valid_unsupported),
            "{sql}: {:?}",
            facts.unanalyzable_statements
        );
    }
}

#[test]
fn malformed_regression_utility_neighbors_are_rejected() {
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/regress-utilities/corpus-invalid.sql"
    ));
    for sql in fixture.lines().filter(|line| !line.starts_with("--")) {
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, sql.trim_end_matches(';'))
            .tokenize_with_location()
            .unwrap();
        assert!(!super::valid(&tokens), "{sql}");
    }
}
