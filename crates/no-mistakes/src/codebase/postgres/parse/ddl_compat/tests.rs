use super::super::{parse_postgres_sql, parse_postgres_sql_with_function_sources};
use sqlparser::ast::Statement;

const VALID: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/postgres-facts/source/ddl-compat/valid.sql"
));
const INVALID: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/postgres-facts/source/ddl-compat/invalid.sql"
));

#[test]
fn compatibility_ddl_is_complete_in_strict_and_located_analysis() {
    let statements = parse_postgres_sql(VALID).expect("valid PostgreSQL utility DDL");
    assert_eq!(statements.len(), 15);
    assert!(matches!(statements[0], Statement::CreateType { .. }));
    assert!(matches!(statements[1], Statement::CreateTrigger(_)));
    assert!(matches!(statements[2], Statement::CreateTrigger(_)));
    assert!(matches!(statements[3], Statement::Comment { .. }));
    assert!(matches!(statements[6], Statement::CreateIndex(_)));
    let located = parse_postgres_sql_with_function_sources(VALID);
    assert_eq!(located.len(), 15);
    for statement in located {
        assert!(
            statement.recovery_failure.is_none(),
            "{:?}",
            statement.recovery_failure
        );
        assert!(statement.complete);
    }
}

#[test]
fn malformed_compatibility_ddl_is_not_silenced() {
    for sql in INVALID.lines() {
        assert!(
            parse_postgres_sql(sql).is_err(),
            "accepted invalid DDL: {sql}"
        );
        let located = parse_postgres_sql_with_function_sources(sql);
        assert!(
            located
                .iter()
                .any(|statement| statement.recovery_failure.is_some()),
            "missing failure for invalid DDL: {sql}"
        );
    }
}

#[test]
fn schema_projection_has_no_false_uncertainty_and_retains_index_facts() {
    use crate::codebase::postgres::extract_schema_facts;
    use crate::codebase::ts_source::{FileInventory, SourceStore};
    use std::{path::PathBuf, sync::Arc};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/source/ddl-compat");
    let paths = [root.join("valid.sql"), root.join("after-do.sql")];
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&paths)));
    let facts = extract_schema_facts(&root, &sources, &paths).expect("schema projection");
    for file in facts {
        assert!(
            file.unanalyzable_statements.is_empty(),
            "{:?}: {:?}",
            file.path,
            file.unanalyzable_statements
        );
        assert_eq!(file.indexes.len(), 1);
    }
}

#[test]
fn utilities_recover_immediately_after_begin_and_conditional_guards() {
    for (wrapper, sql) in [
        (
            "do",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../fixtures/postgres-facts/source/ddl-compat/guarded-do.sql"
            )),
        ),
        (
            "function",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../fixtures/postgres-facts/source/ddl-compat/guarded-function.sql"
            )),
        ),
    ] {
        let facts = crate::codebase::postgres::migration::extract_migration_facts(sql);
        assert!(
            facts.unanalyzable_statements.is_empty(),
            "{wrapper}: {:?}",
            facts.unanalyzable_statements
        );
        assert_eq!(facts.indexes.len(), 1, "{wrapper}");
        assert!(
            facts
                .declared_identifiers
                .iter()
                .any(|identifier| identifier.name == "state"),
            "{wrapper}: {:?}",
            facts.declared_identifiers
        );
    }
}

#[test]
fn malformed_utility_guards_and_suffixes_keep_source_failures() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-compat/guarded-invalid.sql"
    ));
    let facts = crate::codebase::postgres::migration::extract_migration_facts(sql);
    let lines: Vec<_> = facts
        .unanalyzable_statements
        .iter()
        .map(|failure| failure.line)
        .collect();
    assert_eq!(lines, [1, 2, 3, 4]);
    assert!(facts.indexes.is_empty());
}

#[test]
fn null_metadata_comments_are_complete_without_inventing_values() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-compat/edge-valid.sql"
    ));
    let statements = parse_postgres_sql(sql).expect("NULL removes metadata comments");
    assert_eq!(statements.len(), 2);
    assert!(matches!(
        statements[0],
        Statement::Discard {
            object_type: sqlparser::ast::DiscardObject::PLANS,
        }
    ));
    assert!(matches!(
        statements[1],
        Statement::Comment { comment: None, .. }
    ));
    let located = parse_postgres_sql_with_function_sources(sql);
    assert_eq!(located.len(), 2);
    assert!(located
        .iter()
        .all(|statement| statement.complete && statement.recovery_failure.is_none()));
    let facts = crate::codebase::postgres::migration::extract_migration_facts(sql);
    assert!(facts.unanalyzable_statements.is_empty());
}
