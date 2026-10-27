use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

// Output cases moved to the cli_pkgcfg_snapshot modules.

const OPTIONS: &str = r#"
sourceFile: .mise.toml
sourceKey: tools.aqua:lycheeverse/lychee
anchors:
  - file: .github/actions/setup-lychee/action.yml
    pattern: 'LYCHEE_VERSION:\s*(\d+\.\d+\.\d+)'
    label: lychee
"#;

const PAIR: [&str; 2] = [".mise.toml", ".github/actions/setup-lychee/action.yml"];

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/version-pin-consistency/fixture")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(yaml).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn run(root: &Path, yaml: &str, files: &[&str]) -> Vec<RuleFinding> {
    let files: Vec<PathBuf> = files.iter().map(|file| root.join(file)).collect();
    check_with_files(root, &config(yaml), &files).unwrap()
}

#[test]
fn skip_when_configured_files_are_not_tracked() {
    let findings = run(&fixture("pass"), OPTIONS, &["README.md"]);
    assert!(findings.is_empty(), "{findings:?}");
}

mod extra;
mod guard;
mod key_line;

mod unavailable_source;
