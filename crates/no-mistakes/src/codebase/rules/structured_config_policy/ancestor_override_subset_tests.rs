// Output cases moved to cli_rra_snapshot_structured_config_policy.
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/structured-config-policy/ancestor-override-subset"),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn inventory(root: &Path, rels: &[&str]) -> Vec<PathBuf> {
    rels.iter().map(|rel| root.join(rel)).collect()
}

fn oxlintrc_yaml() -> &'static str {
    r#"
policies:
  - files: ["**/.oxlintrc.json"]
    when:
      - key: extends
    valueAssertions:
      - kind: ancestor-override-subset
"#
}

fn oxlintrc_inventory(root: &Path) -> Vec<PathBuf> {
    inventory(
        root,
        &[
            ".oxlintrc.json",
            "extra.jsonc",
            "invalid.json",
            "cycle/peer.json",
            "nested/.oxlintrc.json",
            "nested/file.ts",
            "lost/.oxlintrc.json",
            "lost/file.ts",
            "drift/.oxlintrc.json",
            "drift/file.ts",
            "superset/.oxlintrc.json",
            "superset/file.ts",
            "still-matches/.oxlintrc.json",
            "still-matches/file.ts",
            "unmatched/.oxlintrc.json",
            "unmatched/file.ts",
            "standalone/.oxlintrc.json",
            "empty-children/.oxlintrc.json",
            "package-extends/.oxlintrc.json",
            "package-extends/file.ts",
            "string-extends/.oxlintrc.json",
            "string-extends/file.ts",
            "deep/mid/.oxlintrc.json",
            "deep/nested/.oxlintrc.json",
            "deep/nested/file.ts",
            "deep/lost/.oxlintrc.json",
            "deep/lost/file.ts",
            "cycle/.oxlintrc.json",
            "cycle/file.ts",
            "missing/.oxlintrc.json",
            "missing/file.ts",
            "invalid-ancestor/.oxlintrc.json",
            "invalid-ancestor/file.ts",
            "outside/.oxlintrc.json",
            "outside/file.ts",
            "symlink/.oxlintrc.json",
            "symlink/file.ts",
            "multi/.oxlintrc.json",
            "multi/file.ts",
            "multi-ok/.oxlintrc.json",
            "multi-ok/file.ts",
        ],
    )
}

#[test]
fn ancestor_override_subset_flags_lost_and_keeps_valid_nested_configs() {
    let root = fixture_root();
    let findings =
        check_with_files(&root, &config(oxlintrc_yaml()), &oxlintrc_inventory(&root)).unwrap();
    let body = format!("{findings:?}");
    let files: Vec<&str> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();
    for file in [
        "lost/.oxlintrc.json",
        "drift/.oxlintrc.json",
        "deep/lost/.oxlintrc.json",
        "cycle/.oxlintrc.json",
        "missing/.oxlintrc.json",
        "invalid.json",
        "outside/.oxlintrc.json",
        "symlink/.oxlintrc.json",
        "multi/.oxlintrc.json",
    ] {
        assert!(files.contains(&file), "{body}");
    }
    for file in [
        "nested/.oxlintrc.json",
        "superset/.oxlintrc.json",
        "still-matches/.oxlintrc.json",
        "unmatched/.oxlintrc.json",
        "standalone/.oxlintrc.json",
        "empty-children/.oxlintrc.json",
        "package-extends/.oxlintrc.json",
        "string-extends/.oxlintrc.json",
        "deep/nested/.oxlintrc.json",
        "deep/mid/.oxlintrc.json",
        "multi-ok/.oxlintrc.json",
    ] {
        assert!(!files.contains(&file), "{body}");
    }
    assert_eq!(findings.len(), 9, "{body}");
}
