use super::super::{
    FORBIDDEN_LINT_SUPPRESSIONS, POSTGRES_KEY_COLUMN_TYPES, POSTGRES_NO_TEST_STATE_MUTATION,
    POSTGRES_STATUS_WITH_LIFECYCLE_TIMESTAMPS, PRODUCTION_DEPENDENCY_DECLARATIONS,
};
use crate::codebase::postgres::{PREPARED_EMBEDDED_SQL_RULE_IDS, SCHEMA_CATALOG_RULE_IDS};

/// Copy the saved schema and option snippets. The rule list itself is rendered
/// from the registry so a newly merged filesystem rule stays on the config.
pub(super) fn all_rules_fixture() -> tempfile::TempDir {
    let source = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/filesystem-dispatch/all-rules");
    crate::test_support::materialize_saved_fixture(&source)
}

pub(super) fn prepared_rules(rules: &[&str]) -> (tempfile::TempDir, std::path::PathBuf) {
    let fixture = all_rules_fixture();
    let config_path = write_config(fixture.path(), rules);
    let loaded = crate::config::v2::load_v2_config(fixture.path(), Some(&config_path)).unwrap();
    let loaded_ids: Vec<_> = loaded.rules.iter().map(|rule| rule.rule.as_str()).collect();
    assert_eq!(
        loaded_ids, rules,
        "rendered dispatch config must enable the requested registry ids"
    );
    (fixture, config_path)
}

/// `production-dependency-declarations` requires `workspaceRoots`. Schema-catalog
/// and embedded-SQL rules take the options those registries already declare.
fn options_block(id: &str) -> String {
    if id == POSTGRES_NO_TEST_STATE_MUTATION {
        return include_str!(
            "../../../../../../../fixtures/rules/filesystem-dispatch/all-rules/test-state-options.yml"
        )
        .to_string();
    }
    if id == PRODUCTION_DEPENDENCY_DECLARATIONS {
        return "      workspaceRoots: [\".\"]\n".to_string();
    }
    if id == FORBIDDEN_LINT_SUPPRESSIONS {
        return include_str!(
            "../../../../../../../fixtures/rules/filesystem-dispatch/all-rules/lint-suppression-options.yml"
        )
        .to_string();
    }
    if id == POSTGRES_STATUS_WITH_LIFECYCLE_TIMESTAMPS {
        return include_str!(
            "../../../../../../../fixtures/rules/filesystem-dispatch/all-rules/status-lifecycle-options.yml"
        )
        .to_string();
    }

    let mut options = String::new();
    if SCHEMA_CATALOG_RULE_IDS.contains(&id) {
        options.push_str("      schemaCatalogPath: schema.json\n");
    }
    if id == POSTGRES_KEY_COLUMN_TYPES {
        options.push_str("      allowedTypes: [uuid]\n");
    }
    if PREPARED_EMBEDDED_SQL_RULE_IDS.contains(&id) {
        options.push_str("      executorNames: []\n");
    }
    options
}

fn write_config(dir: &std::path::Path, rules: &[&str]) -> std::path::PathBuf {
    let mut yaml = String::from("rules:\n");
    for id in rules {
        let options = options_block(id);
        yaml.push_str(&format!("  - rule: {id}\n    scope: repository\n"));
        if !options.is_empty() {
            yaml.push_str(&format!("    options:\n{options}"));
        }
    }
    let config_path = dir.join(".no-mistakes.yml");
    std::fs::write(&config_path, yaml).unwrap();
    config_path
}
