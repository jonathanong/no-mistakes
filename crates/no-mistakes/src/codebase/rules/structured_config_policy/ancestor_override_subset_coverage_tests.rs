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
fn ancestor_override_subset_rejects_malformed_overrides_and_keeps_valid_configs() {
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
            // Oxlint `*` does not match `/`. `star-seg/*.ts` must not treat
            // `star-seg/inner/file.ts` as a lost ancestor override.
            "star-seg-parent.json",
            "star-seg/.oxlintrc.json",
            "star-seg/inner/file.ts",
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
    assert!(found.contains(&"odd/.oxlintrc.json"), "{body}");
    assert!(found.contains(&"bool-extends/.oxlintrc.json"), "{body}");
    assert!(!found.contains(&"star-seg/.oxlintrc.json"), "{body}");
    assert!(found.contains(&"abs/.oxlintrc.json"), "{body}");
    assert_eq!(findings.len(), 3, "{body}");
    assert!(
        findings
            .iter()
            .any(|finding| finding.file == "abs/.oxlintrc.json"
                && finding.message.contains("portable relative path")),
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

#[test]
fn ancestor_override_subset_uses_in_scope_children_when_include_is_configs_only() {
    let root = fixture_root();
    let files = inventory(
        &root,
        &[".oxlintrc.json", "lost/.oxlintrc.json", "lost/file.ts"],
    );
    let mut config = config(
        r#"
policies:
  - files: ["lost/.oxlintrc.json"]
    valueAssertions:
      - kind: ancestor-override-subset
"#,
    );
    config.rules[0].include = vec!["**/.oxlintrc.json".to_string()];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "lost/.oxlintrc.json");
}
