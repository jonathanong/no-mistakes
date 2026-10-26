use crate::codebase::postgres::migration::extract_migration_facts;
const VALID: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/postgres-facts/source/ddl-docs-survey/metadata-valid.sql"
));
const INVALID: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/postgres-facts/source/ddl-docs-survey/metadata-invalid.sql"
));
#[test]
fn documented_metadata_remains_explicitly_unsupported_without_schema_false_alarms() {
    for sql in VALID.lines().filter(|sql| !sql.starts_with("--")) {
        let facts = extract_migration_facts(sql);
        assert_eq!(facts.unanalyzable_statements.len(), 1, "{sql}");
        assert!(facts.unanalyzable_statements[0].valid_unsupported, "{sql}");
    }
}
#[test]
fn malformed_metadata_does_not_inherit_validity_from_its_prefix() {
    for sql in INVALID.lines() {
        let facts = extract_migration_facts(sql);
        assert!(
            facts
                .unanalyzable_statements
                .iter()
                .any(|f| !f.valid_unsupported),
            "{sql}"
        );
    }
}

#[test]
fn metadata_recognizer_preserves_internal_terminators() {
    use sqlparser::{dialect::PostgreSqlDialect, tokenizer::Tokenizer};
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/metadata-token-invalid.sql"
    ));
    for line in sql.lines() {
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, line)
            .tokenize_with_location()
            .unwrap();
        assert!(!super::valid(&tokens), "{line}");
    }
}

#[test]
fn catalog_family_examples_are_explicitly_recognized() {
    let valid = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/catalog-valid.sql"
    ));
    for sql in valid.lines() {
        let facts = extract_migration_facts(sql);
        assert!(
            facts
                .unanalyzable_statements
                .iter()
                .all(|f| f.valid_unsupported),
            "{sql}"
        );
    }
    let invalid = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/catalog-invalid.sql"
    ));
    for sql in invalid.lines() {
        let facts = extract_migration_facts(sql);
        assert!(
            facts
                .unanalyzable_statements
                .iter()
                .any(|f| !f.valid_unsupported),
            "{sql}"
        );
    }
}

#[test]
fn postgres_identifier_classes_and_aggregate_signatures_remain_distinct() {
    let valid = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/keyword-classes-valid.sql"
    ));
    for sql in valid.lines() {
        let facts = extract_migration_facts(sql);
        assert!(!facts.unanalyzable_statements.is_empty(), "{sql}");
        assert!(
            facts
                .unanalyzable_statements
                .iter()
                .all(|f| f.valid_unsupported),
            "{sql}"
        );
    }
    let invalid = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/keyword-classes-invalid.sql"
    ));
    for sql in invalid.lines() {
        let facts = extract_migration_facts(sql);
        assert!(
            facts
                .unanalyzable_statements
                .iter()
                .any(|f| !f.valid_unsupported),
            "{sql}"
        );
    }
}

#[test]
fn numeric_modifiers_follow_postgres_grammar() {
    let valid = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/numeric-modifiers-valid.sql"
    ));
    let invalid = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/numeric-modifiers-invalid.sql"
    ));
    for sql in valid.lines() {
        let facts = extract_migration_facts(sql);
        assert!(
            facts
                .unanalyzable_statements
                .iter()
                .all(|f| f.valid_unsupported),
            "{sql}"
        );
    }
    for sql in invalid.lines() {
        let facts = extract_migration_facts(sql);
        assert!(
            facts
                .unanalyzable_statements
                .iter()
                .any(|f| !f.valid_unsupported),
            "{sql}"
        );
    }
}

#[test]
fn metadata_clause_boundaries_follow_postgres() {
    let valid = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/branches-valid.sql"
    ));
    let invalid = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/branches-invalid.sql"
    ));
    for (sql, valid) in valid
        .lines()
        .map(|sql| (sql, true))
        .chain(invalid.lines().map(|sql| (sql, false)))
    {
        let failures = extract_migration_facts(sql).unanalyzable_statements;
        assert_eq!(
            failures.iter().all(|failure| failure.valid_unsupported),
            valid,
            "{sql}: {failures:?}"
        );
        if !valid {
            assert!(!failures.is_empty(), "{sql}");
        }
    }
}

#[test]
fn regress_ddl_rejects_saved_malformed_branches() {
    use super::{regress_alter, regress_create, regress_objects, Cursor};
    use sqlparser::{dialect::PostgreSqlDialect, parser::Parser, tokenizer::Tokenizer};

    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/regress-branches-invalid.sql"
    ));
    for sql in fixture.lines().filter(|line| !line.starts_with("--")) {
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, sql)
            .tokenize_with_location()
            .unwrap();
        for recognizer in [
            regress_alter::parse as fn(&mut Cursor<'_>) -> bool,
            regress_objects::parse,
            regress_create::parse,
        ] {
            let mut cursor = Cursor(
                Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(tokens.clone()),
            );
            assert!(!recognizer(&mut cursor), "{sql}");
        }
        assert!(!regress_create::valid_native(&tokens), "{sql}");
        let failures = extract_migration_facts(sql).unanalyzable_statements;
        assert!(
            failures.iter().any(|failure| !failure.valid_unsupported),
            "{sql}: {failures:?}"
        );
    }
}

#[test]
fn regress_alter_accepts_saved_option_and_partition_forms() {
    use super::{regress_alter, Cursor};
    use sqlparser::{dialect::PostgreSqlDialect, parser::Parser, tokenizer::Tokenizer};

    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-docs-survey/regress-branches-valid.sql"
    ));
    for sql in fixture.lines().filter(|line| !line.starts_with("--")) {
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, sql)
            .tokenize_with_location()
            .unwrap();
        let mut cursor =
            Cursor(Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(tokens));
        assert!(regress_alter::parse(&mut cursor), "{sql}");
        let failures = extract_migration_facts(sql).unanalyzable_statements;
        assert!(
            failures.iter().all(|failure| failure.valid_unsupported),
            "{sql}: {failures:?}"
        );
    }
}
