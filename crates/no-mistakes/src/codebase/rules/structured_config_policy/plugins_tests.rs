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

fn plugins_yaml() -> &'static str {
    r#"
policies:
  - files: ["**/.oxlintrc.json"]
    when:
      - key: extends
    valueAssertions:
      - key: plugins
        kind: equals-file
        file: .oxlintrc.json
        fromKey: plugins
"#
}

#[test]
fn equals_file_flags_nested_plugins_drift() {
    let root = fixture_root("nested-plugins");
    let files = vec![
        root.join(".oxlintrc.json"),
        root.join("pkg/.oxlintrc.json"),
        root.join("drift/.oxlintrc.json"),
        root.join("standalone/.oxlintrc.json"),
        root.join("empty-extends/.oxlintrc.json"),
        root.join("string-extends/.oxlintrc.json"),
        root.join("omit-plugins/.oxlintrc.json"),
        root.join("bool-extends/.oxlintrc.json"),
    ];
    let findings = check_with_files(&root, &config(plugins_yaml()), &files).unwrap();
    let body = format!("{findings:?}");
    assert!(body.contains("drift/.oxlintrc.json"), "{body}");
    assert!(body.contains("omit-plugins/.oxlintrc.json"), "{body}");
    assert!(!body.contains("pkg/.oxlintrc.json"), "{body}");
    assert!(!body.contains("standalone/.oxlintrc.json"), "{body}");
    assert!(!body.contains("empty-extends"), "{body}");
    assert!(!body.contains("string-extends"), "{body}");
    assert!(!body.contains("bool-extends"), "{body}");
    assert_eq!(findings.len(), 2, "{body}");
}

#[test]
fn equals_file_reports_missing_file_and_empty_file_option() {
    let root = fixture_root("nested-plugins");
    let files = vec![root.join("pkg/.oxlintrc.json")];
    let missing = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["pkg/.oxlintrc.json"]
    valueAssertions:
      - key: plugins
        kind: equals-file
        file: missing.json
"#,
        ),
        &files,
    )
    .unwrap();
    assert!(missing[0].message.contains("missing"), "{missing:?}");

    let empty_file = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["pkg/.oxlintrc.json"]
    valueAssertions:
      - key: plugins
        kind: equals-file
"#,
        ),
        &files,
    )
    .unwrap();
    assert!(
        empty_file[0].message.contains("missing `file`"),
        "{empty_file:?}"
    );
}

#[test]
fn equals_file_reports_unreadable_target_parse_errors() {
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
        fromKey: plugins
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
}
