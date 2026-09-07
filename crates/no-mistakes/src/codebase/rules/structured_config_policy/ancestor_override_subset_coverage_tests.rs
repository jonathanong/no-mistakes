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

#[test]
fn ancestor_override_subset_skips_diamond_extends_and_malformed_overrides() {
    let root = fixture_root();
    let files = inventory(
        &root,
        &[
            "diamond/shared.json",
            "diamond/left.json",
            "diamond/right.json",
            "diamond/nested/.oxlintrc.json",
            "diamond/nested/file.ts",
            "odd-parent.json",
            "odd/.oxlintrc.json",
            "odd/file.ts",
            "bool-extends/.oxlintrc.json",
            "abs/.oxlintrc.json",
        ],
    );
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["**/.oxlintrc.json"]
    valueAssertions:
      - kind: ancestor-override-subset
        key: rules
"#,
        ),
        &files,
    )
    .unwrap();
    let body = format!("{findings:?}");
    let found: Vec<&str> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();
    assert!(!found.contains(&"diamond/nested/.oxlintrc.json"), "{body}");
    assert!(!found.contains(&"odd/.oxlintrc.json"), "{body}");
    assert!(!found.contains(&"bool-extends/.oxlintrc.json"), "{body}");
    assert!(found.contains(&"abs/.oxlintrc.json"), "{body}");
    assert_eq!(findings.len(), 1, "{body}");
    assert!(
        findings[0].message.contains("outside the repository root"),
        "{body}"
    );
}

#[test]
fn ancestor_override_subset_uses_assertion_key_as_finding_target() {
    let root = fixture_root();
    let files = inventory(
        &root,
        &[".oxlintrc.json", "lost/.oxlintrc.json", "lost/file.ts"],
    );
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["lost/.oxlintrc.json"]
    valueAssertions:
      - kind: ancestor-override-subset
        key: rules
"#,
        ),
        &files,
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("rules"));
}
