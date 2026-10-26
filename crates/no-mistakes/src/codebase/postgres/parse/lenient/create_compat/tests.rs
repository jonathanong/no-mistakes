use sqlparser::{dialect::PostgreSqlDialect, tokenizer::Tokenizer};

fn tokenize(sql: &str) -> Vec<sqlparser::tokenizer::TokenWithSpan> {
    Tokenizer::new(&PostgreSqlDialect {}, sql.trim_end_matches(';'))
        .tokenize_with_location()
        .unwrap()
}

#[test]
fn original_create_guards_reject_bad_partition_names_and_lists() {
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/regress-create-keywords-invalid.sql"
    ));
    for sql in fixture.lines().filter(|line| line.contains("PARTITION OF")) {
        assert!(!super::original::valid(Some(&tokenize(sql))), "{sql}");
    }
}

#[test]
fn partition_rewrite_keeps_malformed_column_lists_intact() {
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/regress-create-keywords-invalid.sql"
    ));
    for sql in fixture.lines().filter(|line| line.contains("PARTITION OF")) {
        let mut tokens = tokenize(sql);
        let code = super::significant(&tokens);
        assert!(
            !super::clauses::partition_columns(&mut tokens, &code),
            "{sql}"
        );
    }
}
