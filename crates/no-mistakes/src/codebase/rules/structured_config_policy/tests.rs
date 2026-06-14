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
fn reports_required_and_banned_structured_config_keys() {
    let root = fixture_root("fixture");
    let files = vec![root.join("app.yml")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: [app.yml]
    requiredKeys: [runtime.version, runtime.owner]
    bannedKeys: [legacy.enabled]
"#,
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 2);
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("runtime.owner")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("legacy.enabled")));
}

#[test]
fn ignores_invalid_structured_config_files() {
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

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn ignores_unreadable_structured_config_paths() {
    let root = fixture_root("fixture");
    let files = vec![root.join("config")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: [config]
    requiredKeys: [runtime.version]
"#,
        ),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn reports_value_assertion_violations() {
    let root = fixture_root("fixture");
    let files = vec![root.join("app.yml")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: [app.yml]
    valueAssertions:
      - key: features.strict
        kind: boolean
      - key: limits.retries
        kind: positive-number
      - key: badFiles
        kind: string-array
      - key: overrides.[].files.[]
        kind: string-prefix
        prefix: "**/"
      - key: overrides.[].files.[]
        kind: not-single-file
      - key: globTarget
        kind: string-glob
        glob: "src/**/*.ts"
      - key: runtime.mode
        kind: equals
        value: production
      - key: rules.[]
        kind: object-shape
        requiredValues:
          severity: error
"#,
        ),
        &files,
    )
    .unwrap();
    let body = format!("{findings:?}");

    assert_eq!(findings.len(), 8, "{body}");
    assert!(body.contains("strict boolean"), "{body}");
    assert!(body.contains("positive number"), "{body}");
    assert!(body.contains("array of strings"), "{body}");
    assert!(body.contains("starting with `**/`"), "{body}");
    assert!(body.contains("single-file entry"), "{body}");
    assert!(body.contains("match glob `src/**/*.ts`"), "{body}");
    assert!(body.contains("must equal"), "{body}");
    assert!(body.contains("severity"), "{body}");
}

#[test]
fn value_assertions_pass_for_matching_values() {
    let root = fixture_root("fixture");
    let value: serde_yaml::Value = serde_yaml::from_str(
        r#"
enabled: true
count: 2
files: ["**/*.ts"]
entry:
  severity: error
"#,
    )
    .unwrap();
    let assertions = [
        ValueAssertion {
            key: "enabled".to_string(),
            kind: Some(AssertionKind::Boolean),
            ..Default::default()
        },
        ValueAssertion {
            key: "count".to_string(),
            kind: Some(AssertionKind::PositiveNumber),
            ..Default::default()
        },
        ValueAssertion {
            key: "files".to_string(),
            kind: Some(AssertionKind::StringArray),
            ..Default::default()
        },
        ValueAssertion {
            key: "files.[]".to_string(),
            kind: Some(AssertionKind::NotSingleFile),
            ..Default::default()
        },
        ValueAssertion {
            key: "entry".to_string(),
            kind: Some(AssertionKind::ObjectShape),
            required_values: [(
                "severity".to_string(),
                serde_yaml::Value::String("error".to_string()),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        },
    ];

    for assertion in assertions {
        let findings = assert_value("app.yml", &value, &assertion).unwrap();
        assert!(findings.is_empty(), "{root:?} {findings:?}");
    }
}

#[test]
fn value_assertions_cover_defensive_branches() {
    let value: serde_yaml::Value = serde_yaml::from_str(
        r#"
name: "@acme/api"
settings:
  severity: warn
items:
  - name: "@acme/web"
"#,
    )
    .unwrap();

    let findings = assert_value(
        "app.yml",
        &value,
        &ValueAssertion {
            key: "missing".to_string(),
            kind: Some(AssertionKind::StringArray),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(findings[0].message.contains("required by assertion"));

    assert!(assert_value(
        "app.yml",
        &value,
        &ValueAssertion {
            key: "name".to_string(),
            ..Default::default()
        },
    )
    .unwrap()
    .is_empty());
    assert!(assert_value(
        "app.yml",
        &value,
        &ValueAssertion {
            kind: Some(AssertionKind::Boolean),
            ..Default::default()
        },
    )
    .unwrap()
    .is_empty());
    assert!(assert_value(
        "app.yml",
        &value,
        &ValueAssertion {
            key: "name".to_string(),
            kind: Some(AssertionKind::Equals),
            ..Default::default()
        },
    )
    .unwrap()
    .is_empty());
    assert!(assert_value(
        "app.yml",
        &value,
        &ValueAssertion {
            key: "items.0.name".to_string(),
            kind: Some(AssertionKind::StringPrefix),
            prefix: "@acme/".to_string(),
            ..Default::default()
        },
    )
    .unwrap()
    .is_empty());

    let invalid_glob = assert_value(
        "app.yml",
        &value,
        &ValueAssertion {
            key: "name".to_string(),
            kind: Some(AssertionKind::StringGlob),
            glob: "[".to_string(),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(invalid_glob[0].message.contains("invalid glob"));

    let object_findings = assert_value(
        "app.yml",
        &value,
        &ValueAssertion {
            key: "settings".to_string(),
            kind: Some(AssertionKind::ObjectShape),
            required_keys: vec!["owner".to_string()],
            ..Default::default()
        },
    )
    .unwrap();
    assert!(object_findings[0].message.contains("object key `owner`"));

    let non_object = assert_value(
        "app.yml",
        &value,
        &ValueAssertion {
            key: "name".to_string(),
            kind: Some(AssertionKind::ObjectShape),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(non_object[0].message.contains("must be an object"));

    for key in ["name.[]", "items.9", "name.0"] {
        let findings = assert_value(
            "app.yml",
            &value,
            &ValueAssertion {
                key: key.to_string(),
                kind: Some(AssertionKind::StringArray),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(findings[0].message.contains("required by assertion"));
    }

    let non_string_prefix = assert_value(
        "app.yml",
        &value,
        &ValueAssertion {
            key: "settings".to_string(),
            kind: Some(AssertionKind::StringPrefix),
            prefix: "@acme/".to_string(),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(non_string_prefix[0].message.contains("starting with"));

    assert!(assert_value(
        "app.yml",
        &value,
        &ValueAssertion {
            key: "settings".to_string(),
            kind: Some(AssertionKind::ObjectShape),
            required_keys: vec!["severity".to_string()],
            ..Default::default()
        },
    )
    .unwrap()
    .is_empty());

    let missing_required_value = assert_value(
        "app.yml",
        &value,
        &ValueAssertion {
            key: "settings".to_string(),
            kind: Some(AssertionKind::ObjectShape),
            required_values: [(
                "owner".to_string(),
                serde_yaml::Value::String("@acme/api".to_string()),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(missing_required_value[0]
        .message
        .contains("object key `owner`"));

    let wildcard_missing = assert_value(
        "app.yml",
        &serde_yaml::from_str(
            r#"
overrides:
  - files: ["**/*.ts"]
  - {}
"#,
        )
        .unwrap(),
        &ValueAssertion {
            key: "overrides.[].files".to_string(),
            kind: Some(AssertionKind::StringArray),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(wildcard_missing.len(), 1);
    assert!(wildcard_missing[0]
        .message
        .contains("required by assertion"));
}

#[test]
fn unknown_value_assertion_kinds_do_not_disable_the_policy() {
    let root = fixture_root("fixture");
    let files = vec![root.join("app.yml")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["app.yml"]
    requiredKeys: [missingRequiredKey]
    valueAssertions:
      - key: enabled
        kind: boolish
"#,
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1, "unexpected findings: {findings:?}");
    assert!(findings[0].message.contains("missingRequiredKey"));
}
