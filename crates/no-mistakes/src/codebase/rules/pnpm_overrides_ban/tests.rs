use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

// Output cases moved to the cli_pkgcfg_snapshot modules.

fn config() -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        ..Default::default()
    });
    config
}

fn fixture_root(scenario: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/pnpm-overrides-ban/fixture")
            .join(scenario),
    )
}

#[test]
fn reports_unreadable_workspace_and_invalid_json() {
    let root = fixture_root("unanalyzable");
    let files = vec![
        root.join("pnpm-workspace.yaml"),
        root.join("packages/broken/package.json"),
    ];
    let findings = check_with_files(&root, &config(), &files).unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some("unanalyzable")));
}

#[test]
fn key_line_helpers_cover_fallbacks_and_quoted_yaml() {
    use super::keys::{
        json_key_line, json_nested_key_line, json_quoted_key_after, yaml_top_level_key_line,
    };
    assert_eq!(
        yaml_top_level_key_line("packages:\n  overrides: 1\n", "overrides"),
        1
    );
    assert_eq!(yaml_top_level_key_line("\toverrides: 1\n", "overrides"), 1);
    assert_eq!(
        yaml_top_level_key_line("\"overrides\": 1\n", "overrides"),
        1
    );
    assert_eq!(yaml_top_level_key_line("overrides:\n", "overrides"), 1);
    assert_eq!(json_key_line("{\"name\":\"app\"}\n", "overrides"), 1);
    assert_eq!(
        json_nested_key_line("{\"name\":\"app\"}\n", "pnpm", "overrides"),
        1
    );
    assert_eq!(
        json_nested_key_line("{\"pnpm\":{}}\n", "pnpm", "overrides"),
        1
    );
    assert_eq!(json_quoted_key_after("{}", "\"x\"", 99), None);
}

#[path = "tests/include_inputs.rs"]
mod include_inputs;
