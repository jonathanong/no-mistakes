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
fn reports_parse_errors_for_invalid_yaml() {
    let root = fixture_root("fixture");
    let files = vec![root.join("invalid.yml")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: [invalid.yml]
    requiredKeys: [runtime.version]
"#,
        ),
        &files,
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("failed to parse YAML"));
}

#[test]
fn reports_banned_keys_in_commented_jsonc() {
    let root = fixture_root("jsonc");
    let files = vec![root.join(".oxlintrc.json")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: [".oxlintrc.json"]
    bannedKeys: [legacy]
"#,
        ),
        &files,
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("legacy"));
}

#[test]
fn any_of_object_shape_passes_when_one_entry_matches() {
    let root = fixture_root("jsonc");
    let files = vec![root.join(".oxlintrc.json")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: [".oxlintrc.json"]
    valueAssertions:
      - key: rules.no-restricted-properties.[]
        kind: object-shape
        match: any
        requiredKeys: [message]
        forbiddenKeys: [object]
        requiredValues:
          property: bind
"#,
        ),
        &files,
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn any_of_object_shape_fails_when_no_entry_matches() {
    let root = fixture_root("jsonc");
    let files = vec![root.join("no-bind.json")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["no-bind.json"]
    valueAssertions:
      - key: rules.no-restricted-properties.[]
        kind: object-shape
        match: any
        requiredKeys: [message]
        forbiddenKeys: [object]
        requiredValues:
          property: bind
"#,
        ),
        &files,
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("at least one"));
}

#[test]
fn forbidden_keys_flag_object_presence() {
    let value: serde_yaml::Value =
        serde_yaml::from_str("entry:\n  object: Function\n  property: bind\n").unwrap();
    let findings = assert_value(
        "app.yml",
        &value,
        &ValueAssertion {
            key: "entry".to_string(),
            kind: Some(AssertionKind::ObjectShape),
            forbidden_keys: vec!["object".to_string()],
            ..Default::default()
        },
    )
    .unwrap();
    assert!(findings[0]
        .message
        .contains("must not contain object key `object`"));
}
