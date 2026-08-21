use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/structured-config-policy")
            .join(name),
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

#[test]
fn equals_file_compares_indexed_selectors() {
    let root = fixture_root("equals-file-selector");
    let files = vec![
        root.join("root.yml"),
        root.join("match.yml"),
        root.join("drift.yml"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["*.yml"]
    valueAssertions:
      - key: items.0.name
        kind: equals-file
        file: root.yml
        message: names must match
"#,
        ),
        &files,
    )
    .unwrap();
    let body = format!("{findings:?}");
    assert!(body.contains("drift.yml"), "{body}");
    assert!(body.contains("names must match"), "{body}");
    assert!(!body.contains("match.yml"), "{body}");
    assert_eq!(findings.len(), 1, "{body}");
}

#[test]
fn equals_file_match_any_passes_when_one_entry_equals_the_other_file() {
    let root = fixture_root("equals-file-selector");
    let files = vec![
        root.join("root.yml"),
        root.join("any.yml"),
        root.join("drift.yml"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["any.yml", "drift.yml"]
    valueAssertions:
      - key: items.[].name
        kind: equals-file
        match: any
        file: root.yml
        fromKey: items.0.name
"#,
        ),
        &files,
    )
    .unwrap();
    let body = format!("{findings:?}");
    assert!(body.contains("drift.yml"), "{body}");
    assert!(!body.contains("any.yml"), "{body}");
    assert_eq!(findings.len(), 1, "{body}");
}

#[test]
fn equals_file_reports_missing_key() {
    let root = fixture_root("nested-plugins");
    let files = vec![root.join("pkg/.oxlintrc.json"), root.join(".oxlintrc.json")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["pkg/.oxlintrc.json"]
    valueAssertions:
      - kind: equals-file
        file: .oxlintrc.json
"#,
        ),
        &files,
    )
    .unwrap();
    assert!(
        findings[0].message.contains("missing `key`"),
        "{findings:?}"
    );
}

#[test]
fn equals_file_rejects_paths_outside_the_repository_root() {
    let root = fixture_root("nested-plugins");
    let files = vec![root.join("pkg/.oxlintrc.json")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["pkg/.oxlintrc.json"]
    valueAssertions:
      - key: plugins
        kind: equals-file
        file: ../equals-file-selector/root.yml
"#,
        ),
        &files,
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("outside the repository root"),
        "{findings:?}"
    );
}

#[test]
fn equals_file_keeps_parse_errors_when_a_custom_message_is_set() {
    let root = fixture_root("nested-plugins");
    let files = vec![root.join("pkg/.oxlintrc.json"), root.join("invalid.json")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["pkg/.oxlintrc.json"]
    valueAssertions:
      - key: plugins
        kind: equals-file
        file: invalid.json
        message: plugins must match root
"#,
        ),
        &files,
    )
    .unwrap();
    assert_eq!(findings[0].file, "invalid.json", "{findings:?}");
    assert!(
        findings[0].message.contains("failed to parse JSONC"),
        "{findings:?}"
    );
    assert!(
        !findings[0].message.contains("plugins must match root"),
        "{findings:?}"
    );
}
