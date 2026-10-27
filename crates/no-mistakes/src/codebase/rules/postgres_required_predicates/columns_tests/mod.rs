// Output cases moved to cli_postgres_snapshot_required_predicates.
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

mod catalog;
mod options;

pub(super) fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-required-predicates/fixture")
            .join(name),
    )
}

pub(super) fn config_yaml(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: crate::codebase::postgres::tests::fixture_rule_options(yaml),
            ..Default::default()
        }],
        ..Default::default()
    }
}

pub(super) const COLUMNS: &str = r#"
sqlInclude: ["sql/**/*.sql"]
schemaCatalogPath: schema.json
partitionKeys: require
partitionKeyExemptions: [{table: bare, reason: legacy incomplete snapshot fixture}]
relations:
  - table: orders
    requireColumns: [account_id]
"#;

pub(super) fn messages(findings: &[RuleFinding]) -> String {
    findings
        .iter()
        .map(|finding| finding.message.clone())
        .collect::<Vec<_>>()
        .join("\n")
}
