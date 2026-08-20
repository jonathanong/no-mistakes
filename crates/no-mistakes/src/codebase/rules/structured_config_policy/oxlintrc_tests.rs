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

fn bind_override_yaml() -> &'static str {
    r#"
policies:
  - files: ["*.json"]
    valueAssertions:
      - key: overrides.[].rules.no-restricted-properties.[]
        kind: object-shape
        match: any
        requiredKeys: [message]
        forbiddenKeys: [object]
        requiredValues:
          property: bind
"#
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
fn not_single_file_strips_star_star_slash_prefix() {
    let root = fixture_root("prefix-single-file");
    let files = vec![root.join("app.yml")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: [app.yml]
    valueAssertions:
      - key: overrides.[].files.[]
        kind: not-single-file
"#,
        ),
        &files,
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("single-file entry"));
}

#[test]
fn match_any_fails_per_override_that_drops_bind() {
    let root = fixture_root("overrides-bind");
    let files = vec![root.join("drop.json")];
    let findings = check_with_files(&root, &config(bind_override_yaml()), &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("at least one"));
}

#[test]
fn match_any_skips_overrides_that_omit_the_rule() {
    let root = fixture_root("overrides-bind");
    let files = vec![root.join("omit.json")];
    let findings = check_with_files(&root, &config(bind_override_yaml()), &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
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
    assert!(
        findings[0].message.contains("failed to parse JSONC"),
        "{findings:?}"
    );
}

#[test]
fn match_any_groups_root_arrays_and_indexed_rest() {
    let root_array: serde_yaml::Value =
        serde_yaml::from_str("- name: keep\n- name: drop\n").unwrap();
    let root_any = assert_value(
        "app.yml",
        &root_array,
        &ValueAssertion {
            key: "[].name".to_string(),
            kind: Some(AssertionKind::Equals),
            match_mode: MatchMode::Any,
            value: Some(serde_yaml::Value::String("keep".to_string())),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(root_any.is_empty(), "{root_any:?}");

    let items: serde_yaml::Value = serde_yaml::from_str(
        r#"
items:
  - name: keep
  - extra: 1
"#,
    )
    .unwrap();
    let rest = assert_value(
        "app.yml",
        &items,
        &ValueAssertion {
            key: "items.[].name".to_string(),
            kind: Some(AssertionKind::Equals),
            match_mode: MatchMode::Any,
            value: Some(serde_yaml::Value::String("keep".to_string())),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(rest.is_empty(), "{rest:?}");

    let missing = assert_value(
        "app.yml",
        &items,
        &ValueAssertion {
            key: "missing.[].name".to_string(),
            kind: Some(AssertionKind::Equals),
            match_mode: MatchMode::Any,
            value: Some(serde_yaml::Value::String("keep".to_string())),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(missing.is_empty(), "{missing:?}");

    let empty_array: serde_yaml::Value =
        serde_yaml::from_str("overrides:\n  - rules:\n      no-restricted-properties: []\n")
            .unwrap();
    let empty = assert_value(
        "app.yml",
        &empty_array,
        &ValueAssertion {
            key: "overrides.[].rules.no-restricted-properties.[]".to_string(),
            kind: Some(AssertionKind::ObjectShape),
            match_mode: MatchMode::Any,
            required_values: [(
                "property".to_string(),
                serde_yaml::Value::String("bind".to_string()),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(empty.len(), 1, "{empty:?}");

    let indexed = assert_value(
        "app.yml",
        &items,
        &ValueAssertion {
            key: "items.9.name".to_string(),
            kind: Some(AssertionKind::Equals),
            match_mode: MatchMode::Any,
            value: Some(serde_yaml::Value::String("keep".to_string())),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(indexed.is_empty(), "{indexed:?}");

    let not_array = assert_value(
        "app.yml",
        &serde_yaml::from_str("name: keep\n").unwrap(),
        &ValueAssertion {
            key: "name.[]".to_string(),
            kind: Some(AssertionKind::Equals),
            match_mode: MatchMode::Any,
            value: Some(serde_yaml::Value::String("keep".to_string())),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(not_array.is_empty(), "{not_array:?}");

    let not_seq_index = assert_value(
        "app.yml",
        &serde_yaml::from_str("name: keep\n").unwrap(),
        &ValueAssertion {
            key: "name.0".to_string(),
            kind: Some(AssertionKind::Equals),
            match_mode: MatchMode::Any,
            value: Some(serde_yaml::Value::String("keep".to_string())),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(not_seq_index.is_empty(), "{not_seq_index:?}");
}
