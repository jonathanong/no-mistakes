use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

pub(super) fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/workflow-topology-policy/fixture")
            .join(name),
    )
}

pub(super) fn topology_fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/workflow-topology")
            .join(name),
    )
}

pub(super) fn run(root: &Path, yaml: &str) -> Vec<RuleFinding> {
    check_with_files(
        root,
        &NoMistakesConfig {
            rules: vec![RuleDef {
                rule: RULE_ID.to_string(),
                scope: Some(RuleScope::Repository),
                options: serde_yaml::from_str(yaml).unwrap(),
                ..Default::default()
            }],
            ..Default::default()
        },
        &[],
    )
    .unwrap()
}
