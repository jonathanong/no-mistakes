use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

/// Rules that scan executor calls and so require an executor selection.
const EXECUTOR_RULES: &[&str] = &[
    POSTGRES_BOUNDED_STATEMENTS,
    POSTGRES_EXPLICIT_COLUMNS,
    POSTGRES_GENERATED_COLUMN_PREDICATES,
    POSTGRES_NO_GENERATED_COLUMN_WRITES,
    POSTGRES_REQUIRED_PREDICATES,
    POSTGRES_SQL_SHAPE_POLICY,
    POSTGRES_NO_OFFSET,
    POSTGRES_CONFLICT_ORDERING,
    POSTGRES_LOCK_ORDERING,
    POSTGRES_IDEMPOTENT_INSERT,
    POSTGRES_REQUIRE_QUERY_ANNOTATION,
];

/// The executor rules that also load a schema catalog.
const SCHEMA_CATALOG_EXECUTOR_RULES: &[&str] = &[
    POSTGRES_BOUNDED_STATEMENTS,
    POSTGRES_EXPLICIT_COLUMNS,
    POSTGRES_CONFLICT_ORDERING,
];

/// The all-rules dispatch config entry for an executor rule. `executorNames: []`
/// keeps those tests on SQL files and native SQL, since an executor rule with no
/// executor selection is a configuration error.
pub(super) fn config_entry(id: &str) -> Option<String> {
    if !EXECUTOR_RULES.contains(&id) {
        return None;
    }
    let catalog = if SCHEMA_CATALOG_EXECUTOR_RULES.contains(&id) {
        "      schemaCatalogPath: schema.json\n"
    } else {
        ""
    };
    Some(format!(
        "  - rule: {id}\n    scope: repository\n    options:\n{catalog}      executorNames: []\n"
    ))
}

/// Run one executor rule through the real filesystem dispatch. The bounded
/// statements fixture owns a schema catalog, which some of these rules require.
fn run(rule: &str, options: &str) -> anyhow::Result<Vec<RuleFinding>> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/fixture");
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: rule.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(&format!("schemaCatalogPath: schema.json\n{options}"))
                .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    run_filesystem_rules_with_config(&root, &config, &[root.join("schema.json")])
}

#[test]
fn every_executor_rule_rejects_an_unselected_executor_in_dispatch() {
    for rule in EXECUTOR_RULES {
        // A blank module is as absent as no option at all.
        for options in ["", "importSpecifier: ''"] {
            let error = run(rule, options).expect_err("no executor selection must be rejected");
            assert!(
                error.to_string().starts_with(&format!(
                    "{rule} option importSpecifier: set importSpecifier (or executorNames) to select executor calls"
                )),
                "{rule}: {options}: {error:#}"
            );
        }
    }
}

#[test]
fn every_executor_rule_accepts_a_module_or_names_or_the_empty_opt_out() {
    for rule in EXECUTOR_RULES {
        for options in [
            "executorNames: []",
            "importSpecifier: '@example/db'",
            "executorNames: [query]",
        ] {
            let findings =
                run(rule, options).unwrap_or_else(|error| panic!("{rule}: {options}: {error:#}"));
            assert!(findings.is_empty(), "{rule}: {options}: {findings:?}");
        }
    }
}
