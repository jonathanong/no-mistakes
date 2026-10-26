use super::*;

/// Write a minimal `.no-mistakes.yml` that enables all the given rule IDs.
/// `production-dependency-declarations` has no default `workspaceRoots` (it
/// is a required option), so it gets an explicit value here to stay in scope
/// for these "every rule dispatches cleanly" tests.
pub(super) fn write_config(dir: &std::path::Path, rules: &[&str]) -> std::path::PathBuf {
    let rule_entries: String = rules
        .iter()
        .map(|id| {
            let executor_option = if crate::codebase::postgres::PREPARED_EMBEDDED_SQL_RULE_IDS.contains(id)
                || matches!(*id, POSTGRES_IDEMPOTENT_INSERT | POSTGRES_REQUIRE_QUERY_ANNOTATION)
            { "      executorNames: []\n" } else { "" };
            if *id == POSTGRES_NO_TEST_STATE_MUTATION {
                format!(
                    "  - rule: {id}\n    scope: repository\n    options:\n{}",
                    include_str!("../../../../../../../fixtures/rules/filesystem-dispatch/all-rules/test-state-options.yml")
                )
            } else if *id == PRODUCTION_DEPENDENCY_DECLARATIONS {
                format!(
                    "  - rule: {id}\n    scope: repository\n    options:\n      \
                     workspaceRoots: [\".\"]\n"
                )
            } else if *id == FORBIDDEN_LINT_SUPPRESSIONS {
                format!(
                    "  - rule: {id}\n    scope: repository\n    options:\n{}",
                    include_str!(
                        "../../../../../../../fixtures/rules/filesystem-dispatch/all-rules/lint-suppression-options.yml"
                    )
                )
            } else if *id == POSTGRES_STATUS_WITH_LIFECYCLE_TIMESTAMPS {
                format!(
                    "  - rule: {id}\n    scope: repository\n    options:\n{}",
                    include_str!(
                        "../../../../../../../fixtures/rules/filesystem-dispatch/all-rules/status-lifecycle-options.yml"
                    )
                )
            } else if *id == POSTGRES_KEY_COLUMN_TYPES {
                format!(
                    "  - rule: {id}\n    scope: repository\n    options:\n      schemaCatalogPath: schema.json\n      allowedTypes: [uuid]\n"
                )
            } else if matches!(*id,
                POSTGRES_CONFLICT_ORDERING | POSTGRES_COLUMN_REQUIRES_TRIGGER
                | POSTGRES_REQUIRED_COMMENTS | POSTGRES_DUPLICATE_FUNCTION_BODY
                | POSTGRES_TABLE_SHAPE | POSTGRES_OBJECT_NAMING | POSTGRES_COLUMN_NAMING
                | POSTGRES_FINITE_TEXT_COLUMNS | POSTGRES_ARRAY_COLUMNS
                | POSTGRES_EXPLICIT_COLUMNS | POSTGRES_BOUNDED_STATEMENTS
                | POSTGRES_NO_TRANSFORMED_UUID_PREDICATES)
            {
                format!(
                    "  - rule: {id}\n    scope: repository\n    options:\n      schemaCatalogPath: schema.json\n{executor_option}"
                )
            } else if !executor_option.is_empty() {
                format!("  - rule: {id}\n    scope: repository\n    options:\n{executor_option}")
            } else {
                format!("  - rule: {id}\n    scope: repository\n")
            }
        })
        .collect();
    let yaml = format!("rules:\n{rule_entries}");
    let config_path = dir.join(".no-mistakes.yml");
    std::fs::write(&config_path, yaml).unwrap();
    config_path
}

pub(super) fn all_rules_fixture() -> tempfile::TempDir {
    let source = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/filesystem-dispatch/all-rules");
    crate::test_support::materialize_saved_fixture(&source)
}
