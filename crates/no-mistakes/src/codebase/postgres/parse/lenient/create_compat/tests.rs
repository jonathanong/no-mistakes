use sqlparser::{ast::Statement, dialect::PostgreSqlDialect, tokenizer::Tokenizer};

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

#[test]
fn table_storage_and_access_method_rewrite_together() {
    let valid = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/create-clause-combinations-valid.sql"
    ));
    for sql in valid.lines() {
        let mut tokens = tokenize(sql);
        super::rewrite(&mut tokens);
        let code = super::significant(&tokens);
        assert!(
            code.iter().all(|at| {
                !super::clauses::word(&tokens[*at].token, "STORAGE")
                    && !super::clauses::word(&tokens[*at].token, "USING")
            }),
            "{sql}"
        );
        let parsed =
            crate::codebase::postgres::parse::parse_postgres_sql_with_function_sources(sql);
        assert!(
            parsed
                .iter()
                .any(|item| matches!(item.statement, Statement::CreateTable(_))
                    && item.recovery_failure.is_none()),
            "{sql}: {parsed:#?}"
        );
    }
}

#[test]
fn malformed_table_option_combinations_remain_failures() {
    let invalid = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/create-clause-combinations-invalid.sql"
    ));
    for sql in invalid.lines() {
        let mut tokens = tokenize(sql);
        let original = tokens.clone();
        super::rewrite(&mut tokens);
        assert_eq!(tokens, original, "{sql}");
        let failures = crate::codebase::postgres::migration::extract_migration_facts(sql)
            .unanalyzable_statements;
        assert!(
            failures.iter().any(|failure| !failure.valid_unsupported),
            "{sql}: {failures:#?}"
        );
    }
}
