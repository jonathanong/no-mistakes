use crate::codebase::postgres::migration::extract_migration_facts;
use sqlparser::ast::Statement;

const VALID: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/postgres-facts/source/ddl-docs-survey/regress-create-valid.sql"
));

#[test]
fn regression_create_forms_are_parsed_or_recognized() {
    let missing = VALID
        .lines()
        .filter(|sql| {
            extract_migration_facts(sql)
                .unanalyzable_statements
                .iter()
                .any(|f| !f.valid_unsupported)
        })
        .collect::<Vec<_>>();
    assert!(missing.is_empty(), "{missing:#?}");
}

#[test]
fn regress_table_clauses_keep_typed_table_facts() {
    // These clauses do not change the table or constraint identity used by rules.
    for head in [
        "CREATE TABLE cmpart",
        "CREATE TABLE idxpart0",
        "CREATE TABLE gtest_err_8",
        "CREATE TABLE cmdata",
        "CREATE TABLE test_storage",
        "CREATE TEMP TABLE withoutoid",
    ] {
        let sql = VALID.lines().find(|line| line.starts_with(head)).unwrap();
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
fn regression_create_malformed_tails_remain_failures() {
    let invalid = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/regress-create-invalid.sql"
    ));
    for sql in invalid.lines() {
        let facts = extract_migration_facts(sql);
        assert!(
            facts
                .unanalyzable_statements
                .iter()
                .any(|f| !f.valid_unsupported),
            "{sql}: {:?}",
            facts.unanalyzable_statements
        );
    }
}

#[test]
fn regression_create_keyword_classes_and_column_scope() {
    let valid = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/regress-create-keywords-valid.sql"
    ));
    for sql in valid.lines() {
        let facts = extract_migration_facts(sql);
        assert!(
            !facts
                .unanalyzable_statements
                .iter()
                .any(|f| !f.valid_unsupported),
            "{sql}: {:?}",
            facts.unanalyzable_statements
        );
    }
    let invalid = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/regress-create-keywords-invalid.sql"
    ));
    for sql in invalid.lines() {
        let facts = extract_migration_facts(sql);
        assert!(
            facts
                .unanalyzable_statements
                .iter()
                .any(|f| !f.valid_unsupported),
            "{sql}: {:?}",
            facts.unanalyzable_statements
        );
    }
}
